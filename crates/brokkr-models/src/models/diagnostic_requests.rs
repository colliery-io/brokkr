/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostic Request model for on-demand diagnostic requests.
//!
//! Diagnostic requests are created by operators to request detailed diagnostic
//! information from agents about specific deployment objects.

use crate::schema::diagnostic_requests;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Valid diagnostic request statuses
pub const VALID_STATUSES: &[&str] = &["pending", "claimed", "completed", "failed", "expired"];

/// A diagnostic request record from the database.
#[derive(
    Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema,
)]
#[diesel(table_name = diagnostic_requests)]
pub struct DiagnosticRequest {
    /// Unique identifier for the diagnostic request.
    pub id: Uuid,
    /// The agent that should handle this request.
    pub agent_id: Uuid,
    /// The deployment object to gather diagnostics for.
    pub deployment_object_id: Uuid,
    /// Status: pending, claimed, completed, failed, expired.
    pub status: String,
    /// Who requested the diagnostics (e.g., operator username).
    pub requested_by: Option<String>,
    /// When the request was created.
    pub created_at: DateTime<Utc>,
    /// When the agent claimed the request.
    pub claimed_at: Option<DateTime<Utc>>,
    /// When the request was completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// When the request expires and should be cleaned up.
    pub expires_at: DateTime<Utc>,
}

/// A new diagnostic request to be inserted.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = diagnostic_requests)]
pub struct NewDiagnosticRequest {
    /// The agent that should handle this request.
    pub agent_id: Uuid,
    /// The deployment object to gather diagnostics for.
    pub deployment_object_id: Uuid,
    /// Status (defaults to "pending").
    pub status: String,
    /// Who requested the diagnostics.
    pub requested_by: Option<String>,
    /// When the request expires.
    pub expires_at: DateTime<Utc>,
}

impl NewDiagnosticRequest {
    /// Creates a new diagnostic request.
    ///
    /// # Arguments
    /// * `agent_id` - The agent that should handle this request.
    /// * `deployment_object_id` - The deployment object to gather diagnostics for.
    /// * `requested_by` - Optional identifier of who requested the diagnostics.
    /// * `retention_minutes` - How long the request should be retained (default 60).
    ///
    /// # Returns
    /// A Result containing the new diagnostic request or an error.
    pub fn new(
        agent_id: Uuid,
        deployment_object_id: Uuid,
        requested_by: Option<String>,
        retention_minutes: Option<i64>,
    ) -> Result<Self, String> {
        // Validate UUIDs are not nil
        if agent_id.is_nil() {
            return Err("Agent ID cannot be nil".to_string());
        }
        if deployment_object_id.is_nil() {
            return Err("Deployment object ID cannot be nil".to_string());
        }

        let retention = retention_minutes.unwrap_or(60);
        if retention < 1 || retention > 1440 {
            return Err("Retention must be between 1 and 1440 minutes".to_string());
        }

        let expires_at = Utc::now() + Duration::minutes(retention);

        Ok(Self {
            agent_id,
            deployment_object_id,
            status: "pending".to_string(),
            requested_by,
            expires_at,
        })
    }
}

/// Changeset for updating a diagnostic request.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = diagnostic_requests)]
pub struct UpdateDiagnosticRequest {
    /// New status.
    pub status: Option<String>,
    /// When claimed.
    pub claimed_at: Option<DateTime<Utc>>,
    /// When completed.
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_diagnostic_request_success() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();

        let result = NewDiagnosticRequest::new(
            agent_id,
            deployment_object_id,
            Some("admin@example.com".to_string()),
            Some(60),
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.agent_id, agent_id);
        assert_eq!(request.deployment_object_id, deployment_object_id);
        assert_eq!(request.status, "pending");
        assert_eq!(request.requested_by, Some("admin@example.com".to_string()));
        assert!(request.expires_at > Utc::now());
    }

    #[test]
    fn test_new_diagnostic_request_nil_agent_id() {
        let result = NewDiagnosticRequest::new(
            Uuid::nil(),
            Uuid::new_v4(),
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Agent ID cannot be nil");
    }

    #[test]
    fn test_new_diagnostic_request_nil_deployment_object_id() {
        let result = NewDiagnosticRequest::new(
            Uuid::new_v4(),
            Uuid::nil(),
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Deployment object ID cannot be nil");
    }

    #[test]
    fn test_new_diagnostic_request_invalid_retention() {
        let result = NewDiagnosticRequest::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Some(0),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Retention must be between"));
    }

    #[test]
    fn test_new_diagnostic_request_default_retention() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();

        let result = NewDiagnosticRequest::new(
            agent_id,
            deployment_object_id,
            None,
            None,
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        // Default retention is 60 minutes
        let expected_min = Utc::now() + Duration::minutes(59);
        let expected_max = Utc::now() + Duration::minutes(61);
        assert!(request.expires_at > expected_min && request.expires_at < expected_max);
    }
}
