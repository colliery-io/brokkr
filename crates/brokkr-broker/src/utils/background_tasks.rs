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
use brokkr_utils::logging::prelude::*;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DiagnosticCleanupConfig::default();
        assert_eq!(config.interval_seconds, 900);
        assert_eq!(config.max_age_hours, 1);
    }
}
