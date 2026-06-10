/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Deployment Objects API module for Brokkr.
//!
//! This module provides routes and handlers for managing deployment objects,
//! including retrieval based on user authentication and authorization.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::get,
};
use brokkr_models::models::deployment_objects::DeploymentObject;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Creates and returns the router for deployment object endpoints.
pub fn routes() -> Router<DAL> {
    info!("Setting up deployment object routes");
    Router::new().route("/deployment-objects/:id", get(get_deployment_object))
}

/// Retrieves a deployment object by ID, with access control based on user role.
///
/// # Authorization
/// Requires either:
/// - Admin privileges
/// - Agent associated with the deployment object's stack
/// - Generator that owns the deployment object's stack
#[utoipa::path(
    get,
    path = "/deployment-objects/{id}",
    tag = "deployment-objects",
    params(
        ("id" = Uuid, Path, description = "ID of the deployment object to retrieve"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved deployment object", body = DeploymentObject),
        (status = 401, description = "Unauthorized - No valid PAK provided", body = ErrorResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 404, description = "Deployment object not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
        ("generator_pak" = []),
    )
)]
async fn get_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeploymentObject>, ApiError> {
    info!("Handling request to get deployment object with ID: {}", id);

    let object = dal
        .deployment_objects()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch deployment object with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch deployment object")
        })?
        .ok_or_else(|| {
            warn!("Deployment object not found with ID: {}", id);
            ApiError::not_found("deployment_object_not_found", "deployment object not found")
        })?;

    if auth_payload.admin {
        info!("Admin user accessed deployment object with ID: {}", id);
        return Ok(Json(object));
    }

    if let Some(agent_id) = auth_payload.agent {
        let targets = dal.agent_targets().list_for_agent(agent_id).map_err(|e| {
            error!(
                "Failed to fetch agent targets for agent {}: {:?}",
                agent_id, e
            );
            ApiError::internal("failed to fetch agent targets")
        })?;

        if targets.iter().any(|t| t.stack_id == object.stack_id) {
            info!(
                "Agent {} accessed deployment object with ID: {}",
                agent_id, id
            );
            return Ok(Json(object));
        }

        warn!(
            "Agent {} attempted to access unauthorized deployment object with ID: {}",
            agent_id, id
        );
        return Err(ApiError::forbidden(
            "agent_not_associated",
            "agent is not associated with this deployment object",
        ));
    }

    if let Some(generator_id) = auth_payload.generator {
        let stacks = dal.stacks().get(vec![object.stack_id]).map_err(|e| {
            error!(
                "Database error while fetching stack for deployment object '{}' (id: {}, stack_id: {}): {:?}",
                object.yaml_content, id, object.stack_id, e
            );
            ApiError::internal("failed to fetch stack information")
        })?;

        let stack = stacks.first().ok_or_else(|| {
            warn!(
                "Stack not found for deployment object '{}' (id: {}, stack_id: {})",
                object.yaml_content, id, object.stack_id
            );
            ApiError::not_found(
                "stack_not_found",
                "stack not found for this deployment object",
            )
        })?;

        if stack.generator_id == generator_id {
            info!(
                "Generator '{}' (id: {}) accessed deployment object '{}' (id: {})",
                stack.name, generator_id, object.yaml_content, id
            );
            return Ok(Json(object));
        }

        warn!(
            "Generator '{}' (id: {}) attempted unauthorized access to deployment object '{}' (id: {}) owned by generator {}",
            stack.name, generator_id, object.yaml_content, id, stack.generator_id
        );
        return Err(ApiError::forbidden(
            "generator_not_associated",
            "generator is not associated with this deployment object",
        ));
    }

    warn!(
        "Unauthorized access attempt to deployment object with ID: {}",
        id
    );
    Err(ApiError::unauthorized(
        "unauthorized",
        "unauthorized access",
    ))
}
