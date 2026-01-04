/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Admin API endpoints for the Brokkr broker.
//!
//! This module provides administrative endpoints for managing the broker,
//! including configuration hot-reload functionality.

use super::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use brokkr_models::models::audit_logs::{AuditLog, AuditLogFilter};
use brokkr_utils::config::ReloadableConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Response structure for configuration reload operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConfigReloadResponse {
    /// Timestamp when the configuration was reloaded.
    pub reloaded_at: DateTime<Utc>,
    /// List of configuration changes detected during reload.
    pub changes: Vec<ConfigChangeInfo>,
    /// Indicates whether the reload was successful.
    pub success: bool,
    /// Optional message providing additional context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Information about a single configuration change.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConfigChangeInfo {
    /// The configuration key that changed.
    pub key: String,
    /// The previous value (as a string representation).
    pub old_value: String,
    /// The new value (as a string representation).
    pub new_value: String,
}

/// Query parameters for listing audit logs.
#[derive(Debug, Deserialize, IntoParams)]
pub struct AuditLogQueryParams {
    /// Filter by actor type (admin, agent, generator, system).
    #[param(example = "admin")]
    pub actor_type: Option<String>,
    /// Filter by actor ID.
    pub actor_id: Option<Uuid>,
    /// Filter by action (exact match or prefix with *).
    #[param(example = "agent.*")]
    pub action: Option<String>,
    /// Filter by resource type.
    #[param(example = "agent")]
    pub resource_type: Option<String>,
    /// Filter by resource ID.
    pub resource_id: Option<Uuid>,
    /// Filter by start time (inclusive, ISO 8601).
    pub from: Option<DateTime<Utc>>,
    /// Filter by end time (exclusive, ISO 8601).
    pub to: Option<DateTime<Utc>>,
    /// Maximum number of results (default 100, max 1000).
    #[param(example = 100)]
    pub limit: Option<i64>,
    /// Number of results to skip.
    #[param(example = 0)]
    pub offset: Option<i64>,
}

impl From<AuditLogQueryParams> for AuditLogFilter {
    fn from(params: AuditLogQueryParams) -> Self {
        Self {
            actor_type: params.actor_type,
            actor_id: params.actor_id,
            action: params.action,
            resource_type: params.resource_type,
            resource_id: params.resource_id,
            from: params.from,
            to: params.to,
        }
    }
}

/// Response structure for audit log list operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogListResponse {
    /// The audit log entries.
    pub logs: Vec<AuditLog>,
    /// Total count of matching entries (for pagination).
    pub total: i64,
    /// Number of entries returned.
    pub count: usize,
    /// Limit used for this query.
    pub limit: i64,
    /// Offset used for this query.
    pub offset: i64,
}

/// Constructs and returns the admin routes.
///
/// These routes require admin PAK authentication.
pub fn routes() -> Router<DAL> {
    info!("Setting up admin routes");
    Router::new()
        .route("/admin/config/reload", post(reload_config))
        .route("/admin/audit-logs", get(list_audit_logs))
}

/// Reloads the broker configuration from disk.
///
/// This endpoint triggers a hot-reload of configurable settings without
/// requiring a broker restart. Only settings marked as "dynamic" can be
/// reloaded; static settings (like database URL) require a restart.
///
/// # Authentication
///
/// Requires admin PAK authentication.
///
/// # Returns
///
/// - `200 OK`: Configuration reloaded successfully with list of changes.
/// - `401 UNAUTHORIZED`: Missing or invalid authentication.
/// - `403 FORBIDDEN`: Authenticated but not an admin.
/// - `500 INTERNAL_SERVER_ERROR`: Failed to reload configuration.
#[utoipa::path(
    post,
    path = "/api/v1/admin/config/reload",
    tag = "Admin",
    responses(
        (status = 200, description = "Configuration reloaded successfully", body = ConfigReloadResponse),
        (status = 401, description = "Missing or invalid authentication"),
        (status = 403, description = "Not authorized (admin only)"),
        (status = 500, description = "Failed to reload configuration")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn reload_config(
    Extension(auth): Extension<AuthPayload>,
    Extension(config): Extension<ReloadableConfig>,
) -> Result<impl IntoResponse, StatusCode> {
    // Verify admin authorization
    if !auth.admin {
        warn!("Non-admin attempted to reload configuration");
        return Err(StatusCode::FORBIDDEN);
    }

    info!("Admin initiated configuration reload");

    // Attempt to reload configuration
    match config.reload() {
        Ok(changes) => {
            let change_count = changes.len();
            let change_infos: Vec<ConfigChangeInfo> = changes
                .into_iter()
                .map(|c| ConfigChangeInfo {
                    key: c.key,
                    old_value: c.old_value,
                    new_value: c.new_value,
                })
                .collect();

            if change_count > 0 {
                info!(
                    "Configuration reloaded with {} change(s): {:?}",
                    change_count,
                    change_infos.iter().map(|c| &c.key).collect::<Vec<_>>()
                );
            } else {
                info!("Configuration reloaded with no changes detected");
            }

            Ok(Json(ConfigReloadResponse {
                reloaded_at: Utc::now(),
                changes: change_infos,
                success: true,
                message: if change_count > 0 {
                    Some(format!("{} setting(s) updated", change_count))
                } else {
                    Some("No changes detected".to_string())
                },
            }))
        }
        Err(e) => {
            error!("Failed to reload configuration: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Lists audit logs with optional filtering and pagination.
///
/// Returns audit log entries matching the specified filters, ordered by
/// timestamp descending (most recent first).
///
/// # Authentication
///
/// Requires admin PAK authentication.
///
/// # Query Parameters
///
/// - `actor_type`: Filter by actor type (admin, agent, generator, system).
/// - `actor_id`: Filter by actor UUID.
/// - `action`: Filter by action (exact match or prefix with *).
/// - `resource_type`: Filter by resource type.
/// - `resource_id`: Filter by resource UUID.
/// - `from`: Filter by start time (inclusive).
/// - `to`: Filter by end time (exclusive).
/// - `limit`: Maximum results (default 100, max 1000).
/// - `offset`: Number of results to skip.
///
/// # Returns
///
/// - `200 OK`: List of audit logs with pagination info.
/// - `401 UNAUTHORIZED`: Missing or invalid authentication.
/// - `403 FORBIDDEN`: Authenticated but not an admin.
/// - `500 INTERNAL_SERVER_ERROR`: Database error.
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs",
    tag = "Admin",
    params(AuditLogQueryParams),
    responses(
        (status = 200, description = "Audit logs retrieved successfully", body = AuditLogListResponse),
        (status = 401, description = "Missing or invalid authentication"),
        (status = 403, description = "Not authorized (admin only)"),
        (status = 500, description = "Database error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn list_audit_logs(
    State(dal): State<DAL>,
    Extension(auth): Extension<AuthPayload>,
    Query(params): Query<AuditLogQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Verify admin authorization
    if !auth.admin {
        warn!("Non-admin attempted to access audit logs");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    let limit = params.limit.unwrap_or(100).min(1000);
    let offset = params.offset.unwrap_or(0);
    let filter: AuditLogFilter = params.into();

    // Get total count for pagination
    let total = match dal.audit_logs().count(Some(&filter)) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count audit logs: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to query audit logs"})),
            ));
        }
    };

    // Get the logs
    let logs = match dal.audit_logs().list(Some(&filter), Some(limit), Some(offset)) {
        Ok(logs) => logs,
        Err(e) => {
            error!("Failed to list audit logs: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to query audit logs"})),
            ));
        }
    };

    let count = logs.len();

    Ok(Json(AuditLogListResponse {
        logs,
        total,
        count,
        limit,
        offset,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_reload_response_serialization() {
        let response = ConfigReloadResponse {
            reloaded_at: Utc::now(),
            changes: vec![ConfigChangeInfo {
                key: "log.level".to_string(),
                old_value: "info".to_string(),
                new_value: "debug".to_string(),
            }],
            success: true,
            message: Some("1 setting(s) updated".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("log.level"));
        assert!(json.contains("info"));
        assert!(json.contains("debug"));
    }

    #[test]
    fn test_config_change_info_serialization() {
        let change = ConfigChangeInfo {
            key: "broker.webhook_delivery_interval_seconds".to_string(),
            old_value: "5".to_string(),
            new_value: "10".to_string(),
        };

        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("webhook_delivery_interval_seconds"));
    }
}
