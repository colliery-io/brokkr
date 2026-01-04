/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Async Audit Logger for Brokkr.
//!
//! This module provides an asynchronous audit logging service that buffers
//! audit entries and writes them to the database in batches to avoid impacting
//! request latency.
//!
//! # Usage
//!
//! ```rust,ignore
//! use brokkr_broker::utils::audit;
//!
//! // Initialize during startup
//! audit::init_audit_logger(dal)?;
//!
//! // Log an audit entry (non-blocking)
//! audit::log(NewAuditLog::new(
//!     ACTOR_TYPE_ADMIN,
//!     Some(admin_id),
//!     ACTION_AGENT_CREATED,
//!     RESOURCE_TYPE_AGENT,
//!     Some(agent_id),
//! )?);
//! ```

use crate::dal::DAL;
use brokkr_models::models::audit_logs::NewAuditLog;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Default channel buffer size for audit entries.
const DEFAULT_CHANNEL_SIZE: usize = 10000;

/// Default batch size for writing to database.
const DEFAULT_BATCH_SIZE: usize = 100;

/// Default flush interval in milliseconds.
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 1000;

/// Global audit logger storage.
static AUDIT_LOGGER: OnceCell<Arc<AuditLogger>> = OnceCell::new();

/// Configuration for the audit logger.
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    /// Channel buffer size.
    pub channel_size: usize,
    /// Maximum batch size for writes.
    pub batch_size: usize,
    /// Flush interval in milliseconds.
    pub flush_interval_ms: u64,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            channel_size: DEFAULT_CHANNEL_SIZE,
            batch_size: DEFAULT_BATCH_SIZE,
            flush_interval_ms: DEFAULT_FLUSH_INTERVAL_MS,
        }
    }
}

/// The async audit logger for buffering and batching audit entries.
#[derive(Clone)]
pub struct AuditLogger {
    /// Sender for emitting audit entries.
    sender: mpsc::Sender<NewAuditLog>,
}

impl AuditLogger {
    /// Creates a new audit logger and starts the background writer.
    ///
    /// # Arguments
    /// * `dal` - The Data Access Layer for database operations.
    ///
    /// # Returns
    /// An AuditLogger instance.
    pub fn new(dal: DAL) -> Self {
        Self::with_config(dal, AuditLoggerConfig::default())
    }

    /// Creates a new audit logger with custom configuration.
    ///
    /// # Arguments
    /// * `dal` - The Data Access Layer for database operations.
    /// * `config` - The logger configuration.
    ///
    /// # Returns
    /// An AuditLogger instance.
    pub fn with_config(dal: DAL, config: AuditLoggerConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.channel_size);

        // Start the background writer task
        start_audit_writer(dal, receiver, config.batch_size, config.flush_interval_ms);

        info!(
            "Audit logger started (buffer: {}, batch: {}, flush: {}ms)",
            config.channel_size, config.batch_size, config.flush_interval_ms
        );

        Self { sender }
    }

    /// Logs an audit entry asynchronously (non-blocking).
    ///
    /// If the channel is full, the entry will be dropped and an error logged.
    ///
    /// # Arguments
    /// * `entry` - The audit log entry to record.
    pub fn log(&self, entry: NewAuditLog) {
        let sender = self.sender.clone();
        let action = entry.action.clone();

        tokio::spawn(async move {
            match sender.send(entry).await {
                Ok(_) => {
                    debug!("Audit entry queued: {}", action);
                }
                Err(e) => {
                    error!(
                        "Failed to queue audit entry (action: {}): channel full or closed - {}",
                        action, e
                    );
                }
            }
        });
    }

    /// Logs an audit entry, waiting for it to be accepted.
    ///
    /// # Arguments
    /// * `entry` - The audit log entry to record.
    ///
    /// # Returns
    /// Ok if the entry was accepted, Err if the channel is closed.
    pub async fn log_async(
        &self,
        entry: NewAuditLog,
    ) -> Result<(), mpsc::error::SendError<NewAuditLog>> {
        let action = entry.action.clone();

        self.sender.send(entry).await.map_err(|e| {
            error!("Failed to queue audit entry (action: {}): {}", action, e);
            e
        })?;

        debug!("Audit entry queued (async): {}", action);
        Ok(())
    }

    /// Tries to log an audit entry without blocking.
    ///
    /// # Arguments
    /// * `entry` - The audit log entry to record.
    ///
    /// # Returns
    /// true if the entry was queued, false if the channel is full.
    pub fn try_log(&self, entry: NewAuditLog) -> bool {
        match self.sender.try_send(entry) {
            Ok(_) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("Audit log channel full, entry dropped");
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("Audit log channel closed");
                false
            }
        }
    }
}

/// Initializes the global audit logger.
///
/// This should be called once during broker startup.
///
/// # Arguments
/// * `dal` - The Data Access Layer for database operations.
///
/// # Returns
/// Ok(()) if initialization succeeded, Err if already initialized.
pub fn init_audit_logger(dal: DAL) -> Result<(), String> {
    init_audit_logger_with_config(dal, AuditLoggerConfig::default())
}

/// Initializes the global audit logger with custom configuration.
///
/// # Arguments
/// * `dal` - The Data Access Layer for database operations.
/// * `config` - The logger configuration.
///
/// # Returns
/// Ok(()) if initialization succeeded, Err if already initialized.
pub fn init_audit_logger_with_config(dal: DAL, config: AuditLoggerConfig) -> Result<(), String> {
    let logger = AuditLogger::with_config(dal, config);
    AUDIT_LOGGER
        .set(Arc::new(logger))
        .map_err(|_| "Audit logger already initialized".to_string())
}

/// Gets the global audit logger.
///
/// # Returns
/// The audit logger, or None if not initialized.
pub fn get_audit_logger() -> Option<Arc<AuditLogger>> {
    AUDIT_LOGGER.get().cloned()
}

/// Logs an audit entry to the global audit logger.
///
/// This is a convenience function for logging without needing to get the logger directly.
///
/// # Arguments
/// * `entry` - The audit log entry to record.
pub fn log(entry: NewAuditLog) {
    if let Some(logger) = get_audit_logger() {
        logger.log(entry);
    } else {
        warn!(
            "Audit logger not initialized, entry dropped: {}",
            entry.action
        );
    }
}

/// Tries to log an audit entry without blocking.
///
/// # Arguments
/// * `entry` - The audit log entry to record.
///
/// # Returns
/// true if logged, false if channel full or logger not initialized.
pub fn try_log(entry: NewAuditLog) -> bool {
    if let Some(logger) = get_audit_logger() {
        logger.try_log(entry)
    } else {
        warn!(
            "Audit logger not initialized, entry dropped: {}",
            entry.action
        );
        false
    }
}

/// Starts the background audit writer task.
///
/// This task receives audit entries from the channel and writes them
/// to the database in batches for efficiency.
fn start_audit_writer(
    dal: DAL,
    mut receiver: mpsc::Receiver<NewAuditLog>,
    batch_size: usize,
    flush_interval_ms: u64,
) {
    tokio::spawn(async move {
        info!("Audit writer started");

        let mut buffer: Vec<NewAuditLog> = Vec::with_capacity(batch_size);
        let mut ticker = interval(Duration::from_millis(flush_interval_ms));

        loop {
            tokio::select! {
                // Receive new entries
                Some(entry) = receiver.recv() => {
                    buffer.push(entry);

                    // Flush if buffer is full
                    if buffer.len() >= batch_size {
                        flush_buffer(&dal, &mut buffer);
                    }
                }

                // Periodic flush
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        flush_buffer(&dal, &mut buffer);
                    }
                }

                // Channel closed
                else => {
                    // Final flush before shutdown
                    if !buffer.is_empty() {
                        flush_buffer(&dal, &mut buffer);
                    }
                    warn!("Audit writer stopped - channel closed");
                    break;
                }
            }
        }
    });
}

/// Flushes the buffer to the database.
fn flush_buffer(dal: &DAL, buffer: &mut Vec<NewAuditLog>) {
    if buffer.is_empty() {
        return;
    }

    let count = buffer.len();

    match dal.audit_logs().create_batch(buffer) {
        Ok(inserted) => {
            debug!("Flushed {} audit entries to database", inserted);
        }
        Err(e) => {
            error!(
                "Failed to flush {} audit entries to database: {:?}",
                count, e
            );
            // Don't lose the entries - they'll be retried on next flush
            // Actually, we should clear the buffer anyway to prevent infinite retries
            // Log the actions that failed
            for entry in buffer.iter() {
                error!("Lost audit entry: {} ({})", entry.action, entry.resource_type);
            }
        }
    }

    buffer.clear();
}

// =============================================================================
// Convenience Functions for Common Audit Actions
// =============================================================================

/// Helper to create and log an audit entry in one call.
///
/// # Arguments
/// * `actor_type` - Type of actor (admin, agent, generator, system).
/// * `actor_id` - ID of the actor.
/// * `action` - The action performed.
/// * `resource_type` - Type of resource affected.
/// * `resource_id` - ID of the affected resource.
/// * `details` - Optional additional details.
/// * `ip_address` - Optional client IP address.
/// * `user_agent` - Optional client user agent.
pub fn log_action(
    actor_type: &str,
    actor_id: Option<uuid::Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<uuid::Uuid>,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) {
    match NewAuditLog::new(actor_type, actor_id, action, resource_type, resource_id) {
        Ok(mut entry) => {
            if let Some(d) = details {
                entry = entry.with_details(d);
            }
            if let Some(ip) = ip_address {
                entry = entry.with_ip_address(ip);
            }
            if let Some(ua) = user_agent {
                entry = entry.with_user_agent(ua);
            }
            log(entry);
        }
        Err(e) => {
            error!("Failed to create audit entry: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_models::models::audit_logs::{
        ACTION_AGENT_CREATED, ACTOR_TYPE_ADMIN, RESOURCE_TYPE_AGENT,
    };
    use uuid::Uuid;

    #[test]
    fn test_audit_logger_config_default() {
        let config = AuditLoggerConfig::default();
        assert_eq!(config.channel_size, DEFAULT_CHANNEL_SIZE);
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(config.flush_interval_ms, DEFAULT_FLUSH_INTERVAL_MS);
    }

    #[test]
    fn test_log_without_logger_does_not_panic() {
        // Create a test entry
        let entry = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            Some(Uuid::new_v4()),
            ACTION_AGENT_CREATED,
            RESOURCE_TYPE_AGENT,
            Some(Uuid::new_v4()),
        )
        .unwrap();

        // Calling log when logger is not initialized should not panic
        log(entry);
    }

    #[test]
    fn test_try_log_without_logger() {
        let entry = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            Some(Uuid::new_v4()),
            ACTION_AGENT_CREATED,
            RESOURCE_TYPE_AGENT,
            Some(Uuid::new_v4()),
        )
        .unwrap();

        // Should return false when logger not initialized
        let result = try_log(entry);
        assert!(!result);
    }

    #[test]
    fn test_get_audit_logger_uninitialized() {
        // When not initialized, should return None
        // Note: In test environment, this may or may not be None depending on other tests
        let _ = get_audit_logger();
    }
}
