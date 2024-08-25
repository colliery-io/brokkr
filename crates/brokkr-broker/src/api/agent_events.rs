//! # Agent Events API Module
//!
//! This module provides the API endpoints for managing AgentEvent entities using Axum.
//!
//! It includes routes for creating, retrieving, listing, and soft-deleting agent events.
//! The module uses the Axum web framework and interacts with a data access layer (DAL)
//! to perform operations on AgentEvent entities.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    Router,
    routing::{get, post, delete},
};
use serde::Deserialize;
use uuid::Uuid;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use crate::api::AppState;

/// Query parameters for listing agent events
#[derive(Deserialize)]
pub struct ListAgentEventsQuery {
    /// Optional UUID of the stack to filter events
    stack_id: Option<Uuid>,
    /// Optional UUID of the agent to filter events
    agent_id: Option<Uuid>,
}

/// Configures the agent events API routes.
///
/// This function sets up the following routes:
/// - POST /agent-events: Create a new agent event
/// - GET /agent-events: List agent events
/// - GET /agent-events/:uuid: Get a specific agent event
/// - DELETE /agent-events/:uuid: Soft delete an agent event
///
/// # Returns
/// A configured `Router<AppState>` with all agent event routes.
pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/agent-events", post(create_agent_event))
        .route("/agent-events", get(list_agent_events))
        .route("/agent-events/:uuid", get(get_agent_event))
        .route("/agent-events/:uuid", delete(soft_delete_agent_event))
}

/// Handler for creating a new agent event.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `new_event` - JSON payload containing the new event data
///
/// # Returns
/// * On success: A tuple containing `StatusCode::CREATED` and the created `AgentEvent`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`
async fn create_agent_event(
    State(state): State<AppState>,
    Json(new_event): Json<NewAgentEvent>,
) -> Result<(StatusCode, Json<AgentEvent>), StatusCode> {
    state.dal.agent_events().create(&new_event)
        .map(|event| (StatusCode::CREATED, Json(event)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for retrieving an agent event by UUID.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent event to retrieve
///
/// # Returns
/// * On success: JSON representation of the `AgentEvent`
/// * On not found: `StatusCode::NOT_FOUND`
/// * On other errors: `StatusCode::INTERNAL_SERVER_ERROR`
async fn get_agent_event(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<AgentEvent>, StatusCode> {
    state.dal.agent_events().get(uuid)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Handler for soft-deleting an agent event.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent event to soft delete
///
/// # Returns
/// * On success: `StatusCode::NO_CONTENT`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`
async fn soft_delete_agent_event(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> StatusCode {
    state.dal.agent_events().soft_delete(uuid)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing agent events.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `params` - Query parameters for filtering events
///
/// # Returns
/// * On success: JSON array of `AgentEvent` objects
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`
async fn list_agent_events(
    State(state): State<AppState>,
    Query(params): Query<ListAgentEventsQuery>,
) -> Result<Json<Vec<AgentEvent>>, StatusCode> {
    state.dal.agent_events().get_events(params.stack_id, params.agent_id)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}