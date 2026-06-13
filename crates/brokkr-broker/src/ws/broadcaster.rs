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

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_wire::{Heartbeat, K8sEvent, ObjectRef};
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
}
