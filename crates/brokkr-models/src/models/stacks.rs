//! # Stacks Module
//!
//! This module defines the data structures and operations for stacks in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for stacks is represented by the `Stack` struct:
//!
//! - `id`: Uuid - Unique identifier for the stack
//! - `created_at`: DateTime<Utc> - Timestamp when the stack was created
//! - `updated_at`: DateTime<Utc> - Timestamp when the stack was last updated
//! - `deleted_at`: Option<DateTime<Utc>> - Timestamp when the stack was soft-deleted (if applicable)
//! - `name`: String - Name of the stack
//! - `description`: Option<String> - Optional description of the stack
//! - `labels`: Option<serde_json::Value> - Optional JSON value containing labels associated with the stack
//! - `annotations`: Option<serde_json::Value> - Optional JSON value containing annotations for the stack
//! - `agent_target`: Option<serde_json::Value> - Optional JSON value containing agent targeting information
//!
//! The `NewStack` struct is used for creating new stacks and contains the same fields
//! as `Stack`, except for `id`, `created_at`, `updated_at`, and `deleted_at`, which are
//! managed by the database.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a stack in the system.
///
/// This struct is used for querying existing stacks from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stack {
    /// Unique identifier for the stack
    pub id: Uuid,
    /// Timestamp when the stack was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the stack was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the stack was soft-deleted (if applicable)
    pub deleted_at: Option<DateTime<Utc>>,
    /// Name of the stack
    pub name: String,
    /// Optional description of the stack
    pub description: Option<String>,
    /// Optional JSON value containing labels associated with the stack
    pub labels: Option<serde_json::Value>,
    /// Optional JSON value containing annotations for the stack
    pub annotations: Option<serde_json::Value>,
    /// Optional JSON value containing agent targeting information
    pub agent_target: Option<serde_json::Value>,
}

/// Represents a new stack to be inserted into the database.
///
/// This struct is used when creating new stacks.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
pub struct NewStack {
    /// Name of the stack
    pub name: String,
    /// Optional description of the stack
    pub description: Option<String>,
    /// Optional JSON value containing labels associated with the stack
    pub labels: Option<serde_json::Value>,
    /// Optional JSON value containing annotations for the stack
    pub annotations: Option<serde_json::Value>,
    /// Optional JSON value containing agent targeting information
    pub agent_target: Option<serde_json::Value>,
}

impl NewStack {
     /// Creates a new `NewStack` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the stack
    /// * `description` - Optional description of the stack
    /// * `labels` - Optional vector of strings representing labels
    /// * `annotations` - Optional vector of key-value pairs representing annotations
    /// * `agent_target` - Optional vector of strings representing agent targets
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewStack` instance if successful, or an error message if validation fails.
    pub fn new(
        name: String,
        description: Option<String>,
        labels: Option<Vec<String>>,
        annotations: Option<Vec<(String, String)>>,
        agent_target: Option<Vec<String>>,
    ) -> Result<Self, String> {
        // Check for empty name
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
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
        
        // Check agent_target
        if let Some(ref agent_target) = agent_target {
            if agent_target.iter().any(|target| target.trim().is_empty()) {
                return Err("Agent targets cannot contain empty strings".to_string());
            }
        }

        Ok(NewStack {
            name,
            description,
            labels: labels.map(|l| serde_json::to_value(l).unwrap()),
            annotations: annotations.map(|a| serde_json::to_value(a).unwrap()),
            agent_target: agent_target.map(|l| serde_json::to_value(l).unwrap())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    /// Tests the successful creation of a NewStack with all fields populated.
    ///
    /// This test:
    /// 1. Creates a new NewStack with specific name, description, labels, annotations, and agent_target.
    /// 2. Verifies that the creation is successful and returns an Ok result.
    /// 3. Checks that each field of the created object matches the input values.
    /// 4. Ensures that labels, annotations, and agent_target are correctly converted to JSON.
    fn test_new_stack_success() {
        let name = "Test Stack".to_string();
        let description = Some("A test stack".to_string());
        let labels = Some(vec!["test".to_string(), "example".to_string()]);
        let annotations = Some(vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]);
        let agent_target = Some(vec!["agent1".to_string(), "agent2".to_string()]);

        let new_stack = NewStack::new(
            name.clone(),
            description.clone(),
            labels.clone(),
            annotations.clone(),
            agent_target.clone(),
        ).unwrap();

        assert_eq!(new_stack.name, name, "Name should match the input value");
        assert_eq!(new_stack.description, description, "Description should match the input value");
        assert_eq!(new_stack.labels, labels.map(|l| json!(l)), "Labels should be correctly converted to JSON");
        assert_eq!(new_stack.annotations, annotations.map(|a| json!(a)), "Annotations should be correctly converted to JSON");
        assert_eq!(new_stack.agent_target, agent_target.map(|a| json!(a)), "Agent target should be correctly converted to JSON");
    }

    #[test]
    /// Tests that NewStack creation fails when given an empty name.
    ///
    /// This test ensures that the NewStack::new() method properly
    /// validates the name field and returns an appropriate error for an empty name.
    fn test_new_stack_empty_name() {
        let result = NewStack::new(
            "".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_err(), "NewStack creation should fail with empty name");
        assert_eq!(result.unwrap_err(), "Name cannot be empty", "Error message should indicate empty name");
    }

    #[test]
    /// Tests that NewStack creation fails when given an empty label.
    ///
    /// This test ensures that the NewStack::new() method properly
    /// validates the labels and returns an appropriate error if any label is empty.
    fn test_new_stack_empty_label() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            Some(vec!["valid".to_string(), "".to_string()]),
            None,
            None,
        );
        assert!(result.is_err(), "NewStack creation should fail with empty label");
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings", "Error message should indicate empty label");
    }

    #[test]
    /// Tests that NewStack creation fails when given an empty annotation key.
    ///
    /// This test ensures that the NewStack::new() method properly
    /// validates the annotation keys and returns an appropriate error if any key is empty.
    fn test_new_stack_empty_annotation_key() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            Some(vec![("".to_string(), "value".to_string())]),
            None,
        );
        assert!(result.is_err(), "NewStack creation should fail with empty annotation key");
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values", "Error message should indicate empty annotation key");
    }

    #[test]
    /// Tests that NewStack creation fails when given an empty annotation value.
    ///
    /// This test ensures that the NewStack::new() method properly
    /// validates the annotation values and returns an appropriate error if any value is empty.
    fn test_new_stack_empty_annotation_value() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            Some(vec![("key".to_string(), "".to_string())]),
            None,
        );
        assert!(result.is_err(), "NewStack creation should fail with empty annotation value");
        assert_eq!(result.unwrap_err(), "Annotations cannot contain empty keys or values", "Error message should indicate empty annotation value");
    }

    #[test]
    /// Tests that NewStack creation fails when given an empty agent target.
    ///
    /// This test ensures that the NewStack::new() method properly
    /// validates the agent_target and returns an appropriate error if any target is empty.
    fn test_new_stack_empty_agent_target() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            Some(vec!["valid".to_string(), "".to_string()]),
        );
        assert!(result.is_err(), "NewStack creation should fail with empty agent target");
        assert_eq!(result.unwrap_err(), "Labels cannot contain empty strings", "Error message should indicate empty agent target");
    }

    #[test]
    /// Tests that NewStack creation succeeds with an empty description.
    ///
    /// This test verifies that an empty string is a valid input for the description field.
    fn test_new_stack_valid_empty_description() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            Some("".to_string()),
            None,
            None,
            None,
        );
        assert!(result.is_ok(), "NewStack creation should succeed with empty description");
    }

    #[test]
    /// Tests that NewStack creation succeeds with valid agent targets.
    ///
    /// This test verifies that the NewStack::new() method correctly handles
    /// a list of non-empty agent targets.
    fn test_new_stack_valid_agent_target() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            Some(vec!["agent1".to_string(), "agent2".to_string()]),
        );
        assert!(result.is_ok(), "NewStack creation should succeed with valid agent targets");
    }

    #[test]
    /// Tests that NewStack creation succeeds when all optional fields are None.
    ///
    /// This test ensures that the NewStack::new() method correctly handles
    /// the case where only the name is provided and all other fields are None.
    fn test_new_stack_all_none_optional_fields() {
        let result = NewStack::new(
            "Test Stack".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok(), "NewStack creation should succeed with all optional fields as None");
        let new_stack = result.unwrap();
        assert_eq!(new_stack.name, "Test Stack", "Name should match the input value");
        assert_eq!(new_stack.description, None, "Description should be None");
        assert_eq!(new_stack.labels, None, "Labels should be None");
        assert_eq!(new_stack.annotations, None, "Annotations should be None");
        assert_eq!(new_stack.agent_target, None, "Agent target should be None");
    }
}