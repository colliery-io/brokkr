//! # Agents API Module
//!
//! This module provides the API endpoints for managing Agent entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting agents,
//! as well as listing agents and updating their status and heartbeat. The module
//! uses the Axum web framework and interacts with a data access layer (DAL)
//! to perform operations on Agent entities.

use crate::api::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::agents::{Agent, NewAgent};
use prefixed_api_key::PrefixedApiKeyController;
use serde::Deserialize;
use uuid::Uuid;

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
        .route("/agents/:uuid/generate_api_key", post(generate_api_key))
        // .route("/agents/:uuid/undeployed", get(get_undeployed_objects))
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
    state
        .dal
        .agents()
        .create(&new_agent)
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
    state
        .dal
        .agents()
        .get(uuid, false)
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

async fn soft_delete_agent(State(state): State<AppState>, Path(uuid): Path<Uuid>) -> StatusCode {
    state
        .dal
        .agents()
        .soft_delete(uuid)
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
    state
        .dal
        .agents()
        .list(params.include_deleted.unwrap_or(false))
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
    state
        .dal
        .agents()
        .update(uuid, &agent)
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
    state
        .dal
        .agents()
        .update_heartbeat(uuid)
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
    state
        .dal
        .agents()
        .update_status(uuid, &status)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for generating an API key for an agent.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the agent to generate an API key for
///
/// # Returns
/// * On success: JSON object containing the generated API key
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn generate_api_key(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let agent = state
        .dal
        .agents()
        .get(uuid, false)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let prefix = format!("brokkr+{}+{}", agent.name, agent.cluster_name);
    
    let controller = PrefixedApiKeyController::configure()
        .prefix(prefix.to_owned())
        .seam_defaults()
        .finalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (pak, hash) = controller
        .try_generate_key_and_hash()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update the agent with the new API key hash
    let updated_agent = Agent {
        pak_hash: hash,
        ..agent
    };

    state
        .dal
        .agents()
        .update(uuid, &updated_agent)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "api_key": pak.to_string()
    })))
}


// async fn get_undeployed_objects(
//     State(state): State<AppState>,
//     Path(agent_id): Path<Uuid>,
// ) -> Result<Json<Vec<(Stack, Vec<DeploymentObject>)>>, StatusCode> {
//     state
//         .dal
//         .get_undeployed_objects_for_agent(agent_id)
//         .map(Json)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
// }