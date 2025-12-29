/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Handles API routes and logic for work orders.
//!
//! This module provides functionality for managing work orders through their lifecycle:
//! - Creating work orders with target agents
//! - Claiming work orders by agents
//! - Completing work orders (success/failure)
//! - Querying work order history from the log
//!
//! ## Endpoints
//!
//! ### Work Order Management (Admin)
//! - `POST /api/v1/work-orders` - Create a new work order
//! - `GET /api/v1/work-orders` - List all work orders
//! - `GET /api/v1/work-orders/:id` - Get work order by ID
//! - `DELETE /api/v1/work-orders/:id` - Delete/cancel a work order
//!
//! ### Agent Operations
//! - `GET /api/v1/agents/:id/work-orders/pending` - Get claimable work orders for agent
//! - `POST /api/v1/work-orders/:id/claim` - Claim a work order
//! - `POST /api/v1/work-orders/:id/complete` - Report work order completion
//!
//! ### Work Order Log (Read-only history)
//! - `GET /api/v1/work-order-log` - List completed work orders
//! - `GET /api/v1/work-order-log/:id` - Get completed work order by ID

use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use brokkr_models::models::work_orders::{NewWorkOrder, WorkOrder, WorkOrderLog};
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Creates and returns a router for work order-related endpoints.
pub fn routes() -> Router<DAL> {
    info!("Setting up work order routes");
    Router::new()
        // Work order management
        .route("/work-orders", get(list_work_orders).post(create_work_order))
        .route(
            "/work-orders/:id",
            get(get_work_order).delete(delete_work_order),
        )
        // Claim and complete operations
        .route("/work-orders/:id/claim", post(claim_work_order))
        .route("/work-orders/:id/complete", post(complete_work_order))
        // Work order log
        .route("/work-order-log", get(list_work_order_log))
        .route("/work-order-log/:id", get(get_work_order_log))
}

/// Creates agent-specific routes for work order operations.
/// These routes are nested under /agents/:id in the main router.
pub fn agent_routes() -> Router<DAL> {
    Router::new().route(
        "/agents/:agent_id/work-orders/pending",
        get(list_pending_for_agent),
    )
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// Request body for creating a new work order.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateWorkOrderRequest {
    /// Type of work (e.g., "build", "test", "backup").
    pub work_type: String,
    /// Multi-document YAML content.
    pub yaml_content: String,
    /// Maximum number of retry attempts (default: 3).
    #[serde(default)]
    pub max_retries: Option<i32>,
    /// Base backoff seconds for exponential retry (default: 60).
    #[serde(default)]
    pub backoff_seconds: Option<i32>,
    /// Claim timeout in seconds (default: 3600).
    #[serde(default)]
    pub claim_timeout_seconds: Option<i32>,
    /// Optional targeting configuration. At least one targeting method must be specified.
    #[serde(default)]
    pub targeting: Option<WorkOrderTargeting>,
    /// DEPRECATED: Use targeting.agent_ids instead. Target agent IDs that can claim this work order.
    #[serde(default)]
    pub target_agent_ids: Option<Vec<Uuid>>,
}

/// Targeting configuration for work orders.
/// Work orders can be targeted using any combination of hard targets (agent IDs),
/// labels, or annotations. Matching uses OR logic - an agent is eligible if it
/// matches ANY of the specified targeting criteria.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct WorkOrderTargeting {
    /// Direct agent IDs that can claim this work order (hard targets).
    #[serde(default)]
    pub agent_ids: Option<Vec<Uuid>>,
    /// Labels that agents must have (OR logic - agent needs any one of these labels).
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    /// Annotations that agents must have (OR logic - agent needs any one of these key-value pairs).
    #[serde(default)]
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

/// Request body for claiming a work order.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ClaimWorkOrderRequest {
    /// ID of the agent claiming the work order.
    pub agent_id: Uuid,
}

/// Request body for completing a work order.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteWorkOrderRequest {
    /// Whether the work completed successfully.
    pub success: bool,
    /// Result message (image digest on success, error details on failure).
    pub message: Option<String>,
    /// Whether the failure is retryable. Defaults to true if not specified.
    /// Set to false for permanent failures (e.g., invalid YAML, missing namespace).
    #[serde(default = "default_retryable")]
    pub retryable: bool,
}

fn default_retryable() -> bool {
    true
}

/// Query parameters for listing work orders.
#[derive(Debug, Deserialize)]
pub struct ListWorkOrdersQuery {
    /// Filter by status (PENDING, CLAIMED, RETRY_PENDING).
    pub status: Option<String>,
    /// Filter by work type.
    pub work_type: Option<String>,
}

/// Query parameters for listing pending work orders for an agent.
#[derive(Debug, Deserialize)]
pub struct ListPendingQuery {
    /// Filter by work type.
    pub work_type: Option<String>,
}

/// Query parameters for listing work order log.
#[derive(Debug, Deserialize)]
pub struct ListLogQuery {
    /// Filter by work type.
    pub work_type: Option<String>,
    /// Filter by success status.
    pub success: Option<bool>,
    /// Filter by agent ID.
    pub agent_id: Option<Uuid>,
    /// Limit number of results.
    pub limit: Option<i64>,
}

// =============================================================================
// WORK ORDER MANAGEMENT ENDPOINTS
// =============================================================================

/// Lists all work orders.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/work-orders",
    tag = "work-orders",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("work_type" = Option<String>, Query, description = "Filter by work type")
    ),
    responses(
        (status = 200, description = "List of work orders", body = Vec<WorkOrder>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
    ),
    security(("pak" = []))
)]
async fn list_work_orders(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(params): Query<ListWorkOrdersQuery>,
) -> Result<Json<Vec<WorkOrder>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list work orders");

    if !auth_payload.admin {
        warn!("Unauthorized attempt to list work orders");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal
        .work_orders()
        .list_filtered(params.status.as_deref(), params.work_type.as_deref())
    {
        Ok(work_orders) => {
            info!("Successfully retrieved {} work orders", work_orders.len());
            Ok(Json(work_orders))
        }
        Err(e) => {
            error!("Failed to fetch work orders: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch work orders"})),
            ))
        }
    }
}

/// Creates a new work order.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/api/v1/work-orders",
    tag = "work-orders",
    request_body = CreateWorkOrderRequest,
    responses(
        (status = 201, description = "Work order created", body = WorkOrder),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
    ),
    security(("pak" = []))
)]
async fn create_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(request): Json<CreateWorkOrderRequest>,
) -> Result<(StatusCode, Json<WorkOrder>), (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create a new work order");

    if !auth_payload.admin {
        warn!("Unauthorized attempt to create work order");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    // Extract targeting from either the new targeting field or deprecated target_agent_ids
    let targeting = request.targeting.unwrap_or_default();
    let legacy_agent_ids = request.target_agent_ids.unwrap_or_default();

    // Combine agent IDs from both sources (for backwards compatibility)
    let agent_ids: Vec<Uuid> = targeting
        .agent_ids
        .unwrap_or_default()
        .into_iter()
        .chain(legacy_agent_ids)
        .collect();

    let labels = targeting.labels.unwrap_or_default();
    let annotations = targeting.annotations.unwrap_or_default();

    // Validate that at least one targeting method is specified
    if agent_ids.is_empty() && labels.is_empty() && annotations.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "At least one targeting method must be specified (agent_ids, labels, or annotations)"
            })),
        ));
    }

    // Create the work order
    let new_work_order = match NewWorkOrder::new(
        request.work_type,
        request.yaml_content,
        request.max_retries,
        request.backoff_seconds,
        request.claim_timeout_seconds,
    ) {
        Ok(wo) => wo,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            ));
        }
    };

    let work_order = match dal.work_orders().create(&new_work_order) {
        Ok(wo) => wo,
        Err(e) => {
            error!("Failed to create work order: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create work order"})),
            ));
        }
    };

    // Add hard targets (agent IDs)
    if !agent_ids.is_empty() {
        if let Err(e) = dal.work_orders().add_targets(work_order.id, &agent_ids) {
            error!("Failed to add work order targets: {:?}", e);
            let _ = dal.work_orders().delete(work_order.id);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add work order targets"})),
            ));
        }
    }

    // Add labels
    if !labels.is_empty() {
        if let Err(e) = dal.work_orders().add_labels(work_order.id, &labels) {
            error!("Failed to add work order labels: {:?}", e);
            let _ = dal.work_orders().delete(work_order.id);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add work order labels"})),
            ));
        }
    }

    // Add annotations
    if !annotations.is_empty() {
        if let Err(e) = dal
            .work_orders()
            .add_annotations(work_order.id, &annotations)
        {
            error!("Failed to add work order annotations: {:?}", e);
            let _ = dal.work_orders().delete(work_order.id);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add work order annotations"})),
            ));
        }
    }

    info!("Successfully created work order with ID: {}", work_order.id);
    Ok((StatusCode::CREATED, Json(work_order)))
}

/// Gets a work order by ID.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/work-orders/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    responses(
        (status = 200, description = "Work order found", body = WorkOrder),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Work order not found"),
    ),
    security(("pak" = []))
)]
async fn get_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkOrder>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get work order with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to get work order");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.work_orders().get(id) {
        Ok(Some(work_order)) => {
            info!("Successfully retrieved work order with ID: {}", id);
            Ok(Json(work_order))
        }
        Ok(None) => {
            warn!("Work order not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Work order not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch work order with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch work order"})),
            ))
        }
    }
}

/// Deletes/cancels a work order.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/api/v1/work-orders/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    responses(
        (status = 204, description = "Work order deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Work order not found"),
    ),
    security(("pak" = []))
)]
async fn delete_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete work order with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to delete work order");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.work_orders().delete(id) {
        Ok(0) => {
            warn!("Work order not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Work order not found"})),
            ))
        }
        Ok(_) => {
            info!("Successfully deleted work order with ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete work order with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete work order"})),
            ))
        }
    }
}

// =============================================================================
// AGENT OPERATIONS
// =============================================================================

/// Lists pending work orders claimable by a specific agent.
///
/// # Authorization
/// Requires admin privileges or agent authentication.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_id}/work-orders/pending",
    tag = "work-orders",
    params(
        ("agent_id" = Uuid, Path, description = "Agent ID"),
        ("work_type" = Option<String>, Query, description = "Filter by work type")
    ),
    responses(
        (status = 200, description = "List of pending work orders", body = Vec<WorkOrder>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("pak" = []))
)]
async fn list_pending_for_agent(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(agent_id): Path<Uuid>,
    Query(params): Query<ListPendingQuery>,
) -> Result<Json<Vec<WorkOrder>>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to list pending work orders for agent: {}",
        agent_id
    );

    // Allow admin or the agent itself
    if !auth_payload.admin && auth_payload.agent != Some(agent_id) {
        warn!("Unauthorized attempt to list pending work orders for agent");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    match dal
        .work_orders()
        .list_pending_for_agent(agent_id, params.work_type.as_deref())
    {
        Ok(work_orders) => {
            info!(
                "Successfully retrieved {} pending work orders for agent {}",
                work_orders.len(),
                agent_id
            );
            Ok(Json(work_orders))
        }
        Err(e) => {
            error!(
                "Failed to fetch pending work orders for agent {}: {:?}",
                agent_id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch pending work orders"})),
            ))
        }
    }
}

/// Claims a work order for an agent.
///
/// # Authorization
/// Requires admin privileges or agent authentication.
#[utoipa::path(
    post,
    path = "/api/v1/work-orders/{id}/claim",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    request_body = ClaimWorkOrderRequest,
    responses(
        (status = 200, description = "Work order claimed", body = WorkOrder),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Work order not found or not claimable"),
        (status = 409, description = "Work order already claimed"),
    ),
    security(("pak" = []))
)]
async fn claim_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<ClaimWorkOrderRequest>,
) -> Result<Json<WorkOrder>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to claim work order {} by agent {}",
        id, request.agent_id
    );

    // Allow admin or the agent itself
    if !auth_payload.admin && auth_payload.agent != Some(request.agent_id) {
        warn!("Unauthorized attempt to claim work order");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    match dal.work_orders().claim(id, request.agent_id) {
        Ok(work_order) => {
            info!(
                "Successfully claimed work order {} by agent {}",
                id, request.agent_id
            );
            Ok(Json(work_order))
        }
        Err(diesel::result::Error::NotFound) => {
            warn!(
                "Work order {} not found or not claimable by agent {}",
                id, request.agent_id
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Work order not found or not claimable"})),
            ))
        }
        Err(e) => {
            error!("Failed to claim work order {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to claim work order"})),
            ))
        }
    }
}

/// Completes a work order (success or failure).
///
/// On success, the work order is moved to the log.
/// On failure, if retries remain, the work order is scheduled for retry.
/// If max retries exceeded, the work order is moved to the log.
///
/// # Authorization
/// Requires admin privileges or agent authentication.
#[utoipa::path(
    post,
    path = "/api/v1/work-orders/{id}/complete",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order ID")),
    request_body = CompleteWorkOrderRequest,
    responses(
        (status = 200, description = "Work order completed", body = WorkOrderLog),
        (status = 202, description = "Work order scheduled for retry"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Work order not found"),
    ),
    security(("pak" = []))
)]
async fn complete_work_order(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(request): Json<CompleteWorkOrderRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling request to complete work order {} (success: {})",
        id, request.success
    );

    // Get the work order to verify authorization
    let work_order = match dal.work_orders().get(id) {
        Ok(Some(wo)) => wo,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Work order not found"})),
            ));
        }
        Err(e) => {
            error!("Failed to fetch work order {}: {:?}", id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch work order"})),
            ));
        }
    };

    // Allow admin or the agent that claimed the work order
    if !auth_payload.admin && auth_payload.agent != work_order.claimed_by {
        warn!("Unauthorized attempt to complete work order");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    if request.success {
        match dal.work_orders().complete_success(id, request.message) {
            Ok(log_entry) => {
                info!("Successfully completed work order {} with success", id);
                Ok((StatusCode::OK, Json(serde_json::json!(log_entry))))
            }
            Err(e) => {
                error!("Failed to complete work order {}: {:?}", id, e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to complete work order"})),
                ))
            }
        }
    } else {
        let error_message = request.message.unwrap_or_else(|| "Unknown error".to_string());
        match dal
            .work_orders()
            .complete_failure(id, error_message, request.retryable)
        {
            Ok(Some(log_entry)) => {
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
                Ok((StatusCode::OK, Json(serde_json::json!(log_entry))))
            }
            Ok(None) => {
                info!("Work order {} failed and scheduled for retry", id);
                Ok((
                    StatusCode::ACCEPTED,
                    Json(serde_json::json!({"status": "retry_scheduled"})),
                ))
            }
            Err(e) => {
                error!("Failed to complete work order {} with failure: {:?}", id, e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to complete work order"})),
                ))
            }
        }
    }
}

// =============================================================================
// WORK ORDER LOG ENDPOINTS
// =============================================================================

/// Lists completed work orders from the log.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/work-order-log",
    tag = "work-orders",
    params(
        ("work_type" = Option<String>, Query, description = "Filter by work type"),
        ("success" = Option<bool>, Query, description = "Filter by success status"),
        ("agent_id" = Option<Uuid>, Query, description = "Filter by agent ID"),
        ("limit" = Option<i64>, Query, description = "Limit number of results")
    ),
    responses(
        (status = 200, description = "List of completed work orders", body = Vec<WorkOrderLog>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
    ),
    security(("pak" = []))
)]
async fn list_work_order_log(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Query(params): Query<ListLogQuery>,
) -> Result<Json<Vec<WorkOrderLog>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list work order log");

    if !auth_payload.admin {
        warn!("Unauthorized attempt to list work order log");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.work_orders().list_log(
        params.work_type.as_deref(),
        params.success,
        params.agent_id,
        params.limit,
    ) {
        Ok(log_entries) => {
            info!(
                "Successfully retrieved {} work order log entries",
                log_entries.len()
            );
            Ok(Json(log_entries))
        }
        Err(e) => {
            error!("Failed to fetch work order log: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch work order log"})),
            ))
        }
    }
}

/// Gets a completed work order from the log by ID.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/work-order-log/{id}",
    tag = "work-orders",
    params(("id" = Uuid, Path, description = "Work order log ID")),
    responses(
        (status = 200, description = "Work order log entry found", body = WorkOrderLog),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Work order log entry not found"),
    ),
    security(("pak" = []))
)]
async fn get_work_order_log(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkOrderLog>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get work order log entry with ID: {}", id);

    if !auth_payload.admin {
        warn!("Unauthorized attempt to get work order log entry");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.work_orders().get_log(id) {
        Ok(Some(log_entry)) => {
            info!("Successfully retrieved work order log entry with ID: {}", id);
            Ok(Json(log_entry))
        }
        Ok(None) => {
            warn!("Work order log entry not found with ID: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Work order log entry not found"})),
            ))
        }
        Err(e) => {
            error!(
                "Failed to fetch work order log entry with ID {}: {:?}",
                id, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch work order log entry"})),
            ))
        }
    }
}
