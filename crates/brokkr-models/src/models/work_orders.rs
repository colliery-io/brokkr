/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Work Orders Module
//!
//! This module defines structures for managing work orders in the system.
//! Work orders represent transient operations like builds, tests, or backups
//! that are routed to agents for execution.
//!
//! ## Data Model
//!
//! The work order system uses a two-table design:
//! - `work_orders`: Active queue for routing and retry management
//! - `work_order_log`: Permanent audit trail of completed work orders
//!
//! ## Status Flow
//!
//! Work orders transition through these states:
//! - `PENDING`: Ready to be claimed by an agent
//! - `CLAIMED`: Currently being processed by an agent
//! - `RETRY_PENDING`: Failed but waiting for retry backoff period
//!
//! On completion (success or max retries exceeded), records move to `work_order_log`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Valid work order statuses
pub const WORK_ORDER_STATUS_PENDING: &str = "PENDING";
pub const WORK_ORDER_STATUS_CLAIMED: &str = "CLAIMED";
pub const WORK_ORDER_STATUS_RETRY_PENDING: &str = "RETRY_PENDING";

/// Valid work types
pub const WORK_TYPE_BUILD: &str = "build";

/// Represents an active work order in the queue.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    ToSchema,
)]
#[diesel(table_name = crate::schema::work_orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[schema(example = json!({
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z",
    "work_type": "build",
    "yaml_content": "apiVersion: shipwright.io/v1beta1\nkind: Build\n...",
    "status": "PENDING",
    "claimed_by": null,
    "claimed_at": null,
    "claim_timeout_seconds": 3600,
    "max_retries": 3,
    "retry_count": 0,
    "backoff_seconds": 60,
    "next_retry_after": null,
    "last_error": null,
    "last_error_at": null
}))]
pub struct WorkOrder {
    /// Unique identifier for the work order.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,
    /// Timestamp when the work order was created.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Timestamp when the work order was last updated.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Type of work (e.g., "build", "test", "backup").
    #[schema(example = "build")]
    pub work_type: String,
    /// Multi-document YAML content (e.g., Build + WorkOrder definitions).
    #[schema(example = "apiVersion: shipwright.io/v1beta1\nkind: Build\n...")]
    pub yaml_content: String,
    /// Queue status: PENDING, CLAIMED, or RETRY_PENDING.
    #[schema(example = "PENDING")]
    pub status: String,
    /// ID of the agent that claimed this work order (if any).
    #[schema(example = "null")]
    pub claimed_by: Option<Uuid>,
    /// Timestamp when the work order was claimed.
    #[schema(example = "null")]
    pub claimed_at: Option<DateTime<Utc>>,
    /// Seconds before a claimed work order is considered stale.
    #[schema(example = 3600)]
    pub claim_timeout_seconds: i32,
    /// Maximum number of retry attempts.
    #[schema(example = 3)]
    pub max_retries: i32,
    /// Current retry count.
    #[schema(example = 0)]
    pub retry_count: i32,
    /// Base backoff seconds for exponential retry calculation.
    #[schema(example = 60)]
    pub backoff_seconds: i32,
    /// Timestamp when RETRY_PENDING work order becomes PENDING again.
    #[schema(example = "null")]
    pub next_retry_after: Option<DateTime<Utc>>,
    /// Most recent error message from failed execution attempt.
    #[schema(example = "null")]
    pub last_error: Option<String>,
    /// Timestamp of the most recent failure.
    #[schema(example = "null")]
    pub last_error_at: Option<DateTime<Utc>>,
}

/// Represents a new work order to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::work_orders)]
#[schema(example = json!({
    "work_type": "build",
    "yaml_content": "apiVersion: shipwright.io/v1beta1\nkind: Build\n...",
    "max_retries": 3,
    "backoff_seconds": 60,
    "claim_timeout_seconds": 3600
}))]
pub struct NewWorkOrder {
    /// Type of work (e.g., "build", "test", "backup").
    pub work_type: String,
    /// Multi-document YAML content.
    pub yaml_content: String,
    /// Maximum number of retry attempts.
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
    /// Base backoff seconds for exponential retry calculation.
    #[serde(default = "default_backoff_seconds")]
    pub backoff_seconds: i32,
    /// Seconds before a claimed work order is considered stale.
    #[serde(default = "default_claim_timeout_seconds")]
    pub claim_timeout_seconds: i32,
}

fn default_max_retries() -> i32 {
    3
}

fn default_backoff_seconds() -> i32 {
    60
}

fn default_claim_timeout_seconds() -> i32 {
    3600
}

impl NewWorkOrder {
    /// Creates a new `NewWorkOrder` instance with validation.
    ///
    /// # Parameters
    ///
    /// * `work_type`: Type of work (e.g., "build", "test").
    /// * `yaml_content`: Multi-document YAML content.
    /// * `max_retries`: Maximum retry attempts (optional, defaults to 3).
    /// * `backoff_seconds`: Base backoff for retries (optional, defaults to 60).
    /// * `claim_timeout_seconds`: Claim timeout (optional, defaults to 3600).
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewWorkOrder)` if valid, otherwise `Err` with validation error.
    pub fn new(
        work_type: String,
        yaml_content: String,
        max_retries: Option<i32>,
        backoff_seconds: Option<i32>,
        claim_timeout_seconds: Option<i32>,
    ) -> Result<Self, String> {
        // Validate work_type
        if work_type.trim().is_empty() {
            return Err("Work type cannot be empty".to_string());
        }

        // Validate yaml_content
        if yaml_content.trim().is_empty() {
            return Err("YAML content cannot be empty".to_string());
        }

        let max_retries = max_retries.unwrap_or(3);
        let backoff_seconds = backoff_seconds.unwrap_or(60);
        let claim_timeout_seconds = claim_timeout_seconds.unwrap_or(3600);

        if max_retries < 0 {
            return Err("max_retries must be non-negative".to_string());
        }

        if backoff_seconds <= 0 {
            return Err("backoff_seconds must be positive".to_string());
        }

        if claim_timeout_seconds <= 0 {
            return Err("claim_timeout_seconds must be positive".to_string());
        }

        Ok(NewWorkOrder {
            work_type,
            yaml_content,
            max_retries,
            backoff_seconds,
            claim_timeout_seconds,
        })
    }
}

/// Represents a completed work order in the audit log.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    ToSchema,
)]
#[diesel(table_name = crate::schema::work_order_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[schema(example = json!({
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "work_type": "build",
    "created_at": "2023-01-01T00:00:00Z",
    "claimed_at": "2023-01-01T00:01:00Z",
    "completed_at": "2023-01-01T00:05:00Z",
    "claimed_by": "123e4567-e89b-12d3-a456-426614174001",
    "success": true,
    "retries_attempted": 0,
    "result_message": "sha256:abc123...",
    "yaml_content": "apiVersion: shipwright.io/v1beta1\nkind: Build\n..."
}))]
pub struct WorkOrderLog {
    /// Original work order ID.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,
    /// Type of work.
    #[schema(example = "build")]
    pub work_type: String,
    /// Timestamp when the work order was created.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Timestamp when the work order was claimed.
    #[schema(example = "2023-01-01T00:01:00Z")]
    pub claimed_at: Option<DateTime<Utc>>,
    /// Timestamp when the work order completed.
    #[schema(example = "2023-01-01T00:05:00Z")]
    pub completed_at: DateTime<Utc>,
    /// ID of the agent that executed this work order.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174001")]
    pub claimed_by: Option<Uuid>,
    /// Whether the work completed successfully.
    #[schema(example = true)]
    pub success: bool,
    /// Number of retry attempts before completion.
    #[schema(example = 0)]
    pub retries_attempted: i32,
    /// Result message (image digest on success, error details on failure).
    #[schema(example = "sha256:abc123...")]
    pub result_message: Option<String>,
    /// Original YAML content for debugging/reconstruction.
    #[schema(example = "apiVersion: shipwright.io/v1beta1\nkind: Build\n...")]
    pub yaml_content: String,
}

/// Represents a new work order log entry to be inserted.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::work_order_log)]
pub struct NewWorkOrderLog {
    /// Original work order ID.
    pub id: Uuid,
    /// Type of work.
    pub work_type: String,
    /// Timestamp when the work order was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the work order was claimed.
    pub claimed_at: Option<DateTime<Utc>>,
    /// ID of the agent that executed this work order.
    pub claimed_by: Option<Uuid>,
    /// Whether the work completed successfully.
    pub success: bool,
    /// Number of retry attempts before completion.
    pub retries_attempted: i32,
    /// Result message.
    pub result_message: Option<String>,
    /// Original YAML content.
    pub yaml_content: String,
}

impl NewWorkOrderLog {
    /// Creates a new log entry from a completed work order.
    pub fn from_work_order(work_order: &WorkOrder, success: bool, result_message: Option<String>) -> Self {
        NewWorkOrderLog {
            id: work_order.id,
            work_type: work_order.work_type.clone(),
            created_at: work_order.created_at,
            claimed_at: work_order.claimed_at,
            claimed_by: work_order.claimed_by,
            success,
            retries_attempted: work_order.retry_count,
            result_message,
            yaml_content: work_order.yaml_content.clone(),
        }
    }
}

/// Represents a work order target (agent routing).
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    ToSchema,
)]
#[diesel(table_name = crate::schema::work_order_targets)]
#[diesel(belongs_to(WorkOrder, foreign_key = work_order_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[schema(example = json!({
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "work_order_id": "123e4567-e89b-12d3-a456-426614174001",
    "agent_id": "123e4567-e89b-12d3-a456-426614174002",
    "created_at": "2023-01-01T00:00:00Z"
}))]
pub struct WorkOrderTarget {
    /// Unique identifier for the target entry.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,
    /// ID of the work order.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174001")]
    pub work_order_id: Uuid,
    /// ID of the eligible agent.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174002")]
    pub agent_id: Uuid,
    /// Timestamp when the target was created.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Represents a new work order target to be inserted.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::work_order_targets)]
pub struct NewWorkOrderTarget {
    /// ID of the work order.
    pub work_order_id: Uuid,
    /// ID of the eligible agent.
    pub agent_id: Uuid,
}

impl NewWorkOrderTarget {
    /// Creates a new work order target.
    pub fn new(work_order_id: Uuid, agent_id: Uuid) -> Result<Self, String> {
        if work_order_id.is_nil() {
            return Err("Invalid work order ID".to_string());
        }
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }
        Ok(NewWorkOrderTarget {
            work_order_id,
            agent_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_work_order_success() {
        let result = NewWorkOrder::new(
            "build".to_string(),
            "apiVersion: v1\nkind: ConfigMap".to_string(),
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let work_order = result.unwrap();
        assert_eq!(work_order.work_type, "build");
        assert_eq!(work_order.max_retries, 3);
        assert_eq!(work_order.backoff_seconds, 60);
        assert_eq!(work_order.claim_timeout_seconds, 3600);
    }

    #[test]
    fn test_new_work_order_empty_work_type() {
        let result = NewWorkOrder::new(
            "".to_string(),
            "yaml content".to_string(),
            None,
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Work type cannot be empty");
    }

    #[test]
    fn test_new_work_order_empty_yaml() {
        let result = NewWorkOrder::new(
            "build".to_string(),
            "".to_string(),
            None,
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "YAML content cannot be empty");
    }

    #[test]
    fn test_new_work_order_invalid_max_retries() {
        let result = NewWorkOrder::new(
            "build".to_string(),
            "yaml content".to_string(),
            Some(-1),
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "max_retries must be non-negative");
    }

    #[test]
    fn test_new_work_order_target_success() {
        let result = NewWorkOrderTarget::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_work_order_target_invalid_ids() {
        let result = NewWorkOrderTarget::new(Uuid::nil(), Uuid::new_v4());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid work order ID");

        let result = NewWorkOrderTarget::new(Uuid::new_v4(), Uuid::nil());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid agent ID");
    }
}
