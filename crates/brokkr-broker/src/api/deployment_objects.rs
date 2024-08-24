//! This module provides the API endpoints for managing DeploymentObject entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting deployment objects,
//! as well as listing deployment objects for a specific stack.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    Router,
    routing::{get, post, put, delete},
};
use uuid::Uuid;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use crate::api::AppState;

/// Configures the deployment objects API routes.
pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/deployment-objects", post(create_deployment_object))
        .route("/deployment-objects/:uuid", get(get_deployment_object))
        .route("/deployment-objects/:uuid", put(update_deployment_object))
        .route("/deployment-objects/:uuid", delete(soft_delete_deployment_object))
        .route("/stacks/:stack_uuid/deployment-objects", get(list_deployment_objects))
        .route("/deployment-objects/active", get(list_active_deployment_objects))
}

/// Handler for creating a new deployment object.
async fn create_deployment_object(
    State(state): State<AppState>,
    Json(new_deployment_object): Json<NewDeploymentObject>,
) -> Result<(StatusCode, Json<DeploymentObject>), StatusCode> {
    state.dal.deployment_objects().create(&new_deployment_object)
        .map(|object| (StatusCode::CREATED, Json(object)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for retrieving a deployment object by UUID.
async fn get_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<DeploymentObject>, StatusCode> {
    state.dal.deployment_objects().get_by_id(uuid)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Handler for updating a deployment object.
async fn update_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(deployment_object): Json<DeploymentObject>,
) -> Result<Json<DeploymentObject>, StatusCode> {
    state.dal.deployment_objects().update(uuid, &deployment_object)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for soft-deleting a deployment object.
async fn soft_delete_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> StatusCode {
    state.dal.deployment_objects().soft_delete(uuid)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all deployment objects for a specific stack.
async fn list_deployment_objects(
    State(state): State<AppState>,
    Path(stack_uuid): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, StatusCode> {
    state.dal.deployment_objects().get_by_stack_id(stack_uuid)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all active deployment objects.
async fn list_active_deployment_objects(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeploymentObject>>, StatusCode> {
    state.dal.deployment_objects().get_active()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}