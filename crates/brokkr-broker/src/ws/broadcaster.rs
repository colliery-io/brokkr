/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! In-memory per-stack fan-out of agent telemetry frames.
//!
//! Sits between the WS reader (`dispatch_uplink`) and the live-tail
//! subscription endpoint (WS-11). Agents stream `K8sEvent` / `PodLogLine`
//! / `LogGap` frames upstream; this hub broadcasts them, *unmodified*, to
//! any UI/SDK clients currently subscribed to the originating stack.
//!
//! ## Slow-subscriber policy
//!
//! Each per-stack channel is bounded (`CHANNEL_CAPACITY`). Subscribers
//! that fall behind get `RecvError::Lagged(n)` from `tokio::sync::broadcast`,
//! which the subscription handler converts into a synthetic `LogGap` so
//! the UI can render a visible gap rather than silently dropping frames.
//! This is per ADR-0008: "a slow subscriber must not slow ingestion".

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use brokkr_wire::WsMessage;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Per-stack broadcast capacity. Sized for a moderate burst of log lines
/// from a chatty workload (≈ a few seconds of headroom at 100 lines/s);
/// over this, subscribers see a `Lagged` and gap-mark.
const CHANNEL_CAPACITY: usize = 1024;

#[derive(Default)]
pub struct LiveBroadcaster {
    channels: RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>,
}

impl LiveBroadcaster {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Send a frame to every subscriber of `stack_id`. No-op when nobody
    /// is subscribed; `broadcast::Sender::send` errors are intentionally
    /// swallowed (an empty channel returns `SendError`).
    pub fn broadcast(&self, stack_id: Uuid, msg: WsMessage) {
        let channels = self.channels.read().expect("broadcaster poisoned");
        if let Some(tx) = channels.get(&stack_id) {
            let _ = tx.send(msg);
        }
    }

    /// Subscribe to all future frames for `stack_id`. Creates the
    /// underlying channel lazily on first subscription.
    pub fn subscribe(&self, stack_id: Uuid) -> broadcast::Receiver<WsMessage> {
        let mut channels = self.channels.write().expect("broadcaster poisoned");
        let sender = channels
            .entry(stack_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);
        sender.subscribe()
    }

    /// Diagnostics: number of stacks with at least one live subscriber.
    pub fn stack_count(&self) -> usize {
        self.channels.read().expect("broadcaster poisoned").len()
    }

    /// Diagnostics: total subscriber count across all stacks.
    pub fn subscriber_count(&self) -> usize {
        self.channels
            .read()
            .expect("broadcaster poisoned")
            .values()
            .map(|tx| tx.receiver_count())
            .sum()
    }
}

/// Fleet-wide broadcast capacity. A single channel fans every per-agent
/// `FleetUpdate` out to every `/api/v1/fleet/live` subscriber. Sized for a
/// burst of fleet churn (many agents connecting/heartbeating at once); over
/// this, a slow subscriber sees a `Lagged` and the handler simply continues
/// — there is no per-agent gap concept for fleet records (the consumer holds
/// the latest record per agent_id, so a missed update is superseded by the
/// next one for that agent).
const FLEET_CHANNEL_CAPACITY: usize = 1024;

/// In-memory fleet-wide fan-out of per-agent `FleetUpdate` frames
/// (BROKKR-I-0028).
///
/// Unlike [`LiveBroadcaster`], this is a *single* channel, not keyed per
/// stack: every `/api/v1/fleet/live` subscriber receives every agent's
/// updates. Same slow-subscriber policy as the per-stack hub (ADR-0008): a
/// lagging subscriber gets `RecvError::Lagged` and the handler drops/continues
/// — broadcasting never blocks the triggering operation.
pub struct FleetBroadcaster {
    tx: broadcast::Sender<WsMessage>,
}

impl Default for FleetBroadcaster {
    fn default() -> Self {
        Self {
            tx: broadcast::channel(FLEET_CHANNEL_CAPACITY).0,
        }
    }
}

impl FleetBroadcaster {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Broadcast one frame to every fleet subscriber. No-op when nobody is
    /// subscribed (`broadcast::Sender::send` returns `SendError`, swallowed).
    /// Never blocks — this is called from producer hot paths where a push
    /// failure must never affect the triggering operation.
    pub fn broadcast(&self, msg: WsMessage) {
        let _ = self.tx.send(msg);
    }

    /// Subscribe to all future fleet frames.
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    /// Diagnostics: current number of fleet-live subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_wire::{FleetAgentRecord, Heartbeat, K8sEvent, ObjectRef};
    use chrono::Utc;

    fn evt(stack_id: Uuid) -> WsMessage {
        WsMessage::K8sEvent(K8sEvent {
            agent_id: Uuid::new_v4(),
            stack_id,
            observed_at: Utc::now(),
            reason: "T".into(),
            message: "x".into(),
            event_type: "Normal".into(),
            source: None,
            involved_object: ObjectRef {
                api_version: "v1".into(),
                kind: "Pod".into(),
                namespace: None,
                name: "p".into(),
                uid: None,
            },
        })
    }

    #[tokio::test]
    async fn broadcast_with_no_subscribers_is_a_noop() {
        let b = LiveBroadcaster::default();
        b.broadcast(Uuid::new_v4(), evt(Uuid::new_v4())); // no panic
    }

    #[tokio::test]
    async fn subscriber_receives_only_their_stack() {
        let b = LiveBroadcaster::default();
        let s1 = Uuid::new_v4();
        let s2 = Uuid::new_v4();
        let mut rx = b.subscribe(s1);

        b.broadcast(s2, evt(s2)); // wrong stack — must not arrive
        b.broadcast(s1, evt(s1));

        let m = rx.recv().await.unwrap();
        let WsMessage::K8sEvent(e) = m else {
            panic!("wrong variant")
        };
        assert_eq!(e.stack_id, s1);
    }

    #[tokio::test]
    async fn diagnostic_counters_track_subscriptions() {
        let b = LiveBroadcaster::default();
        let s = Uuid::new_v4();
        assert_eq!(b.stack_count(), 0);
        let _rx1 = b.subscribe(s);
        let _rx2 = b.subscribe(s);
        assert_eq!(b.stack_count(), 1);
        assert_eq!(b.subscriber_count(), 2);
    }

    // sanity: ws-message types other than telemetry are still broadcast
    // verbatim if the caller asks — the broadcaster is intentionally
    // dumb about content.
    #[tokio::test]
    async fn broadcaster_does_not_filter_by_message_type() {
        let b = LiveBroadcaster::default();
        let s = Uuid::new_v4();
        let mut rx = b.subscribe(s);
        b.broadcast(
            s,
            WsMessage::Heartbeat(Heartbeat {
                agent_id: Uuid::nil(),
                sent_at: Utc::now(),
                k8s_reachable: None,
                k8s_api_latency_ms: None,
            }),
        );
        assert!(matches!(rx.recv().await.unwrap(), WsMessage::Heartbeat(_)));
    }

    fn fleet_record(agent_id: Uuid) -> WsMessage {
        WsMessage::FleetUpdate(FleetAgentRecord {
            agent_id,
            name: "a".into(),
            status: "ACTIVE".into(),
            ws_connected: true,
            connected_since: Some(Utc::now()),
            last_heartbeat: Some(Utc::now()),
            heartbeat_age_seconds: Some(0),
            pending_object_count: 0,
            pending_work_orders: 0,
            claimed_work_orders: 0,
            last_event_at: None,
            seconds_since_last_event: None,
            health_failing: 0,
            health_degraded: 0,
            k8s_reachable: None,
            k8s_api_latency_ms: None,
        })
    }

    #[tokio::test]
    async fn fleet_live_broadcast_with_no_subscribers_is_a_noop() {
        let b = FleetBroadcaster::default();
        b.broadcast(fleet_record(Uuid::new_v4())); // no panic
        assert_eq!(b.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn fleet_live_subscriber_receives_fleet_update() {
        let b = FleetBroadcaster::default();
        let mut rx = b.subscribe();
        let id = Uuid::new_v4();
        b.broadcast(fleet_record(id));
        let m = rx.recv().await.unwrap();
        let WsMessage::FleetUpdate(rec) = m else {
            panic!("wrong variant")
        };
        assert_eq!(rec.agent_id, id);
    }

    // ADR-0008: a slow fleet subscriber must not stall the producer. We fill
    // the channel past capacity without draining; broadcast() still returns
    // immediately and the lagged receiver observes a Lagged error rather than
    // blocking anyone.
    #[tokio::test]
    async fn fleet_live_slow_subscriber_does_not_stall_producer() {
        let b = FleetBroadcaster::default();
        let mut rx = b.subscribe();
        for _ in 0..(FLEET_CHANNEL_CAPACITY + 16) {
            b.broadcast(fleet_record(Uuid::new_v4())); // never blocks
        }
        // The slow subscriber that fell behind sees Lagged, not a hang.
        match rx.recv().await {
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
            other => panic!("expected Lagged, got {other:?}"),
        }
    }
}
