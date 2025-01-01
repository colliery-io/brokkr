//! # Stack Label Module
//!
//! This module defines structures and methods for managing stack labels in the system.
//!
//! ## Data Model
//!
//! Stack labels are used to categorize and organize stacks. They are stored in the `stack_labels` table
//! with the following structure:
//!
//! - `id`: UUID, primary key
//! - `stack_id`: UUID, foreign key referencing the `stacks` table
//! - `label`: VARCHAR(255), the label associated with the stack
//!
//! ## Usage
//!
//! Stack labels can be used to group stacks, filter them, or provide additional metadata.
//! This can be useful for organizing stacks based on their purpose, environment, or any other
//! relevant categorization.
//!
//! ## Constraints
//!
//! - The `stack_id` must be a valid, non-nil UUID.
//! - The `label` must be a non-empty string.
//! - The `label` must not exceed 64 characters.
//! - The `label` cannot contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a stack label in the database.
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
#[diesel(table_name = crate::schema::stack_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StackLabel {
    /// Unique identifier for the stack label.
    pub id: Uuid,
    /// ID of the stack this label is associated with.
    pub stack_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

/// Represents a new stack label to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stack_labels)]
pub struct NewStackLabel {
    /// ID of the stack this label is associated with.
    pub stack_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

impl NewStackLabel {
    /// Creates a new `NewStackLabel` instance.
    ///
    /// # Parameters
    ///
    /// * `stack_id`: UUID of the stack to associate the label with.
    /// * `label`: The label text. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewStackLabel)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(stack_id: Uuid, label: String) -> Result<Self, String> {
        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        // Validate label
        if label.trim().is_empty() {
            return Err("Label cannot be empty".to_string());
        }

        // Check label length
        if label.len() > 64 {
            return Err("Label cannot exceed 64 characters".to_string());
        }

        // Check whitespace
        if label.contains(char::is_whitespace) {
            return Err("Label cannot contain whitespace".to_string());
        }

        Ok(NewStackLabel { stack_id, label })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_label_success() {
        let stack_id = Uuid::new_v4();
        let label = "test-label".to_string();

        let result = NewStackLabel::new(stack_id, label.clone());

        assert!(
            result.is_ok(),
            "NewStackLabel creation should succeed with valid inputs"
        );
        let new_label = result.unwrap();
        assert_eq!(
            new_label.stack_id, stack_id,
            "stack_id should match the input value"
        );
        assert_eq!(new_label.label, label, "label should match the input value");
    }

    #[test]
    fn test_new_stack_label_invalid_stack_id() {
        let result = NewStackLabel::new(Uuid::nil(), "test-label".to_string());
        assert!(
            result.is_err(),
            "NewStackLabel creation should fail with nil stack ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid stack ID",
            "Error message should indicate invalid stack ID"
        );
    }

    #[test]
    fn test_new_stack_label_empty_label() {
        let result = NewStackLabel::new(Uuid::new_v4(), "".to_string());
        assert!(
            result.is_err(),
            "NewStackLabel creation should fail with empty label"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot be empty",
            "Error message should indicate empty label"
        );
    }

    #[test]
    fn test_new_stack_label_whitespace_label() {
        let result = NewStackLabel::new(Uuid::new_v4(), "   ".to_string());
        assert!(
            result.is_err(),
            "NewStackLabel creation should fail with whitespace-only label"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot be empty",
            "Error message should indicate empty label"
        );
    }

    #[test]
    fn test_new_stack_label_too_long() {
        let long_label = "a".repeat(65);
        let result = NewStackLabel::new(Uuid::new_v4(), long_label);
        assert!(
            result.is_err(),
            "NewStackLabel creation should fail with label exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot exceed 64 characters",
            "Error message should indicate label is too long"
        );
    }

    #[test]
    fn test_new_stack_label_max_length() {
        let max_length_label = "a".repeat(64);
        let result = NewStackLabel::new(Uuid::new_v4(), max_length_label);
        assert!(
            result.is_ok(),
            "NewStackLabel creation should succeed with 64-character label"
        );
    }
}
