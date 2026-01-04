/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Deployment health API endpoints.
//!
//! This module provides routes and handlers for managing deployment health status,
//! including endpoints for agents to report health and for operators to query health.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Path, State},
    routing::{get, patch},
    Json, Router,
};
use brokkr_models::models::deployment_health::{
    DeploymentHealth, HealthSummary, NewDeploymentHealth,
};
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Creates and returns the router for health-related endpoints.
pub fn routes() -> Router<DAL> {
    info!("Setting up health routes");
    Router::new()
        .route("/agents/:id/health-status", patch(update_health_status))
        .route("/deployment-objects/:id/health", get(get_deployment_health))
        .route("/stacks/:id/health", get(get_stack_health))
}

/// Request body for updating health status from an agent.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthStatusUpdate {
    /// List of deployment object health updates.
    pub deployment_objects: Vec<DeploymentObjectHealthUpdate>,
}

/// Health update for a single deployment object.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeploymentObjectHealthUpdate {
    /// The deployment object ID.
    pub id: Uuid,
    /// Health status: healthy, degraded, failing, or unknown.
    pub status: String,
    /// Structured health summary.
    pub summary: Option<HealthSummary>,
    /// When the health was checked.
    pub checked_at: DateTime<Utc>,
}

/// Response for deployment object health query.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeploymentHealthResponse {
    /// The deployment object ID.
    pub deployment_object_id: Uuid,
    /// List of health records from different agents.
    pub health_records: Vec<DeploymentHealth>,
    /// Overall status (worst status across all agents).
    pub overall_status: String,
}

/// Response for stack health query.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StackHealthResponse {
    /// The stack ID.
    pub stack_id: Uuid,
    /// Overall status for the stack.
    pub overall_status: String,
    /// Health per deployment object.
    pub deployment_objects: Vec<DeploymentObjectHealthSummary>,
}

/// Summary of health for a deployment object within a stack.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeploymentObjectHealthSummary {
    /// The deployment object ID.
    pub id: Uuid,
    /// Overall status for this deployment object.
    pub status: String,
    /// Number of agents reporting healthy.
    pub healthy_agents: usize,
    /// Number of agents reporting degraded.
    pub degraded_agents: usize,
    /// Number of agents reporting failing.
    pub failing_agents: usize,
}

/// Updates health status for deployment objects from an agent.
///
/// # Authorization
/// Requires matching agent ID.
#[utoipa::path(
    patch,
    path = "/agents/{id}/health-status",
    tag = "health",
    params(
        ("id" = Uuid, Path, description = "ID of the agent reporting health"),
    ),
    request_body = HealthStatusUpdate,
    responses(
        (status = 200, description = "Successfully updated health status"),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 500, description = "Internal server error"),
    ),
    security(("agent_pak" = []))
)]
async fn update_health_status(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
    Json(update): Json<HealthStatusUpdate>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling health status update from agent {} for {} deployment objects",
        agent_id,
        update.deployment_objects.len()
    );

    // Verify the agent is authorized to report for this agent ID
    if auth_payload.agent != Some(agent_id) && !auth_payload.admin {
        warn!(
            "Unauthorized attempt to update health status for agent {}",
            agent_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    // Convert updates to NewDeploymentHealth records
    let health_records: Vec<NewDeploymentHealth> = update
        .deployment_objects
        .into_iter()
        .filter_map(|update| {
            let summary_json = update
                .summary
                .map(|s| serde_json::to_string(&s).ok())
                .flatten();

            NewDeploymentHealth::new(
                agent_id,
                update.id,
                update.status,
                summary_json,
                update.checked_at,
            )
            .ok()
        })
        .collect();

    if health_records.is_empty() {
        return Ok(StatusCode::OK);
    }

    match dal.deployment_health().upsert_batch(&health_records) {
        Ok(count) => {
            info!(
                "Successfully updated {} health records for agent {}",
                count, agent_id
            );
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!(
                "Failed to update health status for agent {}: {:?}",
                agent_id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update health status"})),
            ))
        }
    }
}

/// Gets health status for a specific deployment object.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/deployment-objects/{id}/health",
    tag = "health",
    params(
        ("id" = Uuid, Path, description = "ID of the deployment object"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved health", body = DeploymentHealthResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 500, description = "Internal server error"),
    ),
    security(("admin_pak" = []))
)]
async fn get_deployment_health(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(deployment_object_id): Path<Uuid>,
) -> Result<Json<DeploymentHealthResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to get health for deployment object {}",
        deployment_object_id
    );

    if !auth_payload.admin {
        warn!("Unauthorized attempt to get deployment health");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal
        .deployment_health()
        .list_by_deployment_object(deployment_object_id)
    {
        Ok(health_records) => {
            let overall_status = compute_overall_status(&health_records);

            Ok(Json(DeploymentHealthResponse {
                deployment_object_id,
                health_records,
                overall_status,
            }))
        }
        Err(e) => {
            error!(
                "Failed to get health for deployment object {}: {:?}",
                deployment_object_id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get deployment health"})),
            ))
        }
    }
}

/// Gets health status for all deployment objects in a stack.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/stacks/{id}/health",
    tag = "health",
    params(
        ("id" = Uuid, Path, description = "ID of the stack"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved stack health", body = StackHealthResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 500, description = "Internal server error"),
    ),
    security(("admin_pak" = []))
)]
async fn get_stack_health(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<StackHealthResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get health for stack {}", stack_id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to get stack health");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.deployment_health().list_by_stack(stack_id) {
        Ok(health_records) => {
            // Group health records by deployment object
            let mut deployment_health_map: std::collections::HashMap<Uuid, Vec<&DeploymentHealth>> =
                std::collections::HashMap::new();

            for record in &health_records {
                deployment_health_map
                    .entry(record.deployment_object_id)
                    .or_default()
                    .push(record);
            }

            // Compute summary per deployment object
            let deployment_objects: Vec<DeploymentObjectHealthSummary> = deployment_health_map
                .into_iter()
                .map(|(id, records)| {
                    let healthy_agents = records.iter().filter(|r| r.status == "healthy").count();
                    let degraded_agents = records.iter().filter(|r| r.status == "degraded").count();
                    let failing_agents = records.iter().filter(|r| r.status == "failing").count();

                    let status = if failing_agents > 0 {
                        "failing".to_string()
                    } else if degraded_agents > 0 {
                        "degraded".to_string()
                    } else if healthy_agents > 0 {
                        "healthy".to_string()
                    } else {
                        "unknown".to_string()
                    };

                    DeploymentObjectHealthSummary {
                        id,
                        status,
                        healthy_agents,
                        degraded_agents,
                        failing_agents,
                    }
                })
                .collect();

            // Compute overall stack status
            let overall_status = if deployment_objects.iter().any(|d| d.status == "failing") {
                "failing".to_string()
            } else if deployment_objects.iter().any(|d| d.status == "degraded") {
                "degraded".to_string()
            } else if deployment_objects.iter().any(|d| d.status == "healthy") {
                "healthy".to_string()
            } else {
                "unknown".to_string()
            };

            Ok(Json(StackHealthResponse {
                stack_id,
                overall_status,
                deployment_objects,
            }))
        }
        Err(e) => {
            error!("Failed to get health for stack {}: {:?}", stack_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get stack health"})),
            ))
        }
    }
}

/// Computes the overall status from a list of health records.
/// Returns the worst status: failing > degraded > healthy > unknown.
fn compute_overall_status(records: &[DeploymentHealth]) -> String {
    if records.iter().any(|r| r.status == "failing") {
        "failing".to_string()
    } else if records.iter().any(|r| r.status == "degraded") {
        "degraded".to_string()
    } else if records.iter().any(|r| r.status == "healthy") {
        "healthy".to_string()
    } else {
        "unknown".to_string()
    }
}
