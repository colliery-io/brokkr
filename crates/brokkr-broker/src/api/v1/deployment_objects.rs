//! Deployment Objects API module for Brokkr.
//!
//! This module provides routes and handlers for managing deployment objects,
//! including retrieval based on user authentication and authorization.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{
    extract::{Extension, Path, State},
    routing::get,
    Json, Router,
};
use brokkr_models::models::deployment_objects::DeploymentObject;
use brokkr_utils::logging::prelude::*;
use uuid::Uuid;

/// Creates and returns the router for deployment object endpoints.
///
/// # Returns
///
/// A `Router` instance configured with the deployment object routes.
pub fn routes() -> Router<DAL> {
    info!("Setting up deployment object routes");
    Router::new().route("/deployment-objects/:id", get(get_deployment_object))
}

/// Retrieves a deployment object by ID, with access control based on user role.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
/// * `id` - The UUID of the deployment object to retrieve.
///
/// # Returns
///
/// A `Result` containing either the `DeploymentObject` as JSON or an error response.
async fn get_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeploymentObject>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get deployment object with ID: {}", id);
    match dal.deployment_objects().get(id) {
        Ok(Some(object)) => {
            if auth_payload.admin {
                info!("Admin user accessed deployment object with ID: {}", id);
                Ok(Json(object))
            } else if let Some(agent_id) = auth_payload.agent {
                match dal.agent_targets().list_for_agent(agent_id) {
                    Ok(targets) => {
                        if targets
                            .iter()
                            .any(|target| target.stack_id == object.stack_id)
                        {
                            info!(
                                "Agent {} accessed deployment object with ID: {}",
                                agent_id, id
                            );
                            Ok(Json(object))
                        } else {
                            warn!("Agent {} attempted to access unauthorized deployment object with ID: {}", agent_id, id);
                            Err((
                                axum::http::StatusCode::FORBIDDEN,
                                Json(
                                    serde_json::json!({"error": "Agent is not associated with this deployment object"}),
                                ),
                            ))
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to fetch agent targets for agent {}: {:?}",
                            agent_id, e
                        );
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to fetch agent targets"})),
                        ))
                    }
                }
            } else if let Some(generator_id) = auth_payload.generator {
                // Check if the generator is associated with the stack of this deployment object
                match dal.stacks().get(vec![object.stack_id]) {
                    Ok(stacks) => {
                        if let Some(stack) = stacks.first() {
                            if stack.generator_id == generator_id {
                                info!(
                                    "Generator '{}' (id: {}) accessed deployment object '{}' (id: {})",
                                    stack.name, generator_id, object.yaml_content, id
                                );
                                Ok(Json(object))
                            } else {
                                warn!(
                                    "Generator '{}' (id: {}) attempted unauthorized access to deployment object '{}' (id: {}) owned by generator {}",
                                    stack.name, generator_id, object.yaml_content, id, stack.generator_id
                                );
                                Err((
                                    axum::http::StatusCode::FORBIDDEN,
                                    Json(
                                        serde_json::json!({"error": "Generator is not associated with this deployment object"}),
                                    ),
                                ))
                            }
                        } else {
                            warn!(
                                "Stack not found for deployment object '{}' (id: {}, stack_id: {})",
                                object.yaml_content, id, object.stack_id
                            );
                            Err((
                                axum::http::StatusCode::NOT_FOUND,
                                Json(
                                    serde_json::json!({"error": "Stack not found for this deployment object"}),
                                ),
                            ))
                        }
                    }
                    Err(e) => {
                        error!(
                            "Database error while fetching stack for deployment object '{}' (id: {}, stack_id: {}): {:?}",
                            object.yaml_content, id, object.stack_id, e
                        );
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to fetch stack information"})),
                        ))
                    }
                }
            } else {
                warn!(
                    "Unauthorized access attempt to deployment object with ID: {}",
                    id
                );
                Err((
                    axum::http::StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Unauthorized access"})),
                ))
            }
        }
        Ok(None) => {
            warn!("Deployment object not found with ID: {}", id);
            Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Deployment object not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch deployment object with ID {}: {:?}", id, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch deployment object"})),
            ))
        }
    }
}
