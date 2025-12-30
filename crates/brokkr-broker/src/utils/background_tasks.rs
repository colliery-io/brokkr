/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Background tasks for the Brokkr Broker.
//!
//! This module contains background tasks that run periodically to maintain
//! system health and cleanup expired data.

use crate::dal::DAL;
use tracing::{debug, error, info, warn};
use std::time::Duration;
use tokio::time::interval;

/// Configuration for diagnostic cleanup task.
pub struct DiagnosticCleanupConfig {
    /// How often to run the cleanup (in seconds).
    pub interval_seconds: u64,
    /// Maximum age for completed/expired diagnostics before deletion (in hours).
    pub max_age_hours: i64,
}

impl Default for DiagnosticCleanupConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 900, // 15 minutes
            max_age_hours: 1,      // 1 hour
        }
    }
}

/// Starts the diagnostic cleanup background task.
///
/// This task periodically:
/// 1. Expires pending diagnostic requests that have passed their expiry time
/// 2. Deletes old completed/expired/failed diagnostic requests and their results
///
/// # Arguments
/// * `dal` - The Data Access Layer instance
/// * `config` - Configuration for the cleanup task
pub fn start_diagnostic_cleanup_task(dal: DAL, config: DiagnosticCleanupConfig) {
    info!(
        "Starting diagnostic cleanup task (interval: {}s, max_age: {}h)",
        config.interval_seconds, config.max_age_hours
    );

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_seconds));

        loop {
            ticker.tick().await;

            // Expire pending requests that have passed their expiry time
            match dal.diagnostic_requests().expire_old_requests() {
                Ok(expired) => {
                    if expired > 0 {
                        info!("Expired {} pending diagnostic requests", expired);
                    }
                }
                Err(e) => {
                    error!("Failed to expire diagnostic requests: {:?}", e);
                }
            }

            // Delete old completed/expired/failed requests (cascades to results)
            match dal
                .diagnostic_requests()
                .cleanup_old_requests(config.max_age_hours)
            {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(
                            "Cleaned up {} old diagnostic requests (age > {}h)",
                            deleted, config.max_age_hours
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup old diagnostic requests: {:?}", e);
                }
            }
        }
    });
}

/// Configuration for work order maintenance task.
pub struct WorkOrderMaintenanceConfig {
    /// How often to run the maintenance (in seconds).
    pub interval_seconds: u64,
}

impl Default for WorkOrderMaintenanceConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 10, // Check every 10 seconds
        }
    }
}

/// Starts the work order maintenance background task.
///
/// This task periodically:
/// 1. Moves RETRY_PENDING work orders back to PENDING when their backoff has elapsed
/// 2. Reclaims stale CLAIMED work orders that have timed out
///
/// # Arguments
/// * `dal` - The Data Access Layer instance
/// * `config` - Configuration for the maintenance task
pub fn start_work_order_maintenance_task(dal: DAL, config: WorkOrderMaintenanceConfig) {
    info!(
        "Starting work order maintenance task (interval: {}s)",
        config.interval_seconds
    );

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_seconds));

        loop {
            ticker.tick().await;

            // Process RETRY_PENDING work orders whose backoff has elapsed
            match dal.work_orders().process_retry_pending() {
                Ok(count) => {
                    if count > 0 {
                        info!("Reset {} RETRY_PENDING work orders to PENDING", count);
                    }
                }
                Err(e) => {
                    error!("Failed to process retry pending work orders: {:?}", e);
                }
            }

            // Reclaim stale CLAIMED work orders
            match dal.work_orders().process_stale_claims() {
                Ok(count) => {
                    if count > 0 {
                        info!("Released {} stale claimed work orders", count);
                    }
                }
                Err(e) => {
                    error!("Failed to process stale claims: {:?}", e);
                }
            }
        }
    });
}

/// Configuration for webhook delivery worker.
pub struct WebhookDeliveryConfig {
    /// How often to poll for pending deliveries (in seconds).
    pub interval_seconds: u64,
    /// Maximum number of deliveries to process per interval.
    pub batch_size: i64,
}

impl Default for WebhookDeliveryConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 5,  // Poll every 5 seconds
            batch_size: 50,       // Process up to 50 deliveries per batch
        }
    }
}

/// Configuration for webhook cleanup task.
pub struct WebhookCleanupConfig {
    /// How often to run the cleanup (in seconds).
    pub interval_seconds: u64,
    /// Number of days to retain completed/dead deliveries.
    pub retention_days: i64,
}

impl Default for WebhookCleanupConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 3600, // Every hour
            retention_days: 7,      // Keep for 7 days
        }
    }
}

/// Starts the webhook delivery worker background task.
///
/// This task periodically:
/// 1. Fetches pending deliveries that are ready to be sent
/// 2. Attempts to deliver each via HTTP POST
/// 3. Marks deliveries as success or failure (with retry scheduling)
///
/// # Arguments
/// * `dal` - The Data Access Layer instance
/// * `config` - Configuration for the delivery worker
pub fn start_webhook_delivery_task(dal: DAL, config: WebhookDeliveryConfig) {
    info!(
        "Starting webhook delivery worker (interval: {}s, batch_size: {})",
        config.interval_seconds, config.batch_size
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_seconds));

        loop {
            ticker.tick().await;

            // Fetch pending deliveries
            let deliveries = match dal.webhook_deliveries().get_pending(config.batch_size) {
                Ok(d) => d,
                Err(e) => {
                    error!("Failed to fetch pending webhook deliveries: {:?}", e);
                    continue;
                }
            };

            if deliveries.is_empty() {
                continue;
            }

            debug!("Processing {} pending webhook deliveries", deliveries.len());

            for delivery in deliveries {
                // Get the subscription to retrieve URL and auth header
                let subscription = match dal.webhook_subscriptions().get(delivery.subscription_id) {
                    Ok(Some(sub)) => sub,
                    Ok(None) => {
                        warn!(
                            "Subscription {} not found for delivery {}, marking as dead",
                            delivery.subscription_id, delivery.id
                        );
                        let _ = dal.webhook_deliveries().mark_failed(
                            delivery.id,
                            "Subscription not found",
                            0, // Force dead
                        );
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Failed to get subscription {} for delivery {}: {:?}",
                            delivery.subscription_id, delivery.id, e
                        );
                        continue;
                    }
                };

                // Decrypt URL and auth header
                let url = match super::encryption::decrypt_string(&subscription.url_encrypted) {
                    Ok(u) => u,
                    Err(e) => {
                        error!(
                            "Failed to decrypt URL for subscription {}: {}",
                            subscription.id, e
                        );
                        let _ = dal.webhook_deliveries().mark_failed(
                            delivery.id,
                            &format!("Failed to decrypt URL: {}", e),
                            0,
                        );
                        continue;
                    }
                };

                let auth_header = subscription
                    .auth_header_encrypted
                    .as_ref()
                    .map(|encrypted| super::encryption::decrypt_string(encrypted))
                    .transpose();

                let auth_header = match auth_header {
                    Ok(h) => h,
                    Err(e) => {
                        error!(
                            "Failed to decrypt auth header for subscription {}: {}",
                            subscription.id, e
                        );
                        let _ = dal.webhook_deliveries().mark_failed(
                            delivery.id,
                            &format!("Failed to decrypt auth header: {}", e),
                            0,
                        );
                        continue;
                    }
                };

                // Attempt delivery
                let result = attempt_delivery(&client, &url, auth_header.as_deref(), &delivery.payload).await;

                match result {
                    Ok(_) => {
                        match dal.webhook_deliveries().mark_success(delivery.id) {
                            Ok(_) => {
                                debug!(
                                    "Webhook delivery {} succeeded for subscription {}",
                                    delivery.id, subscription.id
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to mark delivery {} as success: {:?}",
                                    delivery.id, e
                                );
                            }
                        }
                    }
                    Err(error) => {
                        match dal.webhook_deliveries().mark_failed(
                            delivery.id,
                            &error,
                            subscription.max_retries,
                        ) {
                            Ok(updated) => {
                                if updated.status == "dead" {
                                    warn!(
                                        "Webhook delivery {} dead after {} attempts: {}",
                                        delivery.id, updated.attempts, error
                                    );
                                } else {
                                    debug!(
                                        "Webhook delivery {} failed (attempt {}), will retry: {}",
                                        delivery.id, updated.attempts, error
                                    );
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Failed to mark delivery {} as failed: {:?}",
                                    delivery.id, e
                                );
                            }
                        }
                    }
                }
            }
        }
    });
}

/// Attempts to deliver a webhook payload via HTTP POST.
async fn attempt_delivery(
    client: &reqwest::Client,
    url: &str,
    auth_header: Option<&str>,
    payload: &str,
) -> Result<(), String> {
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(payload.to_string());

    if let Some(auth) = auth_header {
        request = request.header("Authorization", auth);
    }

    let response = request.send().await.map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("HTTP {}: {}", status, body.chars().take(200).collect::<String>()))
    }
}


/// Starts the webhook cleanup background task.
///
/// This task periodically deletes old completed/dead deliveries
/// based on the retention policy.
///
/// # Arguments
/// * `dal` - The Data Access Layer instance
/// * `config` - Configuration for the cleanup task
pub fn start_webhook_cleanup_task(dal: DAL, config: WebhookCleanupConfig) {
    info!(
        "Starting webhook cleanup task (interval: {}s, retention: {}d)",
        config.interval_seconds, config.retention_days
    );

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_seconds));

        loop {
            ticker.tick().await;

            match dal.webhook_deliveries().cleanup_old(config.retention_days) {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(
                            "Cleaned up {} old webhook deliveries (age > {}d)",
                            deleted, config.retention_days
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup old webhook deliveries: {:?}", e);
                }
            }
        }
    });
}

/// Configuration for audit log cleanup task.
pub struct AuditLogCleanupConfig {
    /// How often to run the cleanup (in seconds).
    pub interval_seconds: u64,
    /// Number of days to retain audit logs.
    pub retention_days: i64,
}

impl Default for AuditLogCleanupConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 86400, // Daily
            retention_days: 90,      // 90 days default
        }
    }
}

/// Starts the audit log cleanup background task.
///
/// This task periodically deletes old audit log entries based on
/// the configured retention policy.
///
/// # Arguments
/// * `dal` - The Data Access Layer instance
/// * `config` - Configuration for the cleanup task
pub fn start_audit_log_cleanup_task(dal: DAL, config: AuditLogCleanupConfig) {
    info!(
        "Starting audit log cleanup task (interval: {}s, retention: {}d)",
        config.interval_seconds, config.retention_days
    );

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_seconds));

        loop {
            ticker.tick().await;

            match dal.audit_logs().cleanup_old_logs(config.retention_days) {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(
                            "Cleaned up {} old audit logs (age > {}d)",
                            deleted, config.retention_days
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup old audit logs: {:?}", e);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_diagnostic_config() {
        let config = DiagnosticCleanupConfig::default();
        assert_eq!(config.interval_seconds, 900);
        assert_eq!(config.max_age_hours, 1);
    }

    #[test]
    fn test_custom_diagnostic_config() {
        let config = DiagnosticCleanupConfig {
            interval_seconds: 60,
            max_age_hours: 24,
        };
        assert_eq!(config.interval_seconds, 60);
        assert_eq!(config.max_age_hours, 24);
    }

    #[test]
    fn test_default_work_order_config() {
        let config = WorkOrderMaintenanceConfig::default();
        assert_eq!(config.interval_seconds, 10);
    }

    #[test]
    fn test_custom_work_order_config() {
        let config = WorkOrderMaintenanceConfig {
            interval_seconds: 30,
        };
        assert_eq!(config.interval_seconds, 30);
    }

    #[test]
    fn test_default_webhook_delivery_config() {
        let config = WebhookDeliveryConfig::default();
        assert_eq!(config.interval_seconds, 5);
        assert_eq!(config.batch_size, 50);
    }

    #[test]
    fn test_custom_webhook_delivery_config() {
        let config = WebhookDeliveryConfig {
            interval_seconds: 10,
            batch_size: 100,
        };
        assert_eq!(config.interval_seconds, 10);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_default_webhook_cleanup_config() {
        let config = WebhookCleanupConfig::default();
        assert_eq!(config.interval_seconds, 3600);
        assert_eq!(config.retention_days, 7);
    }

    #[test]
    fn test_custom_webhook_cleanup_config() {
        let config = WebhookCleanupConfig {
            interval_seconds: 7200,
            retention_days: 30,
        };
        assert_eq!(config.interval_seconds, 7200);
        assert_eq!(config.retention_days, 30);
    }

    #[tokio::test]
    async fn test_attempt_delivery_invalid_url() {
        let client = reqwest::Client::new();
        let result = attempt_delivery(
            &client,
            "http://invalid.invalid.invalid:12345/webhook",
            None,
            r#"{"test": "data"}"#,
        )
        .await;

        // Should fail with a connection error
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Request failed"));
    }

    #[tokio::test]
    async fn test_attempt_delivery_with_auth_header_invalid_url() {
        let client = reqwest::Client::new();
        let result = attempt_delivery(
            &client,
            "http://invalid.invalid.invalid:12345/webhook",
            Some("Bearer test-token"),
            r#"{"event": "test"}"#,
        )
        .await;

        // Should fail, but auth header should be accepted without panic
        assert!(result.is_err());
    }
}
