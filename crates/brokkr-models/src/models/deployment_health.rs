/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Deployment Health Module
//!
//! This module defines structures and methods for tracking deployment health status.
//!
//! ## Data Model
//!
//! Deployment health records track the health status of deployments on a per-agent basis.
//! Each agent that applies a deployment object has its own health record, since the same
//! deployment may have different health on different clusters.
//!
//! - `id`: UUID, primary key
//! - `agent_id`: UUID, foreign key referencing the `agents` table
//! - `deployment_object_id`: UUID, foreign key referencing the `deployment_objects` table
//! - `status`: VARCHAR(20), health status (healthy, degraded, failing, unknown)
//! - `summary`: TEXT, JSON-encoded health summary with pod counts and conditions
//! - `checked_at`: TIMESTAMP, when the agent last checked health
//! - `created_at`: TIMESTAMP, when the record was created
//! - `updated_at`: TIMESTAMP, when the record was last updated
//!
//! ## Usage
//!
//! Agents periodically check the health of deployed resources and report status to the broker.
//! This enables operators to see at-a-glance health across all deployments without direct
//! cluster access.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Valid health status values
pub const HEALTH_STATUS_HEALTHY: &str = "healthy";
pub const HEALTH_STATUS_DEGRADED: &str = "degraded";
pub const HEALTH_STATUS_FAILING: &str = "failing";
pub const HEALTH_STATUS_UNKNOWN: &str = "unknown";

const VALID_HEALTH_STATUSES: [&str; 4] = [
    HEALTH_STATUS_HEALTHY,
    HEALTH_STATUS_DEGRADED,
    HEALTH_STATUS_FAILING,
    HEALTH_STATUS_UNKNOWN,
];

/// Represents a deployment health record in the database.
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
#[diesel(table_name = crate::schema::deployment_health)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[schema(example = json!({
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "agent_id": "123e4567-e89b-12d3-a456-426614174001",
    "deployment_object_id": "123e4567-e89b-12d3-a456-426614174002",
    "status": "healthy",
    "summary": "{\"pods_ready\": 3, \"pods_total\": 3, \"conditions\": []}",
    "checked_at": "2023-01-01T00:00:00Z",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z"
}))]
pub struct DeploymentHealth {
    /// Unique identifier for the health record.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,
    /// ID of the agent that reported this health status.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174001")]
    pub agent_id: Uuid,
    /// ID of the deployment object this health status applies to.
    #[schema(example = "123e4567-e89b-12d3-a456-426614174002")]
    pub deployment_object_id: Uuid,
    /// Health status: healthy, degraded, failing, or unknown.
    #[schema(example = "healthy")]
    pub status: String,
    /// JSON-encoded summary with pod counts, conditions, and resource details.
    #[schema(example = "{\"pods_ready\": 3, \"pods_total\": 3, \"conditions\": []}")]
    pub summary: Option<String>,
    /// Timestamp when the agent last checked health.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub checked_at: DateTime<Utc>,
    /// Timestamp when the record was created.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Timestamp when the record was last updated.
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Represents a new deployment health record to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::deployment_health)]
pub struct NewDeploymentHealth {
    /// ID of the agent reporting this health status.
    pub agent_id: Uuid,
    /// ID of the deployment object this health status applies to.
    pub deployment_object_id: Uuid,
    /// Health status: healthy, degraded, failing, or unknown.
    pub status: String,
    /// JSON-encoded summary with pod counts, conditions, and resource details.
    pub summary: Option<String>,
    /// Timestamp when the agent checked health.
    pub checked_at: DateTime<Utc>,
}

impl NewDeploymentHealth {
    /// Creates a new `NewDeploymentHealth` instance.
    ///
    /// # Parameters
    ///
    /// * `agent_id`: UUID of the agent reporting health.
    /// * `deployment_object_id`: UUID of the deployment object.
    /// * `status`: Health status (healthy, degraded, failing, unknown).
    /// * `summary`: Optional JSON-encoded health summary.
    /// * `checked_at`: When the health was checked.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewDeploymentHealth)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        agent_id: Uuid,
        deployment_object_id: Uuid,
        status: String,
        summary: Option<String>,
        checked_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        // Validate deployment_object_id
        if deployment_object_id.is_nil() {
            return Err("Invalid deployment object ID".to_string());
        }

        // Validate status
        if !VALID_HEALTH_STATUSES.contains(&status.as_str()) {
            return Err(format!(
                "Invalid health status. Must be one of: {}",
                VALID_HEALTH_STATUSES.join(", ")
            ));
        }

        Ok(NewDeploymentHealth {
            agent_id,
            deployment_object_id,
            status,
            summary,
            checked_at,
        })
    }
}

/// Represents an update to an existing deployment health record.
#[derive(AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_health)]
pub struct UpdateDeploymentHealth {
    /// Updated health status.
    pub status: String,
    /// Updated JSON-encoded summary.
    pub summary: Option<String>,
    /// Updated check timestamp.
    pub checked_at: DateTime<Utc>,
}

/// Structured health summary for serialization/deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthSummary {
    /// Number of pods in ready state.
    pub pods_ready: i32,
    /// Total number of pods.
    pub pods_total: i32,
    /// List of detected problematic conditions (e.g., ImagePullBackOff).
    pub conditions: Vec<String>,
    /// Optional detailed resource status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceHealth>>,
}

/// Health status for an individual Kubernetes resource.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourceHealth {
    /// Resource kind (e.g., Deployment, StatefulSet).
    pub kind: String,
    /// Resource name.
    pub name: String,
    /// Resource namespace.
    pub namespace: String,
    /// Whether the resource is ready.
    pub ready: bool,
    /// Optional status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deployment_health_success() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();
        let status = HEALTH_STATUS_HEALTHY.to_string();
        let summary = Some(r#"{"pods_ready": 3, "pods_total": 3, "conditions": []}"#.to_string());
        let checked_at = Utc::now();

        let result = NewDeploymentHealth::new(
            agent_id,
            deployment_object_id,
            status.clone(),
            summary.clone(),
            checked_at,
        );

        assert!(
            result.is_ok(),
            "NewDeploymentHealth creation should succeed with valid inputs"
        );
        let new_health = result.unwrap();
        assert_eq!(new_health.agent_id, agent_id);
        assert_eq!(new_health.deployment_object_id, deployment_object_id);
        assert_eq!(new_health.status, status);
        assert_eq!(new_health.summary, summary);
    }

    #[test]
    fn test_new_deployment_health_invalid_agent_id() {
        let result = NewDeploymentHealth::new(
            Uuid::nil(),
            Uuid::new_v4(),
            HEALTH_STATUS_HEALTHY.to_string(),
            None,
            Utc::now(),
        );
        assert!(
            result.is_err(),
            "NewDeploymentHealth creation should fail with nil agent ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid agent ID",
            "Error message should indicate invalid agent ID"
        );
    }

    #[test]
    fn test_new_deployment_health_invalid_status() {
        let result = NewDeploymentHealth::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "invalid_status".to_string(),
            None,
            Utc::now(),
        );
        assert!(
            result.is_err(),
            "NewDeploymentHealth creation should fail with invalid status"
        );
        assert!(
            result.unwrap_err().contains("Invalid health status"),
            "Error message should indicate invalid status"
        );
    }

    #[test]
    fn test_health_summary_serialization() {
        let summary = HealthSummary {
            pods_ready: 2,
            pods_total: 3,
            conditions: vec!["ImagePullBackOff".to_string()],
            resources: Some(vec![ResourceHealth {
                kind: "Deployment".to_string(),
                name: "my-app".to_string(),
                namespace: "production".to_string(),
                ready: false,
                message: Some("2/3 replicas available".to_string()),
            }]),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let parsed: HealthSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.pods_ready, 2);
        assert_eq!(parsed.pods_total, 3);
        assert_eq!(parsed.conditions, vec!["ImagePullBackOff"]);
    }
}
