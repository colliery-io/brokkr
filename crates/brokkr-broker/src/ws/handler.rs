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

use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use brokkr_wire::WsMessage;

use crate::api::v1::middleware::{self as v1_middleware, AuthPayload};
use crate::dal::DAL;

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
pub fn internal_routes(dal: DAL, registry: Arc<ConnectionRegistry>) -> Router<DAL> {
    Router::new()
        .route(INTERNAL_WS_PATH, get(ws_upgrade))
        .layer(Extension(registry))
        .layer(from_fn_with_state(
            dal,
            v1_middleware::auth_middleware::<Body>,
        ))
}

async fn ws_upgrade(
    upgrade: WebSocketUpgrade,
    Extension(registry): Extension<Arc<ConnectionRegistry>>,
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
    Ok(upgrade.on_upgrade(move |socket| run_connection(socket, agent_id, registry)))
}

async fn run_connection(
    socket: WebSocket,
    agent_id: uuid::Uuid,
    registry: Arc<ConnectionRegistry>,
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

    let (sender, receiver) = socket.split();

    let writer_messages_out = messages_out.clone();
    let writer = tokio::spawn(writer_task(sender, control_rx, telemetry_rx, writer_messages_out));
    let reader = tokio::spawn(reader_task(receiver, agent_id, messages_in));

    // First task to finish wins; the other is aborted so we don't leak a
    // socket-half task after the peer is gone.
    tokio::select! {
        _ = writer => debug!(%agent_id, "writer task exited"),
        _ = reader => debug!(%agent_id, "reader task exited"),
    }

    registry.unregister_if_matches(agent_id, connected_since);
    info!(%agent_id, "agent WS connection closed");
}

async fn reader_task(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    agent_id: uuid::Uuid,
    messages_in: Arc<std::sync::atomic::AtomicU64>,
) {
    while let Some(frame) = receiver.next().await {
        match frame {
            Ok(Message::Text(text)) => match serde_json::from_str::<WsMessage>(&text) {
                Ok(_msg) => {
                    messages_in.fetch_add(1, Ordering::Relaxed);
                    // Dispatch for uplink (heartbeat / events / health / kube
                    // events / log lines) lands in WS-05 / WS-09; for WS-02
                    // we only need to count and validate that the frame
                    // round-trips through `brokkr-wire`.
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

        if let Err(e) = sender.send(Message::Text(text)).await {
            debug!(error = %e, "WS send failed; closing connection");
            return;
        }
        messages_out.fetch_add(1, Ordering::Relaxed);
    }
}
