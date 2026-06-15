/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Handles API routes and logic for agent events.
//!
//! This module provides functionality to list and retrieve agent events
//! through HTTP endpoints.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::dal::DAL;
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::get,
};
use brokkr_models::models::agent_events::AgentEvent;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Creates and returns a router for agent event-related endpoints.
pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/agent-events", get(list_agent_events))
        .route("/agent-events/:id", get(get_agent_event))
}

#[utoipa::path(
    get,
    path = "/agent-events",
    responses(
        (status = 200, description = "List all agent events", body = Vec<AgentEvent>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "agent-events",
    security(
        ("admin_pak" = []),
    )
)]
async fn list_agent_events(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
) -> Result<Json<Vec<AgentEvent>>, ApiError> {
    info!("Handling request to list agent events");
    // This is a cluster-wide list; only admin may enumerate every agent's
    // events. An agent reads its own via GET /agents/{id}/events.
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }
    let events = dal.agent_events().list().map_err(|e| {
        error!("Failed to fetch agent events: {:?}", e);
        ApiError::internal("failed to fetch agent events")
    })?;
    info!("Successfully retrieved {} agent events", events.len());
    Ok(Json(events))
}

#[utoipa::path(
    get,
    path = "/agent-events/{id}",
    responses(
        (status = 200, description = "Get agent event by id", body = AgentEvent),
        (status = 404, description = "Agent event not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Agent event id")
    ),
    tag = "agent-events",
    security(
        ("admin_pak" = []),
    )
)]
async fn get_agent_event(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentEvent>, ApiError> {
    info!("Handling request to get agent event with ID: {}", id);
    if !auth_payload.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }
    let event = dal
        .agent_events()
        .get(id)
        .map_err(|e| {
            error!("Error fetching agent event with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch agent event")
        })?
        .ok_or_else(|| {
            warn!("Agent event with ID {} not found", id);
            ApiError::not_found("agent_event_not_found", "agent event not found")
        })?;
    info!("Successfully retrieved agent event with ID: {}", id);
    Ok(Json(event))
}
