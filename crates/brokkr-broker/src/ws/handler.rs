/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Axum handler for the internal `/internal/ws/agent` upgrade endpoint.
//!
//! Auth is enforced by the same PAK middleware as the rest of the v1 API
//! (re-used from [`crate::api::v1::middleware`]); only PAKs that resolve to
//! an *agent* identity are allowed to upgrade — admin and generator PAKs
//! get a 403. After the upgrade succeeds, a connection handle is published
//! to the shared [`ConnectionRegistry`] and two tasks run:
//!
//! - **reader**: pulls frames from the socket, parses them as
//!   [`brokkr_wire::WsMessage`], and bumps the inbound counter. Future
//!   tasks (WS-05 uplink, WS-09 ingestion) will dispatch from here.
//! - **writer**: drains the control lane first, falling back to telemetry,
//!   and forwards messages to the socket. This is what guarantees the
//!   ADR's "control plane is never starved by log/event traffic" property.
//!
//! Both tasks share a cancellation token: when either ends (socket close,
//! lane drop, decode error, etc.) the other is unblocked so the agent's
//! entry is removed from the registry cleanly.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::{
    Router,
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Extension, State},
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
};
use brokkr_models::models::agent_events::NewAgentEvent;
use brokkr_models::models::agent_k8s_events::NewAgentK8sEvent;
use brokkr_models::models::agent_pod_logs::NewAgentPodLog;
use brokkr_models::models::deployment_health::NewDeploymentHealth;
use brokkr_wire::WsMessage;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::api::v1::middleware::{self as v1_middleware, AuthPayload};
use crate::dal::DAL;
use crate::metrics;

use super::broadcaster::{FleetBroadcaster, LiveBroadcaster};
use super::registry::{ConnectionHandle, ConnectionRegistry};

/// Public path of the internal WS endpoint. Exposed as a constant so tests
/// and the agent client (WS-03) can reference it without string drift.
pub const INTERNAL_WS_PATH: &str = "/internal/ws/agent";

/// Capacity of the per-connection control lane. Small on purpose: control
/// pushes (work order created, target/stack changed) are rare. A full lane
/// means the agent is wedged — better to surface back-pressure to callers
/// than to buffer indefinitely.
const CONTROL_LANE_CAPACITY: usize = 64;

/// Capacity of the per-connection telemetry lane. Larger to absorb log
/// burst from a chatty workload, but still bounded — agent-side
/// rate-limiting (WS-08) is the primary defence; this is the second line.
const TELEMETRY_LANE_CAPACITY: usize = 1024;

/// Build the standalone router that mounts the internal WS endpoint.
///
/// Mounted *outside* `/api/v1` (so it never appears in the OpenAPI spec)
/// but still behind the same PAK auth middleware as the rest of the
/// authenticated surface. The registry is injected via [`Extension`] so
/// future push code can grab it from the same handle.
pub fn internal_routes(
    dal: DAL,
    registry: Arc<ConnectionRegistry>,
    broadcaster: Arc<LiveBroadcaster>,
    fleet: Arc<FleetBroadcaster>,
) -> Router<DAL> {
    Router::new()
        .route(INTERNAL_WS_PATH, get(ws_upgrade))
        .layer(Extension(registry))
        .layer(Extension(broadcaster))
        .layer(Extension(fleet))
        .layer(from_fn_with_state(
            dal,
            v1_middleware::auth_middleware::<Body>,
        ))
}

async fn ws_upgrade(
    upgrade: WebSocketUpgrade,
    State(dal): State<DAL>,
    Extension(registry): Extension<Arc<ConnectionRegistry>>,
    Extension(broadcaster): Extension<Arc<LiveBroadcaster>>,
    Extension(fleet): Extension<Arc<FleetBroadcaster>>,
    request: Request<Body>,
) -> Result<axum::response::Response, StatusCode> {
    // The auth middleware ran upstream and inserted an AuthPayload. The WS
    // channel is exclusively for agents — refuse admin / generator PAKs.
    let auth = request
        .extensions()
        .get::<AuthPayload>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let agent_id = match auth.agent {
        Some(id) => id,
        None => {
            warn!(
                "WS upgrade rejected: PAK is valid but not an agent identity (admin={}, generator={:?})",
                auth.admin, auth.generator
            );
            return Err(StatusCode::FORBIDDEN);
        }
    };

    info!(%agent_id, "agent WS upgrade accepted");
    Ok(upgrade.on_upgrade(move |socket| {
        run_connection(socket, agent_id, registry, broadcaster, fleet, dal)
    }))
}

async fn run_connection(
    socket: WebSocket,
    agent_id: uuid::Uuid,
    registry: Arc<ConnectionRegistry>,
    broadcaster: Arc<LiveBroadcaster>,
    fleet: Arc<FleetBroadcaster>,
    dal: DAL,
) {
    let connected_since = Utc::now();
    let messages_in = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let messages_out = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let (control_tx, control_rx) = mpsc::channel::<WsMessage>(CONTROL_LANE_CAPACITY);
    let (telemetry_tx, telemetry_rx) = mpsc::channel::<WsMessage>(TELEMETRY_LANE_CAPACITY);

    registry.register(ConnectionHandle {
        agent_id,
        connected_since,
        messages_in: messages_in.clone(),
        messages_out: messages_out.clone(),
        control_tx,
        telemetry_tx,
    });
    metrics::ws_connected_agents().inc();

    // BROKKR-I-0028: event-driven fleet live-push on connect. Best-effort —
    // a push failure must never affect the connection lifecycle. The record
    // now reflects ws_connected=true via the registry snapshot.
    crate::api::v1::fleet::broadcast_agent_fleet_update(&dal, &registry, &fleet, agent_id);

    let (sender, receiver) = socket.split();

    let writer_messages_out = messages_out.clone();
    let writer = tokio::spawn(writer_task(
        sender,
        control_rx,
        telemetry_rx,
        writer_messages_out,
    ));
    // Keep a handle to the DAL for the post-disconnect fleet broadcast below;
    // the reader task takes its own clone (DAL clones are cheap pool handles).
    let disconnect_dal = dal.clone();
    let reader = tokio::spawn(reader_task(
        receiver,
        agent_id,
        messages_in,
        dal,
        broadcaster,
    ));

    // First task to finish wins; the other is aborted so we don't leak a
    // socket-half task after the peer is gone.
    tokio::select! {
        _ = writer => debug!(%agent_id, "writer task exited"),
        _ = reader => debug!(%agent_id, "reader task exited"),
    }

    registry.unregister_if_matches(agent_id, connected_since);
    metrics::ws_connected_agents().dec();

    // BROKKR-I-0028: event-driven fleet live-push on disconnect. Best-effort;
    // the record now reflects ws_connected=false (registry entry removed).
    crate::api::v1::fleet::broadcast_agent_fleet_update(
        &disconnect_dal,
        &registry,
        &fleet,
        agent_id,
    );

    info!(%agent_id, "agent WS connection closed");
}

async fn reader_task(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    agent_id: uuid::Uuid,
    messages_in: Arc<std::sync::atomic::AtomicU64>,
    dal: DAL,
    broadcaster: Arc<LiveBroadcaster>,
) {
    while let Some(frame) = receiver.next().await {
        match frame {
            Ok(Message::Text(text)) => match serde_json::from_str::<WsMessage>(&text) {
                Ok(msg) => {
                    messages_in.fetch_add(1, Ordering::Relaxed);
                    dispatch_uplink(msg, agent_id, &dal, &broadcaster);
                }
                Err(e) => {
                    warn!(%agent_id, error = %e, "dropping undecodable WS frame");
                }
            },
            Ok(Message::Binary(_)) => {
                warn!(%agent_id, "binary WS frame received; only text frames are supported in v1");
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                // axum responds to pings automatically; nothing to do.
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                debug!(%agent_id, error = %e, "WS read error; closing connection");
                break;
            }
        }
    }
}

/// Dispatch an inbound WS message into the same DAL operations the REST
/// handlers would perform. Frames the agent has no legitimate reason to
/// send upstream (broker→agent control plane, future fan-out telemetry)
/// are dropped with a warning. The connection's authenticated `agent_id`
/// is used; any mismatched value in the message body is ignored.
fn dispatch_uplink(msg: WsMessage, agent_id: uuid::Uuid, dal: &DAL, broadcaster: &LiveBroadcaster) {
    metrics::ws_messages_total("in", ws_variant_name(&msg)).inc();
    match msg {
        WsMessage::Heartbeat(hb) => {
            if let Err(e) = dal.agents().record_heartbeat(agent_id) {
                warn!(%agent_id, error = %e, "failed to record WS heartbeat");
            }
            // BROKKR-T-0227: persist agent-reported K8s connectivity when the
            // heartbeat carries it. Absent fields leave the columns untouched.
            if let Some(reachable) = hb.k8s_reachable
                && let Err(e) =
                    dal.agents()
                        .record_k8s_connectivity(agent_id, reachable, hb.k8s_api_latency_ms)
            {
                warn!(%agent_id, error = %e, "failed to record WS K8s connectivity");
            }
        }
        WsMessage::AgentEvent(ev) => {
            if ev.agent_id != agent_id {
                warn!(
                    %agent_id, body_agent = %ev.agent_id,
                    "dropping WS AgentEvent whose body agent_id does not match the connection"
                );
                return;
            }
            let new_event = NewAgentEvent {
                agent_id,
                deployment_object_id: ev.deployment_object_id,
                event_type: ev.event_type,
                status: ev.status,
                message: ev.message,
            };
            if let Err(e) = dal.agent_events().create(&new_event) {
                warn!(%agent_id, error = %e, "failed to persist WS AgentEvent");
            }
        }
        WsMessage::AgentHealth(h) => {
            if h.agent_id != agent_id {
                warn!(
                    %agent_id, body_agent = %h.agent_id,
                    "dropping WS AgentHealth whose body agent_id does not match the connection"
                );
                return;
            }
            let new_health = NewDeploymentHealth {
                agent_id,
                deployment_object_id: h.deployment_object_id,
                status: h.status,
                summary: h.summary,
                checked_at: h.checked_at,
            };
            if let Err(e) = dal.deployment_health().upsert(&new_health) {
                warn!(%agent_id, error = %e, "failed to upsert WS DeploymentHealth");
            }
        }
        WsMessage::K8sEvent(ev) => {
            if ev.agent_id != agent_id {
                warn!(%agent_id, body_agent = %ev.agent_id, "dropping K8sEvent with mismatched agent_id");
                return;
            }
            let stack_id = ev.stack_id;
            // Live fan-out *before* persistence so subscribers see the
            // frame at ingest latency; persist also so the REST history
            // endpoint (WS-10) returns it later.
            broadcaster.broadcast(stack_id, WsMessage::K8sEvent(ev.clone()));
            let involved = match serde_json::to_value(&ev.involved_object) {
                Ok(v) => v,
                Err(e) => {
                    warn!(%agent_id, error = %e, "failed to encode K8sEvent involved_object");
                    return;
                }
            };
            let new = NewAgentK8sEvent {
                agent_id,
                stack_id,
                observed_at: ev.observed_at,
                reason: ev.reason,
                message: ev.message,
                event_type: ev.event_type,
                source: ev.source,
                involved_object: involved,
            };
            if let Err(e) = dal.agent_k8s_events().create(&new) {
                warn!(%agent_id, error = %e, "failed to persist K8sEvent");
            }
        }
        WsMessage::PodLogLine(line) => {
            if line.agent_id != agent_id {
                warn!(%agent_id, body_agent = %line.agent_id, "dropping PodLogLine with mismatched agent_id");
                return;
            }
            let stack_id = line.stack_id;
            broadcaster.broadcast(stack_id, WsMessage::PodLogLine(line.clone()));
            let new = NewAgentPodLog {
                agent_id,
                stack_id,
                namespace: line.namespace,
                pod: line.pod,
                container: line.container,
                ts: line.ts,
                line: line.line,
            };
            if let Err(e) = dal.agent_pod_logs().create(&new) {
                warn!(%agent_id, error = %e, "failed to persist PodLogLine");
            }
        }
        WsMessage::LogGap(gap) => {
            // Gap markers are pure metadata — broadcast for live
            // subscribers but don't persist (per WS-09 decision).
            broadcaster.broadcast(gap.stack_id, WsMessage::LogGap(gap));
        }
        // Anything else is broker → agent / broker → consumer shape; an agent
        // should never send these on the uplink.
        WsMessage::WorkOrder(_)
        | WsMessage::TargetChanged(_)
        | WsMessage::StackChanged(_)
        | WsMessage::FleetUpdate(_) => {
            warn!(
                %agent_id,
                "dropping broker→agent message variant received on agent uplink"
            );
        }
    }
}

/// Snake_case tag matching the wire enum's serde rename. Kept in sync
/// by hand — same source-of-truth as the golden fixture in
/// `brokkr-wire`.
fn ws_variant_name(msg: &WsMessage) -> &'static str {
    match msg {
        WsMessage::WorkOrder(_) => "work_order",
        WsMessage::TargetChanged(_) => "target_changed",
        WsMessage::StackChanged(_) => "stack_changed",
        WsMessage::Heartbeat(_) => "heartbeat",
        WsMessage::AgentEvent(_) => "agent_event",
        WsMessage::AgentHealth(_) => "agent_health",
        WsMessage::K8sEvent(_) => "k8s_event",
        WsMessage::PodLogLine(_) => "pod_log_line",
        WsMessage::LogGap(_) => "log_gap",
        WsMessage::FleetUpdate(_) => "fleet_update",
    }
}

async fn writer_task(
    mut sender: futures::stream::SplitSink<WebSocket, Message>,
    mut control_rx: mpsc::Receiver<WsMessage>,
    mut telemetry_rx: mpsc::Receiver<WsMessage>,
    messages_out: Arc<std::sync::atomic::AtomicU64>,
) {
    loop {
        // `biased` makes `tokio::select!` poll branches top-down, giving
        // the control lane strict priority over telemetry. This is the
        // ADR's "control plane is never starved" guarantee.
        let next = tokio::select! {
            biased;
            msg = control_rx.recv() => msg,
            msg = telemetry_rx.recv() => msg,
        };

        let Some(msg) = next else {
            // Both senders dropped — registry no longer holds this
            // connection. Close the socket and exit.
            let _ = sender.close().await;
            return;
        };

        let text = match serde_json::to_string(&msg) {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "failed to serialize outbound WS message; dropping");
                continue;
            }
        };

        let variant = ws_variant_name(&msg);
        if let Err(e) = sender.send(Message::Text(text)).await {
            debug!(error = %e, "WS send failed; closing connection");
            return;
        }
        messages_out.fetch_add(1, Ordering::Relaxed);
        metrics::ws_messages_total("out", variant).inc();
    }
}
