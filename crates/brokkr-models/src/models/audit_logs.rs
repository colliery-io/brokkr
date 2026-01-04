/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Audit log models for tracking administrative and security-sensitive operations.
//!
//! Audit logs are immutable records that track who did what to which resource.
//! They are used for compliance, debugging, and security incident investigation.

use crate::schema::audit_logs;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Constants
// =============================================================================

/// Actor type for admin users.
pub const ACTOR_TYPE_ADMIN: &str = "admin";
/// Actor type for agents.
pub const ACTOR_TYPE_AGENT: &str = "agent";
/// Actor type for generators.
pub const ACTOR_TYPE_GENERATOR: &str = "generator";
/// Actor type for system operations.
pub const ACTOR_TYPE_SYSTEM: &str = "system";

pub const VALID_ACTOR_TYPES: &[&str] = &[
    ACTOR_TYPE_ADMIN,
    ACTOR_TYPE_AGENT,
    ACTOR_TYPE_GENERATOR,
    ACTOR_TYPE_SYSTEM,
];

// Action constants - Authentication
pub const ACTION_PAK_CREATED: &str = "pak.created";
pub const ACTION_PAK_ROTATED: &str = "pak.rotated";
pub const ACTION_PAK_DELETED: &str = "pak.deleted";
pub const ACTION_AUTH_FAILED: &str = "auth.failed";
pub const ACTION_AUTH_SUCCESS: &str = "auth.success";

// Action constants - Resource Management
pub const ACTION_AGENT_CREATED: &str = "agent.created";
pub const ACTION_AGENT_UPDATED: &str = "agent.updated";
pub const ACTION_AGENT_DELETED: &str = "agent.deleted";
pub const ACTION_STACK_CREATED: &str = "stack.created";
pub const ACTION_STACK_UPDATED: &str = "stack.updated";
pub const ACTION_STACK_DELETED: &str = "stack.deleted";
pub const ACTION_GENERATOR_CREATED: &str = "generator.created";
pub const ACTION_GENERATOR_UPDATED: &str = "generator.updated";
pub const ACTION_GENERATOR_DELETED: &str = "generator.deleted";
pub const ACTION_TEMPLATE_CREATED: &str = "template.created";
pub const ACTION_TEMPLATE_UPDATED: &str = "template.updated";
pub const ACTION_TEMPLATE_DELETED: &str = "template.deleted";

// Action constants - Webhooks
pub const ACTION_WEBHOOK_CREATED: &str = "webhook.created";
pub const ACTION_WEBHOOK_UPDATED: &str = "webhook.updated";
pub const ACTION_WEBHOOK_DELETED: &str = "webhook.deleted";
pub const ACTION_WEBHOOK_DELIVERY_FAILED: &str = "webhook.delivery_failed";

// Action constants - Work Orders
pub const ACTION_WORKORDER_CREATED: &str = "workorder.created";
pub const ACTION_WORKORDER_CLAIMED: &str = "workorder.claimed";
pub const ACTION_WORKORDER_COMPLETED: &str = "workorder.completed";
pub const ACTION_WORKORDER_FAILED: &str = "workorder.failed";
pub const ACTION_WORKORDER_RETRY: &str = "workorder.retry";

// Action constants - Admin
pub const ACTION_CONFIG_RELOADED: &str = "config.reloaded";

// Resource type constants
pub const RESOURCE_TYPE_AGENT: &str = "agent";
pub const RESOURCE_TYPE_STACK: &str = "stack";
pub const RESOURCE_TYPE_GENERATOR: &str = "generator";
pub const RESOURCE_TYPE_TEMPLATE: &str = "template";
pub const RESOURCE_TYPE_WEBHOOK: &str = "webhook_subscription";
pub const RESOURCE_TYPE_WORKORDER: &str = "work_order";
pub const RESOURCE_TYPE_PAK: &str = "pak";
pub const RESOURCE_TYPE_CONFIG: &str = "config";
pub const RESOURCE_TYPE_SYSTEM: &str = "system";

// =============================================================================
// Audit Log Models
// =============================================================================

/// An audit log record from the database.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = audit_logs)]
pub struct AuditLog {
    /// Unique identifier for the log entry.
    pub id: Uuid,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Type of actor: admin, agent, generator, system.
    pub actor_type: String,
    /// ID of the actor (NULL for system or unauthenticated).
    pub actor_id: Option<Uuid>,
    /// The action performed (e.g., "agent.created", "auth.failed").
    pub action: String,
    /// Type of resource affected.
    pub resource_type: String,
    /// ID of the affected resource (NULL if not applicable).
    pub resource_id: Option<Uuid>,
    /// Additional structured details.
    #[schema(value_type = Option<serde_json::Value>)]
    pub details: Option<serde_json::Value>,
    /// Client IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Client user agent string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// When the record was created.
    pub created_at: DateTime<Utc>,
}

/// A new audit log entry to be inserted.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog {
    /// Type of actor.
    pub actor_type: String,
    /// ID of the actor.
    pub actor_id: Option<Uuid>,
    /// The action performed.
    pub action: String,
    /// Type of resource affected.
    pub resource_type: String,
    /// ID of the affected resource.
    pub resource_id: Option<Uuid>,
    /// Additional structured details.
    pub details: Option<serde_json::Value>,
    /// Client IP address.
    pub ip_address: Option<String>,
    /// Client user agent string.
    pub user_agent: Option<String>,
}

impl NewAuditLog {
    /// Creates a new audit log entry.
    ///
    /// # Arguments
    /// * `actor_type` - Type of actor (admin, agent, generator, system).
    /// * `actor_id` - ID of the actor (None for system).
    /// * `action` - The action performed.
    /// * `resource_type` - Type of resource affected.
    /// * `resource_id` - ID of the affected resource (None if not applicable).
    pub fn new(
        actor_type: &str,
        actor_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
    ) -> Result<Self, String> {
        // Validate actor type
        if !VALID_ACTOR_TYPES.contains(&actor_type) {
            return Err(format!(
                "Invalid actor_type '{}'. Must be one of: {:?}",
                actor_type, VALID_ACTOR_TYPES
            ));
        }

        // Validate action is not empty
        if action.trim().is_empty() {
            return Err("Action cannot be empty".to_string());
        }

        // Validate resource_type is not empty
        if resource_type.trim().is_empty() {
            return Err("Resource type cannot be empty".to_string());
        }

        Ok(Self {
            actor_type: actor_type.to_string(),
            actor_id,
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id,
            details: None,
            ip_address: None,
            user_agent: None,
        })
    }

    /// Adds details to the audit log entry.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Adds client IP address to the audit log entry.
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Adds user agent to the audit log entry.
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

// =============================================================================
// Query Filters
// =============================================================================

/// Filters for querying audit logs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AuditLogFilter {
    /// Filter by actor type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_type: Option<String>,
    /// Filter by actor ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<Uuid>,
    /// Filter by action (exact match or prefix with *).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Filter by resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// Filter by resource ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<Uuid>,
    /// Filter by start time (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<DateTime<Utc>>,
    /// Filter by end time (exclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<DateTime<Utc>>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_audit_log_success() {
        let result = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            Some(Uuid::new_v4()),
            ACTION_AGENT_CREATED,
            RESOURCE_TYPE_AGENT,
            Some(Uuid::new_v4()),
        );

        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.actor_type, ACTOR_TYPE_ADMIN);
        assert_eq!(log.action, ACTION_AGENT_CREATED);
    }

    #[test]
    fn test_new_audit_log_invalid_actor_type() {
        let result = NewAuditLog::new(
            "invalid",
            None,
            ACTION_AGENT_CREATED,
            RESOURCE_TYPE_AGENT,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid actor_type"));
    }

    #[test]
    fn test_new_audit_log_empty_action() {
        let result = NewAuditLog::new(ACTOR_TYPE_SYSTEM, None, "", RESOURCE_TYPE_SYSTEM, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Action cannot be empty"));
    }

    #[test]
    fn test_audit_log_with_details() {
        let log = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            Some(Uuid::new_v4()),
            ACTION_AGENT_CREATED,
            RESOURCE_TYPE_AGENT,
            Some(Uuid::new_v4()),
        )
        .unwrap()
        .with_details(serde_json::json!({"agent_name": "test-agent"}));

        assert!(log.details.is_some());
        assert_eq!(log.details.unwrap()["agent_name"], "test-agent");
    }

    #[test]
    fn test_audit_log_with_ip_address() {
        let log = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            None,
            ACTION_AUTH_SUCCESS,
            RESOURCE_TYPE_SYSTEM,
            None,
        )
        .unwrap()
        .with_ip_address("192.168.1.100");

        assert!(log.ip_address.is_some());
        assert_eq!(log.ip_address.unwrap(), "192.168.1.100");
    }

    #[test]
    fn test_audit_log_system_action() {
        let result = NewAuditLog::new(
            ACTOR_TYPE_SYSTEM,
            None,
            ACTION_CONFIG_RELOADED,
            RESOURCE_TYPE_CONFIG,
            None,
        );

        assert!(result.is_ok());
        let log = result.unwrap();
        assert!(log.actor_id.is_none());
        assert_eq!(log.actor_type, ACTOR_TYPE_SYSTEM);
    }
}
