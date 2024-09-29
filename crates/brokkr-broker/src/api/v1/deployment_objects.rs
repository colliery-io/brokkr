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
use brokkr_utils::logging::prelude::*;

/// Creates and returns the router for deployment object endpoints.
///
/// # Returns
///
/// A `Router` instance configured with the deployment object routes.
pub fn routes() -> Router<DAL> {
    info!("Setting up deployment object routes");
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
    info!("Handling request to get deployment object with ID: {}", id);
    match dal.deployment_objects().get(id) {
        Ok(Some(object)) => {
            if auth_payload.admin {
                info!("Admin user accessed deployment object with ID: {}", id);
                Ok(Json(object))
            } else if let Some(agent_id) = auth_payload.agent {
                match dal.agent_targets().list_for_agent(agent_id) {
                    Ok(targets) => {
                        if targets.iter().any(|target| target.stack_id == object.stack_id) {
                            info!("Agent {} accessed deployment object with ID: {}", agent_id, id);
                            Ok(Json(object))
                        } else {
                            warn!("Agent {} attempted to access unauthorized deployment object with ID: {}", agent_id, id);
                            Err((
                                axum::http::StatusCode::FORBIDDEN,
                                Json(serde_json::json!({"error": "Agent is not associated with this deployment object"})),
                            ))
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch agent targets for agent {}: {:?}", agent_id, e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to fetch agent targets"})),
                        ))
                    }
                }
            } else {
                warn!("Unauthorized access attempt to deployment object with ID: {}", id);
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