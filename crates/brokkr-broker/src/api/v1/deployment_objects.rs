//! Deployment Objects API module for Brokkr.
//!
//! This module provides routes and handlers for managing deployment objects,
//! including retrieval based on user authentication and authorization.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use uuid::Uuid;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route(
            "/",
            get(list_deployment_objects).post(create_deployment_object),
        )
        .route(
            "/:id",
            get(get_deployment_object).delete(delete_deployment_object),
        )
}

/// Lists all deployment objects for a stack.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/stacks/{stack_id}/deployment-objects",
    tag = "deployment-objects",
    params(
        ("stack_id" = Uuid, Path, description = "ID of the stack to list deployment objects for"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved deployment objects", body = Vec<DeploymentObject>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
pub async fn list_deployment_objects(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Fetch deployment objects
    match dal.deployment_objects().list_for_stack(stack_id) {
        Ok(objects) => Ok(Json(objects)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch deployment objects"})),
        )),
    }
}

/// Creates a new deployment object.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/stacks/{stack_id}/deployment-objects",
    tag = "deployment-objects",
    params(
        ("stack_id" = Uuid, Path, description = "ID of the stack to create a deployment object for"),
    ),
    request_body = NewDeploymentObject,
    responses(
        (status = 200, description = "Successfully created deployment object", body = DeploymentObject),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
pub async fn create_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(new_deployment_object): Json<NewDeploymentObject>,
) -> Result<Json<DeploymentObject>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Create new deployment object with proper hash calculation
    let new_object = NewDeploymentObject::new(
        stack_id,
        new_deployment_object.yaml_content,
        new_deployment_object.is_deletion_marker,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    // Create the deployment object
    match dal.deployment_objects().create(&new_object) {
        Ok(object) => Ok(Json(object)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create deployment object"})),
        )),
    }
}

/// Retrieves a specific deployment object by ID.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/stacks/{stack_id}/deployment-objects/{id}",
    tag = "deployment-objects",
    params(
        ("stack_id" = Uuid, Path, description = "ID of the stack the deployment object belongs to"),
        ("id" = Uuid, Path, description = "ID of the deployment object to retrieve"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved deployment object", body = DeploymentObject),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Deployment object not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
pub async fn get_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeploymentObject>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Fetch deployment object
    match dal.deployment_objects().get(id) {
        Ok(Some(object)) => {
            if object.stack_id != stack_id {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Deployment object not found"})),
                ));
            }

            Ok(Json(object))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Deployment object not found"})),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch deployment object"})),
        )),
    }
}

/// Deletes a deployment object.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/stacks/{stack_id}/deployment-objects/{id}",
    tag = "deployment-objects",
    params(
        ("stack_id" = Uuid, Path, description = "ID of the stack the deployment object belongs to"),
        ("id" = Uuid, Path, description = "ID of the deployment object to delete"),
    ),
    responses(
        (status = 204, description = "Successfully deleted deployment object"),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = serde_json::Value),
        (status = 404, description = "Deployment object not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value),
    ),
    security(
        ("admin_pak" = []),
    )
)]
pub async fn delete_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Verify the deployment object exists and belongs to the stack
    match dal.deployment_objects().get(id) {
        Ok(Some(object)) => {
            if object.stack_id != stack_id {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Deployment object not found"})),
                ));
            }

            // Delete the deployment object
            match dal.deployment_objects().soft_delete(id) {
                Ok(_) => Ok(StatusCode::NO_CONTENT),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to delete deployment object"})),
                )),
            }
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Deployment object not found"})),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch deployment object"})),
        )),
    }
}
