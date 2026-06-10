/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostics API endpoints.
//!
//! This module provides routes and handlers for on-demand diagnostic requests.
//! Operators can request diagnostics for specific deployment objects, and agents
//! pick up and execute these requests, returning detailed diagnostic data.

use crate::api::v1::error::{ApiError, ErrorResponse};
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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
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

#[utoipa::path(
    post,
    path = "/deployment-objects/{id}/diagnostics",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the deployment object"),
    ),
    request_body = CreateDiagnosticRequest,
    responses(
        (status = 201, description = "Successfully created diagnostic request", body = DiagnosticRequest),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 404, description = "Deployment object or agent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn create_diagnostic_request(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(deployment_object_id): Path<Uuid>,
    Json(request): Json<CreateDiagnosticRequest>,
) -> Result<(StatusCode, Json<DiagnosticRequest>), ApiError> {
    info!(
        "Handling request to create diagnostic for deployment object {}",
        deployment_object_id
    );

    if !auth_payload.admin {
        warn!("Unauthorized attempt to create diagnostic request");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    dal.deployment_objects()
        .get(deployment_object_id)
        .map_err(|e| {
            error!(
                "Failed to fetch deployment object {}: {:?}",
                deployment_object_id, e
            );
            ApiError::internal("failed to verify deployment object")
        })?
        .ok_or_else(|| {
            ApiError::not_found("deployment_object_not_found", "deployment object not found")
        })?;

    dal.agents()
        .get(request.agent_id)
        .map_err(|e| {
            error!("Failed to fetch agent {}: {:?}", request.agent_id, e);
            ApiError::internal("failed to verify agent")
        })?
        .ok_or_else(|| ApiError::not_found("agent_not_found", "agent not found"))?;

    let new_request = NewDiagnosticRequest::new(
        request.agent_id,
        deployment_object_id,
        request.requested_by,
        request.retention_minutes,
    )
    .map_err(|e| {
        warn!("Invalid diagnostic request parameters: {}", e);
        ApiError::bad_request("invalid_diagnostic_request", e)
    })?;

    let diagnostic_request = dal
        .diagnostic_requests()
        .create(&new_request)
        .map_err(|e| {
            error!("Failed to create diagnostic request: {:?}", e);
            ApiError::internal("failed to create diagnostic request")
        })?;

    info!(
        "Created diagnostic request {} for deployment object {} assigned to agent {}",
        diagnostic_request.id, deployment_object_id, request.agent_id
    );
    Ok((StatusCode::CREATED, Json(diagnostic_request)))
}

#[utoipa::path(
    get,
    path = "/diagnostics/{id}",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved diagnostic", body = DiagnosticResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 404, description = "Diagnostic request not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
) -> Result<Json<DiagnosticResponse>, ApiError> {
    info!("Handling request to get diagnostic {}", id);

    let request = dal
        .diagnostic_requests()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch diagnostic request {}: {:?}", id, e);
            ApiError::internal("failed to fetch diagnostic request")
        })?
        .ok_or_else(|| {
            warn!("Diagnostic request not found: {}", id);
            ApiError::not_found("diagnostic_not_found", "diagnostic request not found")
        })?;

    if !auth_payload.admin && auth_payload.agent != Some(request.agent_id) {
        warn!("Unauthorized attempt to get diagnostic request {}", id);
        return Err(ApiError::forbidden(
            "diagnostic_not_owned",
            "not authorized to access this diagnostic request",
        ));
    }

    let result = match dal.diagnostic_results().get_by_request(id) {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to fetch diagnostic result for request {}: {:?}",
                id, e
            );
            None
        }
    };

    Ok(Json(DiagnosticResponse { request, result }))
}

#[utoipa::path(
    get,
    path = "/agents/{id}/diagnostics/pending",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the agent"),
    ),
    responses(
        (status = 200, description = "Successfully retrieved pending diagnostics", body = Vec<DiagnosticRequest>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("agent_pak" = []))
)]
async fn get_pending_diagnostics(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<DiagnosticRequest>>, ApiError> {
    info!(
        "Handling request to get pending diagnostics for agent {}",
        agent_id
    );

    if auth_payload.agent != Some(agent_id) && !auth_payload.admin {
        warn!(
            "Unauthorized attempt to get pending diagnostics for agent {}",
            agent_id
        );
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "agent PAK does not match the agent ID",
        ));
    }

    let requests = dal
        .diagnostic_requests()
        .get_pending_for_agent(agent_id)
        .map_err(|e| {
            error!(
                "Failed to fetch pending diagnostics for agent {}: {:?}",
                agent_id, e
            );
            ApiError::internal("failed to fetch pending diagnostics")
        })?;

    info!(
        "Found {} pending diagnostic requests for agent {}",
        requests.len(),
        agent_id
    );
    Ok(Json(requests))
}

#[utoipa::path(
    post,
    path = "/diagnostics/{id}/claim",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request to claim"),
    ),
    responses(
        (status = 200, description = "Successfully claimed diagnostic request", body = DiagnosticRequest),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 404, description = "Diagnostic request not found", body = ErrorResponse),
        (status = 409, description = "Conflict - request already claimed or completed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("agent_pak" = []))
)]
async fn claim_diagnostic(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<DiagnosticRequest>, ApiError> {
    info!("Handling request to claim diagnostic {}", id);

    let request = dal
        .diagnostic_requests()
        .get(id)
        .map_err(|e| {
            error!(
                "Failed to fetch diagnostic request {} for claim: {:?}",
                id, e
            );
            ApiError::internal("failed to fetch diagnostic request")
        })?
        .ok_or_else(|| {
            warn!("Diagnostic request not found for claim: {}", id);
            ApiError::not_found("diagnostic_not_found", "diagnostic request not found")
        })?;

    if auth_payload.agent != Some(request.agent_id) && !auth_payload.admin {
        warn!("Unauthorized attempt to claim diagnostic {}", id);
        return Err(ApiError::forbidden(
            "diagnostic_not_owned",
            "not authorized to claim this diagnostic request",
        ));
    }

    if request.status != "pending" {
        warn!(
            "Attempt to claim diagnostic {} with status {}",
            id, request.status
        );
        return Err(ApiError::conflict(
            "diagnostic_already_claimed",
            format!("request is already {}", request.status),
        ));
    }

    let claimed = dal.diagnostic_requests().claim(id).map_err(|e| {
        error!("Failed to claim diagnostic request {}: {:?}", id, e);
        ApiError::internal("failed to claim diagnostic request")
    })?;

    info!(
        "Agent {} claimed diagnostic request {}",
        request.agent_id, id
    );
    Ok(Json(claimed))
}

#[utoipa::path(
    post,
    path = "/diagnostics/{id}/result",
    tag = "diagnostics",
    params(
        ("id" = Uuid, Path, description = "ID of the diagnostic request"),
    ),
    request_body = SubmitDiagnosticResult,
    responses(
        (status = 201, description = "Successfully submitted diagnostic result", body = DiagnosticResult),
        (status = 400, description = "Invalid result data", body = ErrorResponse),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = ErrorResponse),
        (status = 404, description = "Diagnostic request not found", body = ErrorResponse),
        (status = 409, description = "Conflict - result already submitted", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("agent_pak" = []))
)]
async fn submit_diagnostic_result(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(request_id): Path<Uuid>,
    Json(result): Json<SubmitDiagnosticResult>,
) -> Result<(StatusCode, Json<DiagnosticResult>), ApiError> {
    info!(
        "Handling submission of diagnostic result for request {}",
        request_id
    );

    let request = dal
        .diagnostic_requests()
        .get(request_id)
        .map_err(|e| {
            error!(
                "Failed to fetch diagnostic request {} for result submission: {:?}",
                request_id, e
            );
            ApiError::internal("failed to fetch diagnostic request")
        })?
        .ok_or_else(|| {
            warn!(
                "Diagnostic request not found for result submission: {}",
                request_id
            );
            ApiError::not_found("diagnostic_not_found", "diagnostic request not found")
        })?;

    if auth_payload.agent != Some(request.agent_id) {
        warn!(
            "Unauthorized attempt to submit result for diagnostic {}",
            request_id
        );
        return Err(ApiError::forbidden(
            "diagnostic_not_owned",
            "not authorized to submit result for this diagnostic request",
        ));
    }

    if request.status != "claimed" {
        warn!(
            "Attempt to submit result for diagnostic {} with status {}",
            request_id, request.status
        );
        return Err(ApiError::conflict(
            "diagnostic_not_claimed",
            format!("request status is {}, expected 'claimed'", request.status),
        ));
    }

    let new_result = NewDiagnosticResult::new(
        request_id,
        result.pod_statuses,
        result.events,
        result.log_tails,
        result.collected_at,
    )
    .map_err(|e| {
        warn!("Invalid diagnostic result: {}", e);
        ApiError::bad_request("invalid_diagnostic_result", e)
    })?;

    let diagnostic_result = dal.diagnostic_results().create(&new_result).map_err(|e| {
        error!("Failed to create diagnostic result: {:?}", e);
        ApiError::internal("failed to store diagnostic result")
    })?;

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
