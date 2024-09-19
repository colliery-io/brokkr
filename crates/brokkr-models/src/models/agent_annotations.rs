//! # Agent Annotation Module
//!
//! This module defines structures and methods for managing agent annotations in the system.
//!
//! ## Data Model
//!
//! Agent annotations are key-value pairs associated with agents. They are stored in the `agent_annotations` table
//! with the following structure:
//!
//! - `id`: UUID, primary key
//! - `agent_id`: UUID, foreign key referencing the `agents` table
//! - `key`: VARCHAR(255), the annotation key
//! - `value`: TEXT, the annotation value
//!
//! ## Usage
//!
//! Agent annotations can be used to add metadata to agents. This metadata can be used for filtering, grouping,
//! or providing additional information about the agent that doesn't fit into the main agent structure.
//!
//! ## Constraints
//!
//! - The `agent_id` must be a valid, non-nil UUID.
//! - Both `key` and `value` must be non-empty strings.
//! - `key` and `value` must not exceed 64 characters.
//! - Neither `key` nor `value` can contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent annotation in the database.
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
)]
#[diesel(table_name = crate::schema::agent_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentAnnotation {
    /// Unique identifier for the annotation.
    pub id: Uuid,
    /// ID of the agent this annotation belongs to.
    pub agent_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String,
}

/// Represents a new agent annotation to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_annotations)]
pub struct NewAgentAnnotation {
    /// ID of the agent this annotation belongs to.
    pub agent_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String,
}

impl NewAgentAnnotation {
    /// Creates a new `NewAgentAnnotation` instance.
    ///
    /// # Parameters
    ///
    /// * `agent_id`: UUID of the agent to associate the annotation with.
    /// * `key`: The key for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    /// * `value`: The value for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewAgentAnnotation)` if all parameters are valid, otherwise returns an `Err` with a description of the validation failure.
    pub fn new(agent_id: Uuid, key: String, value: String) -> Result<Self, String> {
        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        // Validate key
        if key.is_empty() {
            return Err("Key cannot be empty".to_string());
        }
        if key.len() > 64 {
            return Err("Key cannot exceed 64 characters".to_string());
        }
        if key.contains(char::is_whitespace) {
            return Err("Key cannot contain whitespace".to_string());
        }

        // Validate value
        if value.is_empty() {
            return Err("Value cannot be empty".to_string());
        }
        if value.len() > 64 {
            return Err("Value cannot exceed 64 characters".to_string());
        }
        if value.contains(char::is_whitespace) {
            return Err("Value cannot contain whitespace".to_string());
        }

        Ok(NewAgentAnnotation {
            agent_id,
            key,
            value,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_annotation_success() {
        let agent_id = Uuid::new_v4();
        let key = "test-key".to_string();
        let value = "test-value".to_string();

        let result = NewAgentAnnotation::new(agent_id, key.clone(), value.clone());

        assert!(
            result.is_ok(),
            "NewAgentAnnotation creation should succeed with valid inputs"
        );
        let new_annotation = result.unwrap();
        assert_eq!(
            new_annotation.agent_id, agent_id,
            "agent_id should match the input value"
        );
        assert_eq!(new_annotation.key, key, "key should match the input value");
        assert_eq!(
            new_annotation.value, value,
            "value should match the input value"
        );
    }

    #[test]
    fn test_new_agent_annotation_invalid_agent_id() {
        let result = NewAgentAnnotation::new(
            Uuid::nil(),
            "test-key".to_string(),
            "test-value".to_string(),
        );
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with nil agent ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid agent ID",
            "Error message should indicate invalid agent ID"
        );
    }

    #[test]
    fn test_new_agent_annotation_empty_key() {
        let result =
            NewAgentAnnotation::new(Uuid::new_v4(), "".to_string(), "test-value".to_string());
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with empty key"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot be empty",
            "Error message should indicate empty key"
        );
    }

    #[test]
    fn test_new_agent_annotation_empty_value() {
        let result =
            NewAgentAnnotation::new(Uuid::new_v4(), "test-key".to_string(), "".to_string());
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with empty value"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot be empty",
            "Error message should indicate empty value"
        );
    }

    #[test]
    fn test_new_agent_annotation_key_too_long() {
        let long_key = "a".repeat(65);
        let result = NewAgentAnnotation::new(Uuid::new_v4(), long_key, "test-value".to_string());
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with key exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot exceed 64 characters",
            "Error message should indicate key is too long"
        );
    }

    #[test]
    fn test_new_agent_annotation_value_too_long() {
        let long_value = "a".repeat(65);
        let result = NewAgentAnnotation::new(Uuid::new_v4(), "test-key".to_string(), long_value);
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with value exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot exceed 64 characters",
            "Error message should indicate value is too long"
        );
    }

    #[test]
    fn test_new_agent_annotation_key_with_whitespace() {
        let key_with_space = "test key".to_string();
        let result =
            NewAgentAnnotation::new(Uuid::new_v4(), key_with_space, "test-value".to_string());
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with key containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot contain whitespace",
            "Error message should indicate key contains whitespace"
        );
    }

    #[test]
    fn test_new_agent_annotation_value_with_whitespace() {
        let value_with_space = "test value".to_string();
        let result =
            NewAgentAnnotation::new(Uuid::new_v4(), "test-key".to_string(), value_with_space);
        assert!(
            result.is_err(),
            "NewAgentAnnotation creation should fail with value containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot contain whitespace",
            "Error message should indicate value contains whitespace"
        );
    }
}
