/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Agent Label Module
//!
//! This module defines structures and methods for managing agent labels in the system.
//!
//! ## Data Model
//!
//! Agent labels are used to categorize and organize agents. They are stored in the `agent_labels` table
//! with the following structure:
//!
//! - `id`: UUID, primary key
//! - `agent_id`: UUID, foreign key referencing the `agents` table
//! - `label`: VARCHAR(255), the label associated with the agent
//!
//! ## Usage
//!
//! Agent labels can be used to group agents, filter them, or provide additional metadata.
//! This can be useful for organizing agents based on their roles, environments, or any other
//! relevant categorization.
//!
//! ## Constraints
//!
//! - The `agent_id` must be a valid, non-nil UUID.
//! - The `label` must be a non-empty string.
//! - The `label` must not exceed 64 characters.
//! - The `label` cannot contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents an agent label in the database.
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
#[diesel(table_name = crate::schema::agent_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentLabel {
    /// Unique identifier for the agent label.
    pub id: Uuid,
    /// ID of the agent this label is associated with.
    pub agent_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

/// Represents a new agent label to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::agent_labels)]
pub struct NewAgentLabel {
    /// ID of the agent this label is associated with.
    pub agent_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

impl NewAgentLabel {
    /// Creates a new `NewAgentLabel` instance.
    ///
    /// # Parameters
    ///
    /// * `agent_id`: UUID of the agent to associate the label with.
    /// * `label`: The label text. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewAgentLabel)` if all parameters are valid, otherwise returns an `Err` with a description of the validation failure.
    pub fn new(agent_id: Uuid, label: String) -> Result<Self, String> {
        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        // Validate label
        if label.is_empty() {
            return Err("Label cannot be empty".to_string());
        }
        if label.len() > 64 {
            return Err("Label cannot exceed 64 characters".to_string());
        }
        if label.contains(char::is_whitespace) {
            return Err("Label cannot contain whitespace".to_string());
        }

        Ok(NewAgentLabel { agent_id, label })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_label_success() {
        let agent_id = Uuid::new_v4();
        let label = "test-label".to_string();

        let result = NewAgentLabel::new(agent_id, label.clone());

        assert!(
            result.is_ok(),
            "NewAgentLabel creation should succeed with valid inputs"
        );
        let new_label = result.unwrap();
        assert_eq!(
            new_label.agent_id, agent_id,
            "agent_id should match the input value"
        );
        assert_eq!(new_label.label, label, "label should match the input value");
    }

    #[test]
    fn test_new_agent_label_invalid_agent_id() {
        let result = NewAgentLabel::new(Uuid::nil(), "test-label".to_string());
        assert!(
            result.is_err(),
            "NewAgentLabel creation should fail with nil agent ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid agent ID",
            "Error message should indicate invalid agent ID"
        );
    }

    #[test]
    fn test_new_agent_label_empty_label() {
        let result = NewAgentLabel::new(Uuid::new_v4(), "".to_string());
        assert!(
            result.is_err(),
            "NewAgentLabel creation should fail with empty label"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot be empty",
            "Error message should indicate empty label"
        );
    }

    #[test]
    fn test_new_agent_label_too_long() {
        let long_label = "a".repeat(65);
        let result = NewAgentLabel::new(Uuid::new_v4(), long_label);
        assert!(
            result.is_err(),
            "NewAgentLabel creation should fail with label exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot exceed 64 characters",
            "Error message should indicate label is too long"
        );
    }

    #[test]
    fn test_new_agent_label_with_whitespace() {
        let label_with_space = "test label".to_string();
        let result = NewAgentLabel::new(Uuid::new_v4(), label_with_space);
        assert!(
            result.is_err(),
            "NewAgentLabel creation should fail with label containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot contain whitespace",
            "Error message should indicate label contains whitespace"
        );
    }

    #[test]
    fn test_new_agent_label_max_length() {
        let max_length_label = "a".repeat(64);
        let result = NewAgentLabel::new(Uuid::new_v4(), max_length_label);
        assert!(
            result.is_ok(),
            "NewAgentLabel creation should succeed with 64-character label"
        );
    }
}
