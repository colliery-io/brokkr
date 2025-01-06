//! Handles API routes and logic for agent events.
//!
//! This module provides functionality to list and retrieve agent events
//! through HTTP endpoints.

use crate::dal::DAL;
use axum::{
    extract::{Extension, Path, State},
    routing::get,
    Json, Router,
};
use brokkr_models::models::agent_events::AgentEvent;
use brokkr_utils::logging::prelude::*;
use uuid::Uuid;

/// Creates and returns a router for agent event-related endpoints.
pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/agent-events", get(list_agent_events))
        .route("/agent-events/:id", get(get_agent_event))
}

#[utoipa::path(
    get,
    path = "/api/v1/agent-events",
    responses(
        (status = 200, description = "List all agent events", body = Vec<AgentEvent>),
        (status = 500, description = "Internal server error")
    ),
    tag = "agent-events"
)]
/// Retrieves a list of all agent events.
///
/// # Arguments
/// * `State(dal)` - The data access layer state.
/// * `Extension(_auth_payload)` - Authentication payload (unused but required).
///
/// # Returns
/// A JSON response containing a vector of AgentEvents or an error.
async fn list_agent_events(
    State(dal): State<DAL>,
    Extension(_auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
) -> Result<Json<Vec<AgentEvent>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list agent events");
    match dal.agent_events().list() {
        Ok(events) => {
            info!("Successfully retrieved {} agent events", events.len());
            Ok(Json(events))
        }
        Err(e) => {
            error!("Failed to fetch agent events: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent events"})),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/agent-events/{id}",
    responses(
        (status = 200, description = "Get agent event by id", body = AgentEvent),
        (status = 404, description = "Agent event not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Agent event id")
    ),
    tag = "agent-events"
)]
/// Retrieves a specific agent event by its ID.
///
/// # Arguments
/// * `State(dal)` - The data access layer state.
/// * `Extension(_auth_payload)` - Authentication payload (unused but required).
/// * `Path(id)` - The UUID of the agent event to retrieve.
///
/// # Returns
/// A JSON response containing the requested AgentEvent or an error.
async fn get_agent_event(
    State(dal): State<DAL>,
    Extension(_auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentEvent>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get agent event with ID: {}", id);
    match dal.agent_events().get(id) {
        Ok(Some(event)) => {
            info!("Successfully retrieved agent event with ID: {}", id);
            Ok(Json(event))
        }
        Ok(None) => {
            warn!("Agent event with ID {} not found", id);
            Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Agent event not found"})),
            ))
        }
        Err(e) => {
            error!("Error fetching agent event with ID {}: {:?}", id, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent event"})),
            ))
        }
    }
}
