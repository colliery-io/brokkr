/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Read-only WS subscription endpoint for live agent telemetry tail.
//!
//! Mounted under `/api/v1/stacks/{id}/live` so it inherits the standard
//! v1 PAK middleware. The handler then re-applies the same authorisation
//! the REST history endpoints use (admin OR owning generator) before
//! upgrading the connection.
//!
//! Once upgraded, the connection broadcasts every frame the
//! [`LiveBroadcaster`] receives for that stack. When the subscriber lags
//! past the broadcast capacity, `RecvError::Lagged(n)` is converted to a
//! synthetic `LogGap` message so the UI can render a visible gap
//! (per ADR-0008's "a slow subscriber must not slow ingestion").

use std::sync::Arc;

use axum::{
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Extension, Path, State},
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use brokkr_wire::{GapReason, LogGap, WsMessage};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::api::v1::middleware::{self as v1_middleware, AuthPayload};
use crate::dal::DAL;

use super::broadcaster::LiveBroadcaster;

/// Documented path template (Axum colon-style). The realised path is
/// `/api/v1/stacks/{id}/live` after the parent router mounts this branch.
pub const LIVE_SUBSCRIPTION_PATH_TEMPLATE: &str = "/api/v1/stacks/:id/live";

/// Build the live-subscription router. Mounted under `/api/v1` by the
/// parent so the standard PAK middleware applies; this fn adds its own
/// auth-middleware layer too in case the router is mounted elsewhere.
pub fn subscribe_routes(dal: DAL, broadcaster: Arc<LiveBroadcaster>) -> Router<DAL> {
    Router::new()
        .route("/api/v1/stacks/:id/live", get(live_upgrade))
        .layer(Extension(broadcaster))
        .layer(from_fn_with_state(
            dal,
            v1_middleware::auth_middleware::<Body>,
        ))
}

async fn live_upgrade(
    upgrade: WebSocketUpgrade,
    State(dal): State<DAL>,
    Extension(broadcaster): Extension<Arc<LiveBroadcaster>>,
    Path(stack_id): Path<Uuid>,
    request: Request<Body>,
) -> Result<axum::response::Response, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthPayload>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Reuse the existing scoping rules: admin sees any stack; generator
    // sees their own. Agents do not get live tail here in v1 — they have
    // the bidi internal channel for their own work.
    if !authorise(&dal, &auth, stack_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(upgrade.on_upgrade(move |socket| run_subscriber(socket, stack_id, broadcaster)))
}

fn authorise(dal: &DAL, auth: &AuthPayload, stack_id: Uuid) -> bool {
    if auth.admin {
        return true;
    }
    let Some(generator_id) = auth.generator else {
        return false;
    };
    match dal.stacks().get(vec![stack_id]) {
        Ok(stacks) => stacks
            .first()
            .map(|s| s.generator_id == generator_id)
            .unwrap_or(false),
        Err(_) => false,
    }
}

async fn run_subscriber(
    socket: WebSocket,
    stack_id: Uuid,
    broadcaster: Arc<LiveBroadcaster>,
) {
    let mut rx = broadcaster.subscribe(stack_id);
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            frame = rx.recv() => match frame {
                Ok(msg) => {
                    if !forward(&mut sink, &msg).await {
                        return;
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    let gap = WsMessage::LogGap(LogGap {
                        agent_id: Uuid::nil(),
                        stack_id,
                        since_ts: Utc::now(),
                        dropped_count: n,
                        reason: GapReason::BufferFull,
                    });
                    if !forward(&mut sink, &gap).await {
                        return;
                    }
                }
                Err(RecvError::Closed) => {
                    debug!(%stack_id, "broadcast channel closed; ending subscription");
                    return;
                }
            },
            // Detect client-side close cleanly.
            msg = futures::StreamExt::next(&mut stream) => match msg {
                Some(Ok(Message::Close(_))) | None => return,
                Some(Err(e)) => {
                    debug!(error = %e, "subscription stream error");
                    return;
                }
                Some(Ok(_)) => {
                    // Read-only subscription — ignore any frames the
                    // client tries to send back at us.
                }
            }
        }
    }
}

async fn forward(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &WsMessage,
) -> bool {
    match serde_json::to_string(msg) {
        Ok(text) => match sink.send(Message::Text(text)).await {
            Ok(()) => true,
            Err(e) => {
                debug!(error = %e, "subscriber send failed; closing");
                false
            }
        },
        Err(e) => {
            warn!(error = %e, "failed to serialise live frame; dropping");
            true
        }
    }
}
