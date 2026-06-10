/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Handles API routes and logic for work orders.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::audit;
use crate::ws::{ConnectionRegistry, push_work_order};
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use brokkr_models::models::audit_logs::{
    ACTION_WORKORDER_CLAIMED, ACTION_WORKORDER_COMPLETED, ACTION_WORKORDER_CREATED,
    ACTION_WORKORDER_FAILED, ACTION_WORKORDER_RETRY, ACTOR_TYPE_ADMIN, ACTOR_TYPE_AGENT,
    ACTOR_TYPE_GENERATOR, RESOURCE_TYPE_WORKORDER,
};
use brokkr_models::models::work_orders::{NewWorkOrder, WorkOrder, WorkOrderLog};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

pub fn routes() -> Router<DAL> {
    info!("Setting up work order routes");
    Router::new()
        .route(
            "/work-orders",
            get(list_work_orders).post(create_work_order),
        )
        .route(
            "/work-orders/:id",
            get(get_work_order).delete(delete_work_order),
        )
        .route("/work-orders/:id/claim", post(claim_work_order))
        .route("/work-orders/:id/complete", post(complete_work_order))
        .route("/work-order-log", get(list_work_order_log))
        .route("/work-order-log/:id", get(get_work_order_log))
}

pub fn agent_routes() -> Router<DAL> {
    Router::new().route(
        "/agents/:agent_id/work-orders/pending",
        get(list_pending_for_agent),
    )
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateWorkOrderRequest {
    pub work_type: String,
    pub yaml_content: String,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub backoff_seconds: Option<i32>,
    #[serde(default)]
    pub claim_timeout_seconds: Option<i32>,
    #[serde(default)]
    pub targeting: Option<WorkOrderTargeting>,
    #[serde(default)]
    pub target_agent_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct WorkOrderTargeting {
    #[serde(default)]
    pub agent_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ClaimWorkOrderRequest {
    pub agent_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteWorkOrderRequest {
    pub success: bool,
    pub message: Option<String>,
    #[serde(default = "default_retryable")]
    pub retryable: bool,
}

fn default_retryable() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ListWorkOrdersQuery {
    pub status: Option<String>,
    pub work_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListPendingQuery {
    pub work_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListLogQuery {
    pub work_type: Option<String>,
    pub success: Option<bool>,
    pub agent_id: Option<Uuid>,
    pub limit: Option<i64>,
}

// =============================================================================
// WORK ORDER MANAGEMENT
// =============================================================================

#[utoipa::path(
    get,
    path = "/work-orders",
    tag = "work-orders",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("work_type" = Option<String>, Query, description = "Filter by work type")
    ),
    responses(
        (status = 200, description = "List of work orders", body = Vec<WorkOrder>),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn list_work_orders(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(params): Query<ListWorkOrdersQuery>,
) -> Result<Json<Vec<WorkOrder>>, ApiError> {
    info!("Handling request to list work orders");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list work orders");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let work_orders = dal
        .work_orders()
        .list_filtered(params.status.as_deref(), params.work_type.as_deref())
        .map_err(|e| {
            error!("Failed to fetch work orders: {:?}", e);
            ApiError::internal("failed to fetch work orders")
        })?;
    info!("Successfully retrieved {} work orders", work_orders.len());
    Ok(Json(work_orders))
}

#[utoipa::path(
    post,
    path = "/work-orders",
    tag = "work-orders",
    request_body = CreateWorkOrderRequest,
    responses(
        (status = 201, description = "Work order created", body = WorkOrder),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn create_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Json(request): Json<CreateWorkOrderRequest>,
) -> Result<(StatusCode, Json<WorkOrder>), ApiError> {
    info!("Handling request to create a new work order");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to create work order");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let targeting = request.targeting.unwrap_or_default();
    let legacy_agent_ids = request.target_agent_ids.unwrap_or_default();
    let agent_ids: Vec<Uuid> = targeting
        .agent_ids
        .unwrap_or_default()
        .into_iter()
        .chain(legacy_agent_ids)
        .collect();
    let labels = targeting.labels.unwrap_or_default();
    let annotations = targeting.annotations.unwrap_or_default();

    if agent_ids.is_empty() && labels.is_empty() && annotations.is_empty() {
        return Err(ApiError::bad_request(
            "no_targeting_specified",
            "at least one targeting method must be specified (agent_ids, labels, or annotations)",
        ));
    }

    let new_work_order = NewWorkOrder::new(
        request.work_type,
        request.yaml_content,
        request.max_retries,
        request.backoff_seconds,
        request.claim_timeout_seconds,
    )
    .map_err(|e| ApiError::bad_request("invalid_work_order", e))?;

    let work_order = dal.work_orders().create(&new_work_order).map_err(|e| {
        error!("Failed to create work order: {:?}", e);
        ApiError::internal("failed to create work order")
    })?;

    let mut targeting_failed: Option<ApiError> = None;
    if !agent_ids.is_empty()
        && let Err(e) = dal.work_orders().add_targets(work_order.id, &agent_ids)
    {
        error!("Failed to add work order targets: {:?}", e);
        targeting_failed = Some(ApiError::internal("failed to add work order targets"));
    }
    if targeting_failed.is_none()
        && !labels.is_empty()
        && let Err(e) = dal.work_orders().add_labels(work_order.id, &labels)
    {
        error!("Failed to add work order labels: {:?}", e);
        targeting_failed = Some(ApiError::internal("failed to add work order labels"));
    }
    if targeting_failed.is_none()
        && !annotations.is_empty()
        && let Err(e) = dal
            .work_orders()
            .add_annotations(work_order.id, &annotations)
    {
        error!("Failed to add work order annotations: {:?}", e);
        targeting_failed = Some(ApiError::internal("failed to add work order annotations"));
    }

    if let Some(err) = targeting_failed {
        let _ = dal.work_orders().delete(work_order.id);
        return Err(err);
    }

    // Post-commit: push to explicitly-targeted agents via WS. Label /
    // annotation targeting still relies on the agent's REST polling to
    // resolve which work orders apply — broadening this push to those
    // selectors is part of [[BROKKR-I-0019]] WS-04 follow-ups.
    push_work_order(&ws_registry, &work_order, &agent_ids);

    info!("Successfully created work order with ID: {}", work_order.id);
    let (actor_type, actor_id) = if auth_payload.admin {
        (ACTOR_TYPE_ADMIN, None)
    } else {
        (ACTOR_TYPE_GENERATOR, auth_payload.generator)
    };
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_WORKORDER_CREATED,
        RESOURCE_TYPE_WORKORDER,
        Some(work_order.id),
        Some(serde_json::json!({ "work_type": work_order.work_type })),
        None,
        None,
    );
    Ok((StatusCode::CREATED, Json(work_order)))
}

#[utoipa::path(
    get,
    path = "/work-orders/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    responses(
        (status = 200, description = "Work order found", body = WorkOrder),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 404, description = "Work order not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn get_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkOrder>, ApiError> {
    info!("Handling request to get work order with ID: {}", id);
    if !auth_payload.admin {
        warn!("Unauthorized attempt to get work order");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    if let Some(work_order) = dal.work_orders().get(id).map_err(|e| {
        error!("Failed to fetch work order with ID {}: {:?}", id, e);
        ApiError::internal("failed to fetch work order")
    })? {
        info!("Successfully retrieved work order with ID: {}", id);
        return Ok(Json(work_order));
    }

    debug!("Work order {} not in active table, checking log", id);
    let log_entry = dal
        .work_orders()
        .get_log(id)
        .map_err(|e| {
            error!("Failed to fetch work order log with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch work order")
        })?
        .ok_or_else(|| {
            warn!("Work order not found with ID: {}", id);
            ApiError::not_found("work_order_not_found", "work order not found")
        })?;

    info!(
        "Successfully retrieved completed work order with ID: {} from log",
        id
    );
    let status = if log_entry.success {
        "COMPLETED".to_string()
    } else {
        "FAILED".to_string()
    };
    let (last_error, last_error_at) = if !log_entry.success {
        (
            log_entry.result_message.clone(),
            Some(log_entry.completed_at),
        )
    } else {
        (None, None)
    };
    Ok(Json(WorkOrder {
        id: log_entry.id,
        created_at: log_entry.created_at,
        updated_at: log_entry.completed_at,
        work_type: log_entry.work_type,
        yaml_content: log_entry.yaml_content,
        status,
        claimed_by: log_entry.claimed_by,
        claimed_at: log_entry.claimed_at,
        retry_count: log_entry.retries_attempted,
        max_retries: 0,
        next_retry_after: None,
        backoff_seconds: 0,
        claim_timeout_seconds: 0,
        last_error,
        last_error_at,
    }))
}

#[utoipa::path(
    delete,
    path = "/work-orders/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    responses(
        (status = 204, description = "Work order deleted"),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 404, description = "Work order not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn delete_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!("Handling request to delete work order with ID: {}", id);
    if !auth_payload.admin {
        warn!("Unauthorized attempt to delete work order");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let deleted = dal.work_orders().delete(id).map_err(|e| {
        error!("Failed to delete work order with ID {}: {:?}", id, e);
        ApiError::internal("failed to delete work order")
    })?;

    if deleted == 0 {
        warn!("Work order not found with ID: {}", id);
        return Err(ApiError::not_found(
            "work_order_not_found",
            "work order not found",
        ));
    }
    info!("Successfully deleted work order with ID: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// AGENT OPERATIONS
// =============================================================================

#[utoipa::path(
    get,
    path = "/agents/{agent_id}/work-orders/pending",
    tag = "work-orders",
    params(
        ("agent_id" = Uuid, Path, description = "Agent ID"),
        ("work_type" = Option<String>, Query, description = "Filter by work type")
    ),
    responses(
        (status = 200, description = "List of pending work orders", body = Vec<WorkOrder>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn list_pending_for_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
    Query(params): Query<ListPendingQuery>,
) -> Result<Json<Vec<WorkOrder>>, ApiError> {
    info!(
        "Handling request to list pending work orders for agent: {}",
        agent_id
    );

    if !auth_payload.admin && auth_payload.agent != Some(agent_id) {
        warn!("Unauthorized attempt to list pending work orders for agent");
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "agent PAK does not match the agent ID",
        ));
    }

    let work_orders = dal
        .work_orders()
        .list_pending_for_agent(agent_id, params.work_type.as_deref())
        .map_err(|e| {
            error!(
                "Failed to fetch pending work orders for agent {}: {:?}",
                agent_id, e
            );
            ApiError::internal("failed to fetch pending work orders")
        })?;

    info!(
        "Successfully retrieved {} pending work orders for agent {}",
        work_orders.len(),
        agent_id
    );
    Ok(Json(work_orders))
}

#[utoipa::path(
    post,
    path = "/work-orders/{id}/claim",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    request_body = ClaimWorkOrderRequest,
    responses(
        (status = 200, description = "Work order claimed", body = WorkOrder),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Work order not found or not claimable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn claim_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<ClaimWorkOrderRequest>,
) -> Result<Json<WorkOrder>, ApiError> {
    info!(
        "Handling request to claim work order {} by agent {}",
        id, request.agent_id
    );

    if !auth_payload.admin && auth_payload.agent != Some(request.agent_id) {
        warn!("Unauthorized attempt to claim work order");
        return Err(ApiError::forbidden(
            "agent_pak_mismatch",
            "agent PAK does not match the claiming agent",
        ));
    }

    match dal.work_orders().claim(id, request.agent_id) {
        Ok(work_order) => {
            info!(
                "Successfully claimed work order {} by agent {}",
                id, request.agent_id
            );
            audit::log_action(
                ACTOR_TYPE_AGENT,
                Some(request.agent_id),
                ACTION_WORKORDER_CLAIMED,
                RESOURCE_TYPE_WORKORDER,
                Some(id),
                None,
                None,
                None,
            );
            Ok(Json(work_order))
        }
        Err(diesel::result::Error::NotFound) => {
            warn!(
                "Work order {} not found or not claimable by agent {}",
                id, request.agent_id
            );
            Err(ApiError::not_found(
                "work_order_not_claimable",
                "work order not found or not claimable",
            ))
        }
        Err(e) => {
            error!("Failed to claim work order {}: {:?}", id, e);
            Err(ApiError::internal("failed to claim work order"))
        }
    }
}

#[utoipa::path(
    post,
    path = "/work-orders/{id}/complete",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    request_body = CompleteWorkOrderRequest,
    responses(
        // Note: at runtime this endpoint may also return 202 with a
        // `{"status":"retry_scheduled"}` payload when the work order failed
        // but has retries remaining. Progenitor (and most SDK generators)
        // require a single distinct success response type per operation, so
        // the 202 case is intentionally omitted here. SDK consumers that need
        // to react to retry scheduling should match on the raw status code.
        (status = 200, description = "Work order completed", body = WorkOrderLog),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Work order not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("agent_pak" = []))
)]
async fn complete_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<CompleteWorkOrderRequest>,
) -> Result<axum::response::Response, ApiError> {
    info!(
        "Handling request to complete work order {} (success: {})",
        id, request.success
    );

    let work_order = dal
        .work_orders()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch work order {}: {:?}", id, e);
            ApiError::internal("failed to fetch work order")
        })?
        .ok_or_else(|| ApiError::not_found("work_order_not_found", "work order not found"))?;

    if !auth_payload.admin && auth_payload.agent != work_order.claimed_by {
        warn!("Unauthorized attempt to complete work order");
        return Err(ApiError::forbidden(
            "work_order_not_claimed_by_agent",
            "work order is not claimed by this agent",
        ));
    }

    if request.success {
        let log_entry = dal
            .work_orders()
            .complete_success(id, request.message)
            .map_err(|e| {
                error!("Failed to complete work order {}: {:?}", id, e);
                ApiError::internal("failed to complete work order")
            })?;
        info!("Successfully completed work order {} with success", id);
        audit::log_action(
            ACTOR_TYPE_AGENT,
            auth_payload.agent,
            ACTION_WORKORDER_COMPLETED,
            RESOURCE_TYPE_WORKORDER,
            Some(id),
            None,
            None,
            None,
        );
        Ok((StatusCode::OK, Json(log_entry)).into_response())
    } else {
        let error_message = request
            .message
            .unwrap_or_else(|| "Unknown error".to_string());
        match dal
            .work_orders()
            .complete_failure(id, error_message, request.retryable)
            .map_err(|e| {
                error!("Failed to complete work order {} with failure: {:?}", id, e);
                ApiError::internal("failed to complete work order")
            })? {
            Some(log_entry) => {
                if request.retryable {
                    info!(
                        "Work order {} failed and exceeded max retries, moved to log",
                        id
                    );
                } else {
                    info!(
                        "Work order {} failed with non-retryable error, moved to log",
                        id
                    );
                }
                audit::log_action(
                    ACTOR_TYPE_AGENT,
                    auth_payload.agent,
                    ACTION_WORKORDER_FAILED,
                    RESOURCE_TYPE_WORKORDER,
                    Some(id),
                    Some(serde_json::json!({ "retryable": request.retryable })),
                    None,
                    None,
                );
                Ok((StatusCode::OK, Json(log_entry)).into_response())
            }
            None => {
                info!("Work order {} failed and scheduled for retry", id);
                audit::log_action(
                    ACTOR_TYPE_AGENT,
                    auth_payload.agent,
                    ACTION_WORKORDER_RETRY,
                    RESOURCE_TYPE_WORKORDER,
                    Some(id),
                    None,
                    None,
                    None,
                );
                Ok((
                    StatusCode::ACCEPTED,
                    Json(serde_json::json!({"status": "retry_scheduled"})),
                )
                    .into_response())
            }
        }
    }
}

// =============================================================================
// WORK ORDER LOG
// =============================================================================

#[utoipa::path(
    get,
    path = "/work-order-log",
    tag = "work-orders",
    params(
        ("work_type" = Option<String>, Query, description = "Filter by work type"),
        ("success" = Option<bool>, Query, description = "Filter by success status"),
        ("agent_id" = Option<Uuid>, Query, description = "Filter by agent ID"),
        ("limit" = Option<i64>, Query, description = "Limit number of results")
    ),
    responses(
        (status = 200, description = "List of completed work orders", body = Vec<WorkOrderLog>),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn list_work_order_log(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(params): Query<ListLogQuery>,
) -> Result<Json<Vec<WorkOrderLog>>, ApiError> {
    info!("Handling request to list work order log");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list work order log");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let log_entries = dal
        .work_orders()
        .list_log(
            params.work_type.as_deref(),
            params.success,
            params.agent_id,
            params.limit,
        )
        .map_err(|e| {
            error!("Failed to fetch work order log: {:?}", e);
            ApiError::internal("failed to fetch work order log")
        })?;
    info!(
        "Successfully retrieved {} work order log entries",
        log_entries.len()
    );
    Ok(Json(log_entries))
}

#[utoipa::path(
    get,
    path = "/work-order-log/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order log ID")),
    responses(
        (status = 200, description = "Work order log entry found", body = WorkOrderLog),
        (status = 403, description = "Forbidden - requires admin PAK", body = ErrorResponse),
        (status = 404, description = "Work order log entry not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn get_work_order_log(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkOrderLog>, ApiError> {
    info!(
        "Handling request to get work order log entry with ID: {}",
        id
    );
    if !auth_payload.admin {
        warn!("Unauthorized attempt to get work order log entry");
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let log_entry = dal
        .work_orders()
        .get_log(id)
        .map_err(|e| {
            error!(
                "Failed to fetch work order log entry with ID {}: {:?}",
                id, e
            );
            ApiError::internal("failed to fetch work order log entry")
        })?
        .ok_or_else(|| {
            warn!("Work order log entry not found with ID: {}", id);
            ApiError::not_found(
                "work_order_log_entry_not_found",
                "work order log entry not found",
            )
        })?;
    info!(
        "Successfully retrieved work order log entry with ID: {}",
        id
    );
    Ok(Json(log_entry))
}
