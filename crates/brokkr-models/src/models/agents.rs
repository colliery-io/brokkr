//! # Agents Module
//!
//! This module defines the data structures and operations for agents in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for agents is represented by the `Agent` struct:
//!
//! - `id`: Uuid - Unique identifier for the agent
//! - `created_at`: DateTime<Utc> - Timestamp when the agent was created
//! - `updated_at`: DateTime<Utc> - Timestamp when the agent was last updated
//! - `deleted_at`: Option<DateTime<Utc>> - Timestamp when the agent was soft-deleted (if applicable)
//! - `uuid`: Uuid - Secondary UUID for the agent (used for external references)
//! - `name`: String - Name of the agent
//! - `cluster_name`: String - Name of the cluster the agent belongs to
//! - `labels`: Option<serde_json::Value> - Optional JSON value containing labels associated with the agent
//! - `annotations`: Option<serde_json::Value> - Optional JSON value containing annotations for the agent
//! - `last_heartbeat`: Option<DateTime<Utc>> - Timestamp of the last heartbeat received from the agent
//! - `status`: String - Current status of the agent
//!
//! The `NewAgent` struct is used for creating new agents and contains a subset of the fields
//! from `Agent`: `name`, `cluster_name`, `labels`, and `annotations`. The other fields are
//! managed by the database or set after creation.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent in the system.
///
/// This struct is used for querying existing agents from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: Uuid,
    /// Timestamp when the agent was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the agent was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the agent was soft-deleted (if applicable)
    pub deleted_at: Option<DateTime<Utc>>,
    /// Secondary UUID for the agent (used for external references)
    pub uuid: Uuid,
    /// Name of the agent
    pub name: String,
    /// Name of the cluster the agent belongs to
    pub cluster_name: String,
    /// Optional JSON value containing labels associated with the agent
    pub labels: Option<serde_json::Value>,
    /// Optional JSON value containing annotations for the agent
    pub annotations: Option<serde_json::Value>,
    /// Timestamp of the last heartbeat received from the agent
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Current status of the agent
    pub status: String,
}

/// Represents a new agent to be inserted into the database.
///
/// This struct is used when creating new agents.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
pub struct NewAgent {
    /// Name of the agent
    pub name: String,
    /// Name of the cluster the agent belongs to
    pub cluster_name: String,
    /// Optional JSON value containing labels associated with the agent
    pub labels: Option<serde_json::Value>,
    /// Optional JSON value containing annotations for the agent
    pub annotations: Option<serde_json::Value>,
}

impl NewAgent {
    /// Creates a new `NewAgent` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the agent
    /// * `cluster_name` - Name of the cluster the agent belongs to
    /// * `labels` - Optional vector of strings representing labels
    /// * `annotations` - Optional vector of key-value pairs representing annotations
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewAgent` instance if successful, or an error message if validation fails.
    pub fn new(
        name: String,
        cluster_name: String,
        labels: Option<Vec<String>>,
        annotations: Option<Vec<(String, String)>>,
    ) -> Result<Self, String> {
        // Check for empty strings
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if cluster_name.trim().is_empty() {
            return Err("Cluster name cannot be empty".to_string());
        }

        // Check labels
        if let Some(ref labels) = labels {
            if labels.iter().any(|label| label.trim().is_empty()) {
                return Err("Labels cannot contain empty strings".to_string());
            }
        }

        // Check annotations
        if let Some(ref annotations) = annotations {
            if annotations.iter().any(|(k, v)| k.trim().is_empty() || v.trim().is_empty()) {
                return Err("Annotations cannot contain empty keys or values".to_string());
            }
        }

        Ok(NewAgent {
            name,
            cluster_name,
            labels: labels_to_json(&labels),
            annotations: annotations_to_json(&annotations),
        })
    }
}

/// Converts a vector of strings to a JSON value for labels.
pub fn labels_to_json(labels: &Option<Vec<String>>) -> Option<serde_json::Value> {
    labels.as_ref().map(|l| serde_json::to_value(l).unwrap())
}

/// Converts a vector of key-value pairs to a JSON value for annotations.
pub fn annotations_to_json(annotations: &Option<Vec<(String, String)>>) -> Option<serde_json::Value> {
    annotations.as_ref().map(|a| serde_json::to_value(a).unwrap())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the successful creation of a NewAgent with valid input parameters.
    ///
    /// This test:
    /// 1. Creates a new NewAgent with specific name, cluster name, labels, and annotations.
    /// 2. Verifies that each field of the created agent matches the input values.
    /// 3. Checks that labels and annotations are correctly converted to JSON format.
    ///
    /// It ensures that the NewAgent::new() method correctly sets all fields
    /// and properly handles the conversion of labels and annotations to JSON.
    fn test_new_agent_success() {
        let name = "Test Agent".to_string();
        let cluster_name = "Test Cluster".to_string();
        let labels = Some(vec!["orange".to_string(), "blue".to_string()]);
        let annotations = Some(vec![
            ("security".to_string(), "high".to_string()),
            ("color".to_string(), "blue".to_string()),
        ]);

        let new_agent = NewAgent::new(
            name.clone(),
            cluster_name.clone(),
            labels.clone(),
            annotations.clone(),
        ).unwrap();

        assert_eq!(new_agent.name, name, "Agent name should match the input value");
        assert_eq!(new_agent.cluster_name, cluster_name, "Cluster name should match the input value");
        assert_eq!(new_agent.labels, labels_to_json(&labels), "Labels should be correctly converted to JSON");
        assert_eq!(new_agent.annotations, annotations_to_json(&annotations), "Annotations should be correctly converted to JSON");

        if let Some(lj) = new_agent.labels {
            assert_eq!(lj, serde_json::json!(["orange", "blue"]), "Labels JSON should match expected format");
        }

        if let Some(aj) = new_agent.annotations {
            assert_eq!(aj, serde_json::json!([["security", "high"], ["color", "blue"]]), "Annotations JSON should match expected format");
        }
    }

    #[test]
    /// Tests that NewAgent creation fails when given an empty name.
    ///
    /// This test ensures that the NewAgent::new() method properly
    /// validates the name field and returns an appropriate error.
    fn test_new_agent_empty_name() {
        let result = NewAgent::new(
            "".to_string(),
            "Test Cluster".to_string(),
            None,
            None,
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty name");
        assert_eq!(result.unwrap_err(), "Name cannot be empty", "Error message should indicate empty name");
    }

    #[test]
    /// Tests that NewAgent creation fails when given an empty cluster name.
    ///
    /// This test ensures that the NewAgent::new() method properly
    /// validates the cluster_name field and returns an appropriate error.
    fn test_new_agent_empty_cluster_name() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "".to_string(),
            None,
            None,
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty cluster name");
        assert_eq!(result.unwrap_err(), "Cluster name cannot be empty", "Error message should indicate empty cluster name");
    }

    #[test]
    /// Tests that NewAgent creation fails when given an empty label.
    ///
    /// This test ensures that the NewAgent::new() method properly
    /// validates the labels and returns an appropriate error if any label is empty.
    fn test_new_agent_empty_label() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            Some(vec!["valid".to_string(), "".to_string()]),
            None,
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty label");
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings", "Error message should indicate empty label");
    }

    #[test]
    /// Tests that NewAgent creation fails when given an empty annotation key.
    ///
    /// This test ensures that the NewAgent::new() method properly
    /// validates the annotation keys and returns an appropriate error if any key is empty.
    fn test_new_agent_empty_annotation_key() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            None,
            Some(vec![("".to_string(), "value".to_string())]),
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty annotation key");
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values", "Error message should indicate empty annotation key or value");
    }

    #[test]
    /// Tests that NewAgent creation fails when given an empty annotation value.
    ///
    /// This test ensures that the NewAgent::new() method properly
    /// validates the annotation values and returns an appropriate error if any value is empty.
    fn test_new_agent_empty_annotation_value() {
        let result = NewAgent::new(
            "Test Agent".to_string(),
            "Test Cluster".to_string(),
            None,
            Some(vec![("key".to_string(), "".to_string())]),
        );
        assert!(result.is_err(), "NewAgent creation should fail with empty annotation value");
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values", "Error message should indicate empty annotation key or value");
    }
}