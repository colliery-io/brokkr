/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Consumer-facing WS subscription endpoint for the live fleet stream
//! (BROKKR-I-0028).
//!
//! Mounted under `/api/v1/fleet/live` so it inherits the standard v1 PAK
//! middleware. The handler then re-applies the **admin** check that
//! `GET /fleet` uses (`AuthPayload.admin`) — simpler than the stack live-tail
//! handler's admin-or-owner rule, because a fleet record spans all agents and
//! is admin-only.
//!
//! Once upgraded, the connection forwards every [`WsMessage::FleetUpdate`] the
//! [`FleetBroadcaster`] receives. When the subscriber lags past the broadcast
//! capacity, `RecvError::Lagged(n)` is handled by simply dropping and
//! continuing (per ADR-0008's "a slow subscriber must not slow ingestion").
//! There is no per-agent gap concept: the consumer holds the latest record per
//! `agent_id`, so a missed update is superseded by the next one for that agent.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Extension, State},
    http::header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL},
    http::{HeaderValue, Request, StatusCode},
    middleware::{Next, from_fn, from_fn_with_state},
    response::Response,
    routing::get,
};
use brokkr_wire::WsMessage;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

use crate::api::v1::middleware::{self as v1_middleware, AuthPayload};
use crate::dal::DAL;
use crate::metrics;

use super::broadcaster::FleetBroadcaster;

/// Documented path template (Axum colon-style). The realised path is
/// `/api/v1/fleet/live` after the parent router mounts this branch.
pub const FLEET_LIVE_SUBSCRIPTION_PATH_TEMPLATE: &str = "/api/v1/fleet/live";

/// Subprotocol that carries the PAK for browser clients that cannot set an
/// `Authorization` header on `new WebSocket()` (mirrors `ws/subscribe.rs`).
const PAK_SUBPROTOCOL_PREFIX: &str = "brokkr.pak.";

/// Non-secret marker subprotocol the browser also offers and the broker echoes
/// back in the handshake response (never the PAK-bearing one).
const WS_MARKER_SUBPROTOCOL: &str = "brokkr.v1";

/// Build the fleet-live-subscription router. Mounted under `/api/v1` by the
/// parent so the standard PAK middleware applies; this fn adds its own
/// auth-middleware layer too in case the router is mounted elsewhere.
pub fn fleet_subscribe_routes(dal: DAL, broadcaster: Arc<FleetBroadcaster>) -> Router<DAL> {
    Router::new()
        .route("/api/v1/fleet/live", get(fleet_live_upgrade))
        .layer(Extension(broadcaster))
        .layer(from_fn_with_state(
            dal,
            v1_middleware::auth_middleware::<Body>,
        ))
        // Outermost: supply the Authorization header from the WS subprotocol
        // for browser clients before auth_middleware runs.
        .layer(from_fn(ws_subprotocol_auth))
}

/// Browser WS clients can't set request headers, so they pass the PAK in
/// `Sec-WebSocket-Protocol` as `brokkr.pak.<PAK>`. This lifts it into an
/// `Authorization: Bearer` header **only when one isn't already present**, so
/// header-based callers are unaffected (mirrors `ws/subscribe.rs`).
async fn ws_subprotocol_auth(mut request: Request<Body>, next: Next) -> Response {
    if request.headers().get(AUTHORIZATION).is_none()
        && let Some(pak) = request
            .headers()
            .get(SEC_WEBSOCKET_PROTOCOL)
            .and_then(|v| v.to_str().ok())
            .and_then(|list| {
                list.split(',')
                    .map(str::trim)
                    .find_map(|p| p.strip_prefix(PAK_SUBPROTOCOL_PREFIX))
            })
        && let Ok(value) = HeaderValue::from_str(&format!("Bearer {pak}"))
    {
        request.headers_mut().insert(AUTHORIZATION, value);
    }
    next.run(request).await
}

async fn fleet_live_upgrade(
    upgrade: WebSocketUpgrade,
    State(_dal): State<DAL>,
    Extension(broadcaster): Extension<Arc<FleetBroadcaster>>,
    request: Request<Body>,
) -> Result<axum::response::Response, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthPayload>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Admin-gated, exactly like GET /fleet. A fleet record spans all agents;
    // there is no per-owner scoping here.
    if !auth.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    // Echo only the non-secret marker subprotocol when the client offered it.
    Ok(upgrade
        .protocols([WS_MARKER_SUBPROTOCOL])
        .on_upgrade(move |socket| run_fleet_subscriber(socket, broadcaster)))
}

async fn run_fleet_subscriber(socket: WebSocket, broadcaster: Arc<FleetBroadcaster>) {
    let mut rx = broadcaster.subscribe();
    metrics::fleet_live_subscribers().inc();
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            frame = rx.recv() => match frame {
                Ok(msg) => {
                    if !forward(&mut sink, &msg).await {
                        break;
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    // No per-agent gap concept — the consumer replaces by
                    // agent_id, so a dropped update is superseded by the next.
                    // Just continue (ADR-0008: never block ingestion).
                    debug!(dropped = n, "fleet-live subscriber lagged; continuing");
                }
                Err(RecvError::Closed) => {
                    debug!("fleet broadcast channel closed; ending subscription");
                    break;
                }
            },
            // Detect client-side close cleanly.
            msg = futures::StreamExt::next(&mut stream) => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(e)) => {
                    debug!(error = %e, "fleet-live subscription stream error");
                    break;
                }
                Some(Ok(_)) => {
                    // Read-only subscription — ignore anything the client sends.
                }
            }
        }
    }

    metrics::fleet_live_subscribers().dec();
}

async fn forward(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &WsMessage,
) -> bool {
    match serde_json::to_string(msg) {
        Ok(text) => match sink.send(Message::Text(text)).await {
            Ok(()) => true,
            Err(e) => {
                debug!(error = %e, "fleet-live subscriber send failed; closing");
                false
            }
        },
        Err(e) => {
            warn!(error = %e, "failed to serialise fleet-live frame; dropping");
            true
        }
    }
}
