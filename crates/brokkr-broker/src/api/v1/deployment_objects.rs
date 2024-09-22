//! Deployment Objects API module for Brokkr.
//!
//! This module provides routes and handlers for managing deployment objects,
//! including retrieval based on user authentication and authorization.

use crate::dal::DAL;
use crate::api::v1::middleware::AuthPayload;
use axum::{
    extract::{Extension, Path, State},
    routing::get,
    Json, Router,
};
use brokkr_models::models::deployment_objects::DeploymentObject;
use uuid::Uuid;

/// Creates and returns the router for deployment object endpoints.
///
/// # Returns
///
/// A `Router` instance configured with the deployment object routes.
pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/deployment-objects/:id", get(get_deployment_object))
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
    match dal.deployment_objects().get(id) {
        Ok(Some(object)) => {
            if auth_payload.admin {
                // Admin users have unrestricted access
                Ok(Json(object))
            } else if let Some(agent_id) = auth_payload.agent {
                // Check if the agent is associated with this deployment object
                match dal.agent_targets().list_for_agent(agent_id) {
                    Ok(targets) => {
                        if targets.iter().any(|target| target.stack_id == object.stack_id) {
                            Ok(Json(object))
                        } else {
                            Err((
                                axum::http::StatusCode::FORBIDDEN,
                                Json(serde_json::json!({"error": "Agent is not associated with this deployment object"})),
                            ))
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching agent targets: {:?}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to verify agent association"})),
                        ))
                    }
                }
            } else if let Some(generator_id) = auth_payload.generator {
                // Check if the generator is associated with this deployment object
                match dal.stacks().get(vec![object.stack_id]) {
                    Ok(stacks) => {
                        if let Some(stack) = stacks.into_iter().next() {
                            if stack.generator_id == generator_id {
                                Ok(Json(object))
                            } else {
                                Err((
                                    axum::http::StatusCode::FORBIDDEN,
                                    Json(serde_json::json!({"error": "Generator is not associated with this deployment object"})),
                                ))
                            }
                        } else {
                            Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({"error": "Associated stack not found"})),
                            ))
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching associated stack: {:?}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to fetch associated stack"})),
                        ))
                    }
                }
            } else {
                Err((
                    axum::http::StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Unauthorized access"})),
                ))
            }
        }
        Ok(None) => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Deployment object not found"})),
        )),
        Err(e) => {
            eprintln!("Error fetching deployment object: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch deployment object"})),
            ))
        }
    }
}