/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostics API endpoints.
//!
//! This module provides routes and handlers for on-demand diagnostic requests.
//! Operators can request diagnostics for specific deployment objects, and agents
//! pick up and execute these requests, returning detailed diagnostic data.

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use brokkr_models::models::diagnostic_requests::{DiagnosticRequest, NewDiagnosticRequest};
use brokkr_models::models::diagnostic_results::{DiagnosticResult, NewDiagnosticResult};
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Creates and returns the router for diagnostic endpoints.
pub fn routes() -> Router<DAL> {
    info!("Setting up diagnostic routes");
    Router::new()
        .route(
            "/deployment-objects/:id/diagnostics",
            post(create_diagnostic_request),
        )
        .route("/diagnostics/:id", get(get_diagnostic))
        .route(
            "/agents/:id/diagnostics/pending",
            get(get_pending_diagnostics),
        )
        .route("/diagnostics/:id/claim", post(claim_diagnostic))
        .route("/diagnostics/:id/result", post(submit_diagnostic_result))
}

/// Request body for creating a diagnostic request.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateDiagnosticRequest {
    /// The agent that should handle this request.
    pub agent_id: Uuid,
    /// Who is requesting the diagnostics (optional).
    pub requested_by: Option<String>,
    /// How long the request should be retained in minutes (default 60, max 1440).
    pub retention_minutes: Option<i64>,
}

/// Response containing a diagnostic request with optional result.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DiagnosticResponse {
    /// The diagnostic request.
    pub request: DiagnosticRequest,
    /// The diagnostic result, if completed.
    pub result: Option<DiagnosticResult>,
}

/// Request body for submitting diagnostic results.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SubmitDiagnosticResult {
    /// JSON-encoded pod statuses.
    pub pod_statuses: String,
    /// JSON-encoded Kubernetes events.
    pub events: String,
    /// JSON-encoded log tails (optional).
    pub log_tails: Option<String>,
    /// When the diagnostics were collected.
    pub collected_at: DateTime<Utc>,
}

/// Creates a diagnostic request for a deployment object.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/api/v1/deployment-objects/{id}/diagnostics",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the deployment object"),
    ),
    request_body = CreateDiagnosticRequest,
    responses(
        (status = 201, description = "Successfully created diagnostic request", body = DiagnosticRequest),
        (status = 400, description = "Invalid request parameters"),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 500, description = "Internal server error"),
    ),
    security(("admin_pak" = []))
)]
async fn create_diagnostic_request(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(deployment_object_id): Path<Uuid>,
    Json(request): Json<CreateDiagnosticRequest>,
) -> Result<(StatusCode, Json<DiagnosticRequest>), (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to create diagnostic for deployment object {}",
        deployment_object_id
    );

    if !auth_payload.admin {
        warn!("Unauthorized attempt to create diagnostic request");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    // Verify the deployment object exists
    match dal.deployment_objects().get(deployment_object_id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Deployment object not found"})),
            ))
        }
        Err(e) => {
            error!(
                "Failed to fetch deployment object {}: {:?}",
                deployment_object_id, e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to verify deployment object"})),
            ));
        }
    }

    // Verify the agent exists and is associated with the deployment object's stack
    match dal.agents().get(request.agent_id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Agent not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch agent {}: {:?}", request.agent_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to verify agent"})),
            ));
        }
    }

    // Create the diagnostic request
    match NewDiagnosticRequest::new(
        request.agent_id,
        deployment_object_id,
        request.requested_by,
        request.retention_minutes,
    ) {
        Ok(new_request) => match dal.diagnostic_requests().create(&new_request) {
            Ok(diagnostic_request) => {
                info!(
                    "Created diagnostic request {} for deployment object {} assigned to agent {}",
                    diagnostic_request.id, deployment_object_id, request.agent_id
                );
                Ok((StatusCode::CREATED, Json(diagnostic_request)))
            }
            Err(e) => {
                error!("Failed to create diagnostic request: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create diagnostic request"})),
                ))
            }
        },
        Err(e) => {
            warn!("Invalid diagnostic request parameters: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e}))))
        }
    }
}

/// Gets a diagnostic request with its result if available.
///
/// # Authorization
/// Requires admin privileges or the assigned agent.
#[utoipa::path(
    get,
    path = "/api/v1/diagnostics/{id}",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved diagnostic", body = DiagnosticResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 404, description = "Diagnostic request not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("admin_pak" = []),
        ("agent_pak" = []),
    )
)]
async fn get_diagnostic(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<DiagnosticResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get diagnostic {}", id);

    match dal.diagnostic_requests().get(id) {
        Ok(Some(request)) => {
            // Check authorization
            if !auth_payload.admin && auth_payload.agent != Some(request.agent_id) {
                warn!("Unauthorized attempt to get diagnostic request {}", id);
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Unauthorized"})),
                ));
            }

            // Get the result if available
            let result = match dal.diagnostic_results().get_by_request(id) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to fetch diagnostic result for request {}: {:?}", id, e);
                    None
                }
            };

            Ok(Json(DiagnosticResponse { request, result }))
        }
        Ok(None) => {
            warn!("Diagnostic request not found: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Diagnostic request not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch diagnostic request {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch diagnostic request"})),
            ))
        }
    }
}

/// Gets pending diagnostic requests for an agent.
///
/// # Authorization
/// Requires matching agent ID.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{id}/diagnostics/pending",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved pending diagnostics", body = Vec<DiagnosticRequest>),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 500, description = "Internal server error"),
    ),
    security(("agent_pak" = []))
)]
async fn get_pending_diagnostics(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<DiagnosticRequest>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to get pending diagnostics for agent {}",
        agent_id
    );

    if auth_payload.agent != Some(agent_id) && !auth_payload.admin {
        warn!(
            "Unauthorized attempt to get pending diagnostics for agent {}",
            agent_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.diagnostic_requests().get_pending_for_agent(agent_id) {
        Ok(requests) => {
            info!(
                "Found {} pending diagnostic requests for agent {}",
                requests.len(),
                agent_id
            );
            Ok(Json(requests))
        }
        Err(e) => {
            error!(
                "Failed to fetch pending diagnostics for agent {}: {:?}",
                agent_id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch pending diagnostics"})),
            ))
        }
    }
}

/// Claims a diagnostic request for processing.
///
/// # Authorization
/// Requires matching agent ID.
#[utoipa::path(
    post,
    path = "/api/v1/diagnostics/{id}/claim",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request to claim"),
    ),
    responses(
        (status = 200, description = "Successfully claimed diagnostic request", body = DiagnosticRequest),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 404, description = "Diagnostic request not found"),
        (status = 409, description = "Conflict - request already claimed or completed"),
        (status = 500, description = "Internal server error"),
    ),
    security(("agent_pak" = []))
)]
async fn claim_diagnostic(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<DiagnosticRequest>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to claim diagnostic {}", id);

    // First get the request to check authorization and status
    match dal.diagnostic_requests().get(id) {
        Ok(Some(request)) => {
            // Verify the agent is authorized
            if auth_payload.agent != Some(request.agent_id) && !auth_payload.admin {
                warn!(
                    "Unauthorized attempt to claim diagnostic {} by {:?}",
                    id, auth_payload
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Unauthorized"})),
                ));
            }

            // Verify the request is in pending status
            if request.status != "pending" {
                warn!(
                    "Attempt to claim diagnostic {} with status {}",
                    id, request.status
                );
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": format!("Request is already {}", request.status)
                    })),
                ));
            }

            // Claim the request
            match dal.diagnostic_requests().claim(id) {
                Ok(claimed) => {
                    info!(
                        "Agent {} claimed diagnostic request {}",
                        request.agent_id, id
                    );
                    Ok(Json(claimed))
                }
                Err(e) => {
                    error!("Failed to claim diagnostic request {}: {:?}", id, e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to claim diagnostic request"})),
                    ))
                }
            }
        }
        Ok(None) => {
            warn!("Diagnostic request not found for claim: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Diagnostic request not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch diagnostic request {} for claim: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch diagnostic request"})),
            ))
        }
    }
}

/// Submits diagnostic results for a request.
///
/// # Authorization
/// Requires the assigned agent.
#[utoipa::path(
    post,
    path = "/api/v1/diagnostics/{id}/result",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request"),
    ),
    request_body = SubmitDiagnosticResult,
    responses(
        (status = 201, description = "Successfully submitted diagnostic result", body = DiagnosticResult),
        (status = 400, description = "Invalid result data"),
        (status = 403, description = "Forbidden - PAK does not have required rights"),
        (status = 404, description = "Diagnostic request not found"),
        (status = 409, description = "Conflict - result already submitted"),
        (status = 500, description = "Internal server error"),
    ),
    security(("agent_pak" = []))
)]
async fn submit_diagnostic_result(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(request_id): Path<Uuid>,
    Json(result): Json<SubmitDiagnosticResult>,
) -> Result<(StatusCode, Json<DiagnosticResult>), (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling submission of diagnostic result for request {}",
        request_id
    );

    // Get the request to verify authorization
    match dal.diagnostic_requests().get(request_id) {
        Ok(Some(request)) => {
            // Verify the agent is authorized
            if auth_payload.agent != Some(request.agent_id) {
                warn!(
                    "Unauthorized attempt to submit result for diagnostic {} by {:?}",
                    request_id, auth_payload
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Unauthorized"})),
                ));
            }

            // Verify the request is in claimed status
            if request.status != "claimed" {
                warn!(
                    "Attempt to submit result for diagnostic {} with status {}",
                    request_id, request.status
                );
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": format!("Request status is {}, expected 'claimed'", request.status)
                    })),
                ));
            }

            // Create the result
            match NewDiagnosticResult::new(
                request_id,
                result.pod_statuses,
                result.events,
                result.log_tails,
                result.collected_at,
            ) {
                Ok(new_result) => {
                    // Insert result and mark request as completed
                    match dal.diagnostic_results().create(&new_result) {
                        Ok(diagnostic_result) => {
                            // Mark the request as completed
                            if let Err(e) = dal.diagnostic_requests().complete(request_id) {
                                error!(
                                    "Failed to mark diagnostic request {} as completed: {:?}",
                                    request_id, e
                                );
                            }

                            info!(
                                "Agent {} submitted diagnostic result for request {}",
                                request.agent_id, request_id
                            );
                            Ok((StatusCode::CREATED, Json(diagnostic_result)))
                        }
                        Err(e) => {
                            error!("Failed to create diagnostic result: {:?}", e);
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({"error": "Failed to store diagnostic result"})),
                            ))
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid diagnostic result: {}", e);
                    Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e}))))
                }
            }
        }
        Ok(None) => {
            warn!("Diagnostic request not found for result submission: {}", request_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Diagnostic request not found"})),
            ))
        }
        Err(e) => {
            error!(
                "Failed to fetch diagnostic request {} for result submission: {:?}",
                request_id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch diagnostic request"})),
            ))
        }
    }
}
