/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Per-agent WebSocket connection registry.
//!
//! Tracks which agents currently hold an open WebSocket connection to this
//! broker process and routes outgoing messages to them on one of two
//! priority lanes. Control-plane messages (work orders, target/stack
//! changes, heartbeat) use the control lane; logs and kube events use the
//! telemetry lane. The per-connection writer task drains control first
//! (see [`handler`](super::handler)).
//!
//! Connections are keyed by `agent_id`. If a second connection arrives for
//! the same agent (reconnect, duplicate process), it replaces the existing
//! handle — dropping the prior senders causes the old writer task to wind
//! down cleanly.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

use brokkr_wire::WsMessage;

/// Errors returned when trying to push a message to a registered agent.
#[derive(Debug)]
pub enum SendError {
    /// No connection exists for the given agent id.
    NotConnected(Uuid),
    /// The connection exists but its lane is full or the writer has gone away.
    /// In practice this means the agent is either too slow or dying; callers
    /// should treat it the same as "not connected" and let the REST polling
    /// fallback surface the change on the next tick.
    LaneUnavailable(Uuid),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConnected(id) => write!(f, "agent {id} is not connected"),
            Self::LaneUnavailable(id) => {
                write!(f, "send to agent {id} failed (lane full or closed)")
            }
        }
    }
}

impl std::error::Error for SendError {}

/// Sender-side handle for a single registered connection.
///
/// Owned by the registry. Cloned cheaply (just `Arc`s and mpsc senders).
/// The corresponding receiver halves are held by the connection's writer
/// task — dropping all senders for a lane signals the writer to exit.
#[derive(Clone)]
pub struct ConnectionHandle {
    pub agent_id: Uuid,
    pub connected_since: DateTime<Utc>,
    pub messages_in: Arc<AtomicU64>,
    pub messages_out: Arc<AtomicU64>,
    pub(super) control_tx: mpsc::Sender<WsMessage>,
    pub(super) telemetry_tx: mpsc::Sender<WsMessage>,
}

/// Snapshot view of one connection for diagnostics endpoints (WS-13).
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub agent_id: Uuid,
    pub connected_since: DateTime<Utc>,
    pub messages_in: u64,
    pub messages_out: u64,
}

/// Per-broker-process registry of live agent connections.
///
/// Wrap in `Arc` and share via `axum::Extension` so handlers and future
/// broker-push code (WS-04) can route messages without depending on a
/// specific app-state struct.
#[derive(Default)]
pub struct ConnectionRegistry {
    inner: RwLock<HashMap<Uuid, ConnectionHandle>>,
}

impl ConnectionRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Insert a new handle, evicting any prior connection for the same agent.
    pub fn register(&self, handle: ConnectionHandle) {
        let agent_id = handle.agent_id;
        let mut map = self.inner.write().expect("registry poisoned");
        map.insert(agent_id, handle);
    }

    /// Remove the handle iff it still matches the writer's `connected_since`
    /// timestamp. This prevents a late-cleaning writer from unregistering a
    /// fresh reconnect that happens to share its agent id.
    pub fn unregister_if_matches(&self, agent_id: Uuid, connected_since: DateTime<Utc>) {
        let mut map = self.inner.write().expect("registry poisoned");
        if let Some(existing) = map.get(&agent_id)
            && existing.connected_since == connected_since
        {
            map.remove(&agent_id);
        }
    }

    /// True if any handle is registered for this agent.
    pub fn is_connected(&self, agent_id: Uuid) -> bool {
        self.inner
            .read()
            .expect("registry poisoned")
            .contains_key(&agent_id)
    }

    /// Send a control-plane message to a specific agent. Non-blocking;
    /// returns immediately with `SendError::LaneUnavailable` if the lane
    /// queue is full (back-pressure). Use this for `WorkOrder`,
    /// `TargetChanged`, `StackChanged`, `Heartbeat`.
    pub fn send_control(&self, agent_id: Uuid, msg: WsMessage) -> Result<(), SendError> {
        let handle = self
            .inner
            .read()
            .expect("registry poisoned")
            .get(&agent_id)
            .cloned()
            .ok_or(SendError::NotConnected(agent_id))?;
        handle
            .control_tx
            .try_send(msg)
            .map_err(|_| SendError::LaneUnavailable(agent_id))
    }

    /// Send a telemetry/log message to a specific agent on the low-priority
    /// lane. Use this for `K8sEvent`, `PodLogLine`, `LogGap` when the broker
    /// originates fan-out (which it does not in v1, but the lane exists for
    /// symmetry and for future broker→agent telemetry).
    pub fn send_telemetry(&self, agent_id: Uuid, msg: WsMessage) -> Result<(), SendError> {
        let handle = self
            .inner
            .read()
            .expect("registry poisoned")
            .get(&agent_id)
            .cloned()
            .ok_or(SendError::NotConnected(agent_id))?;
        handle
            .telemetry_tx
            .try_send(msg)
            .map_err(|_| SendError::LaneUnavailable(agent_id))
    }

    /// Snapshot every connection for diagnostics. O(n) clone — only call
    /// from admin/diagnostics paths.
    pub fn snapshot(&self) -> Vec<ConnectionInfo> {
        self.inner
            .read()
            .expect("registry poisoned")
            .values()
            .map(|h| ConnectionInfo {
                agent_id: h.agent_id,
                connected_since: h.connected_since,
                messages_in: h.messages_in.load(Ordering::Relaxed),
                messages_out: h.messages_out.load(Ordering::Relaxed),
            })
            .collect()
    }

    /// Number of connected agents (cheap; no clone).
    pub fn connected_count(&self) -> usize {
        self.inner.read().expect("registry poisoned").len()
    }

    /// Forcibly close any live connection for `agent_id`, returning how many
    /// were closed (0 or 1 today — the map is one-connection-per-agent, since
    /// [`register`] evicts a prior handle for the same id).
    ///
    /// Removing the handle drops its lane senders; the per-connection writer
    /// task then observes both lanes closed, closes the socket, and the
    /// connection's `run_connection` loop unwinds (decrementing the connected
    /// gauge). This is the teardown path used by PAK revocation
    /// ([[BROKKR-T-0176]]): once an agent's PAK is invalidated we must not
    /// leave its already-upgraded socket open until its TCP layer happens to
    /// notice. The agent's reconnect will re-hit auth and be rejected.
    pub fn close_for_agent(&self, agent_id: Uuid) -> usize {
        let mut map = self.inner.write().expect("registry poisoned");
        if map.remove(&agent_id).is_some() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_wire::Heartbeat;

    fn handle_for(
        agent_id: Uuid,
    ) -> (
        ConnectionHandle,
        mpsc::Receiver<WsMessage>,
        mpsc::Receiver<WsMessage>,
    ) {
        let (control_tx, control_rx) = mpsc::channel(8);
        let (telemetry_tx, telemetry_rx) = mpsc::channel(8);
        let handle = ConnectionHandle {
            agent_id,
            connected_since: Utc::now(),
            messages_in: Arc::new(AtomicU64::new(0)),
            messages_out: Arc::new(AtomicU64::new(0)),
            control_tx,
            telemetry_tx,
        };
        (handle, control_rx, telemetry_rx)
    }

    fn sample_heartbeat(agent_id: Uuid) -> WsMessage {
        WsMessage::Heartbeat(Heartbeat {
            agent_id,
            sent_at: Utc::now(),
            k8s_reachable: None,
            k8s_api_latency_ms: None,
        })
    }

    #[test]
    fn send_to_unknown_agent_errors() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        let err = reg.send_control(id, sample_heartbeat(id)).unwrap_err();
        assert!(matches!(err, SendError::NotConnected(_)));
    }

    #[test]
    fn register_then_send_lands_on_correct_lane() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        let (handle, mut control_rx, mut telemetry_rx) = handle_for(id);
        reg.register(handle);
        assert!(reg.is_connected(id));
        assert_eq!(reg.connected_count(), 1);

        reg.send_control(id, sample_heartbeat(id)).unwrap();
        reg.send_telemetry(id, sample_heartbeat(id)).unwrap();

        assert!(matches!(control_rx.try_recv(), Ok(WsMessage::Heartbeat(_))));
        assert!(matches!(
            telemetry_rx.try_recv(),
            Ok(WsMessage::Heartbeat(_))
        ));
    }

    #[test]
    fn second_register_evicts_first() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        let (first, _c1, _t1) = handle_for(id);
        let first_ts = first.connected_since;
        reg.register(first);

        // Make the second handle have a guaranteed-different timestamp.
        std::thread::sleep(std::time::Duration::from_millis(2));
        let (second, mut c2, _t2) = handle_for(id);
        reg.register(second);

        reg.send_control(id, sample_heartbeat(id)).unwrap();
        // The control lane that's still receiving must be the second one.
        assert!(c2.try_recv().is_ok());

        // Stale unregister keyed on the first connection's timestamp is a no-op.
        reg.unregister_if_matches(id, first_ts);
        assert!(reg.is_connected(id));
    }

    #[test]
    fn unregister_if_matches_removes_only_matching_generation() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        let (handle, _c, _t) = handle_for(id);
        let ts = handle.connected_since;
        reg.register(handle);
        reg.unregister_if_matches(id, ts);
        assert!(!reg.is_connected(id));
    }

    #[test]
    fn close_for_agent_removes_handle_and_drops_senders() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        let (handle, mut control_rx, _telemetry_rx) = handle_for(id);
        reg.register(handle);
        assert!(reg.is_connected(id));

        // Closing returns 1 and removes the entry.
        assert_eq!(reg.close_for_agent(id), 1);
        assert!(!reg.is_connected(id));

        // The writer-side receiver observes the dropped sender (channel
        // closed) — this is what unblocks the per-connection writer task to
        // close the socket.
        assert!(control_rx.try_recv().is_err());

        // Closing an unknown / already-closed agent is a no-op returning 0.
        assert_eq!(reg.close_for_agent(id), 0);
        assert_eq!(reg.close_for_agent(Uuid::new_v4()), 0);
    }

    #[test]
    fn lane_full_returns_lane_unavailable() {
        let reg = ConnectionRegistry::default();
        let id = Uuid::new_v4();
        // Lane capacity 1; never drain.
        let (control_tx, _control_rx) = mpsc::channel(1);
        let (telemetry_tx, _telemetry_rx) = mpsc::channel(1);
        reg.register(ConnectionHandle {
            agent_id: id,
            connected_since: Utc::now(),
            messages_in: Arc::new(AtomicU64::new(0)),
            messages_out: Arc::new(AtomicU64::new(0)),
            control_tx,
            telemetry_tx,
        });
        reg.send_control(id, sample_heartbeat(id)).unwrap();
        let err = reg.send_control(id, sample_heartbeat(id)).unwrap_err();
        assert!(matches!(err, SendError::LaneUnavailable(_)));
    }
}
