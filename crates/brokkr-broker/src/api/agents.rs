//! # Agents API Module
//!
//! This module provides the API endpoints for managing Agent entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting agents,
//! as well as listing agents and updating their status and heartbeat. The module
//! uses the Axum web framework and interacts with a data access layer (DAL)
//! to perform operations on Agent entities.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    Router,
    routing::{get, post, put, delete},
};
use serde::Deserialize;
use uuid::Uuid;
use brokkr_models::models::agents::{Agent, NewAgent};
use crate::api::AppState;

/// Query parameters for listing agents
#[derive(Deserialize)]
pub struct ListAgentsQuery {

    /// Optional flag to include soft-deleted agents in the list
    include_deleted: Option<bool>,
}

/// Configures the agents API routes.
///
/// This function sets up the following routes:
/// - POST /agents: Create a new agent
/// - GET /agents: List all agents
/// - GET /agents/:uuid: Get a specific agent
/// - PUT /agents/:uuid: Update an agent
/// - DELETE /agents/:uuid: Soft delete an agent
/// - PUT /agents/:uuid/heartbeat: Update an agent's heartbeat
/// - PUT /agents/:uuid/status: Update an agent's status
///
/// # Returns
/// A configured `Router<AppState>` with all agent routes.

pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/agents", post(create_agent))
        .route("/agents", get(list_agents))
        .route("/agents/:uuid", get(get_agent))
        .route("/agents/:uuid", put(update_agent))
        .route("/agents/:uuid", delete(soft_delete_agent))
        .route("/agents/:uuid/heartbeat", put(update_heartbeat))
        .route("/agents/:uuid/status", put(update_status))
}

/// Handler for creating a new agent.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `new_agent` - JSON payload containing the new agent data
///
/// # Returns
/// * On success: A tuple containing `StatusCode::CREATED` and the created `Agent`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn create_agent(
    State(state): State<AppState>,
    Json(new_agent): Json<NewAgent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    state.dal.agents().create(&new_agent)
        .map(|agent| (StatusCode::CREATED, Json(agent)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for retrieving an agent by UUID.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to retrieve
///
/// # Returns
/// * On success: JSON representation of the `Agent`
/// * On not found: `StatusCode::NOT_FOUND`

async fn get_agent(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<Agent>, StatusCode> {

    state.dal.agents().get(uuid,false)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Handler for soft-deleting an agent.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to soft delete
///
/// # Returns
/// * On success: `StatusCode::NO_CONTENT`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn soft_delete_agent(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> StatusCode {
    state.dal.agents().soft_delete(uuid)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all agents.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `params` - Query parameters for filtering agents
///
/// # Returns
/// * On success: JSON array of `Agent` objects
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<ListAgentsQuery>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    state.dal.agents().list(params.include_deleted.unwrap_or(false))
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for updating an agent.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to update
/// * `agent` - JSON payload containing the updated agent data
///
/// # Returns
/// * On success: JSON representation of the updated `Agent`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn update_agent(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(agent): Json<Agent>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().update(uuid, &agent)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for updating an agent's heartbeat.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to update
///
/// # Returns
/// * On success: JSON representation of the updated `Agent`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn update_heartbeat(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().update_heartbeat(uuid)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for updating an agent's status.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to update
/// * `status` - JSON payload containing the new status
///
/// # Returns
/// * On success: JSON representation of the updated `Agent`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn update_status(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(status): Json<String>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().update_status(uuid, &status)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}