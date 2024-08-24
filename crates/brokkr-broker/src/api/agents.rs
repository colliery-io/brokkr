//! This module provides the API endpoints for managing Agent entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting agents,
//! as well as listing agents and updating their status and heartbeat.

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
    include_deleted: Option<bool>,
}

/// Configures the agents API routes.
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
async fn create_agent(
    State(state): State<AppState>,
    Json(new_agent): Json<NewAgent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    state.dal.agents().create(&new_agent)
        .map(|agent| (StatusCode::CREATED, Json(agent)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for retrieving an agent by UUID.
async fn get_agent(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().get(uuid,false)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Handler for soft-deleting an agent.
async fn soft_delete_agent(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> StatusCode {
    state.dal.agents().soft_delete(uuid)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all agents.
async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<ListAgentsQuery>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    state.dal.agents().list(params.include_deleted.unwrap_or(false))
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for updating an agent.
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
async fn update_heartbeat(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().update_heartbeat(uuid)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for updating an agent's status.
async fn update_status(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(status): Json<String>,
) -> Result<Json<Agent>, StatusCode> {
    state.dal.agents().update_status(uuid, &status)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}