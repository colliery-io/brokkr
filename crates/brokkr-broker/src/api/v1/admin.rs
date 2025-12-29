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
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use brokkr_utils::config::ReloadableConfig;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{error, info, warn};
use utoipa::ToSchema;

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

/// Constructs and returns the admin routes.
///
/// These routes require admin PAK authentication.
pub fn routes() -> Router<DAL> {
    info!("Setting up admin routes");
    Router::new().route("/admin/config/reload", post(reload_config))
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
