//! # Deployment Objects API Module
//!
//! This module provides the API endpoints for managing DeploymentObject entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting deployment objects,
//! as well as listing deployment objects for a specific stack and listing active deployment objects.
//! The module uses the Axum web framework and interacts with a data access layer (DAL)
//! to perform operations on DeploymentObject entities.

use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use uuid::Uuid;

/// Configures the deployment objects API routes.
///
/// This function sets up the following routes:
/// - POST /deployment-objects: Create a new deployment object
/// - GET /deployment-objects/:uuid: Get a specific deployment object
/// - PUT /deployment-objects/:uuid: Update a deployment object
/// - DELETE /deployment-objects/:uuid: Soft delete a deployment object
/// - GET /stacks/:stack_uuid/deployment-objects: List all deployment objects for a specific stack
/// - GET /deployment-objects/active: List all active deployment objects
///
/// # Returns
/// A configured `Router<AppState>` with all deployment object routes.

pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/deployment-objects", post(create_deployment_object))
        .route("/deployment-objects/:uuid", get(get_deployment_object))
        .route("/deployment-objects/:uuid", put(update_deployment_object))
        .route(
            "/deployment-objects/:uuid",
            delete(soft_delete_deployment_object),
        )
        .route(
            "/stacks/:stack_uuid/deployment-objects",
            get(list_deployment_objects),
        )
        .route(
            "/deployment-objects/active",
            get(list_active_deployment_objects),
        )
}

/// Handler for creating a new deployment object.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `new_deployment_object` - JSON payload containing the new deployment object data
///
/// # Returns
/// * On success: A tuple containing `StatusCode::CREATED` and the created `DeploymentObject`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn create_deployment_object(
    State(state): State<AppState>,
    Json(new_deployment_object): Json<NewDeploymentObject>,
) -> Result<(StatusCode, Json<DeploymentObject>), StatusCode> {
    state
        .dal
        .deployment_objects()
        .create(&new_deployment_object)
        .map(|object| (StatusCode::CREATED, Json(object)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for retrieving a deployment object by UUID.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the deployment object to retrieve
///
/// # Returns
/// * On success: JSON representation of the `DeploymentObject`
/// * On not found: `StatusCode::NOT_FOUND`

async fn get_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<DeploymentObject>, StatusCode> {
    state
        .dal
        .deployment_objects()
        .get_by_id(uuid)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Handler for updating a deployment object.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the deployment object to update
/// * `deployment_object` - JSON payload containing the updated deployment object data
///
/// # Returns
/// * On success: JSON representation of the updated `DeploymentObject`
/// * On failure:
///   - `StatusCode::BAD_REQUEST` if the object cannot be modified
///   - `StatusCode::INTERNAL_SERVER_ERROR` for other errors

async fn update_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(deployment_object): Json<DeploymentObject>,
) -> Result<Json<DeploymentObject>, (StatusCode, String)> {
    match state
        .dal
        .deployment_objects()
        .update(uuid, &deployment_object)
    {
        Ok(updated) => Ok(Json(updated)),
        Err(e) => {
            if e.to_string().contains("cannot be modified") {
                Err((StatusCode::BAD_REQUEST, "Deployment objects cannot be modified except for soft deletion or updating deletion markers".to_string()))
            } else {
                eprintln!("Error updating deployment object: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update: {:?}", e),
                ))
            }
        }
    }
}

/// Handler for soft-deleting a deployment object.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `uuid` - The UUID of the deployment object to soft delete
///
/// # Returns
/// * On success: `StatusCode::NO_CONTENT`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn soft_delete_deployment_object(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> StatusCode {
    state
        .dal
        .deployment_objects()
        .soft_delete(uuid)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all deployment objects for a specific stack.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `stack_uuid` - The UUID of the stack to list deployment objects for
///
/// # Returns
/// * On success: JSON array of `DeploymentObject` objects
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn list_deployment_objects(
    State(state): State<AppState>,
    Path(stack_uuid): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, StatusCode> {
    state
        .dal
        .deployment_objects()
        .get_by_stack_id(stack_uuid)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for listing all active deployment objects.
///
/// # Arguments
/// * `state` - The application state containing the DAL
///
/// # Returns
/// * On success: JSON array of active `DeploymentObject` objects
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR`

async fn list_active_deployment_objects(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeploymentObject>>, StatusCode> {
    state
        .dal
        .deployment_objects()
        .get_active()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
