/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Work Order Label Module
//!
//! This module defines structures and methods for managing work order labels in the system.
//!
//! ## Data Model
//!
//! Work order labels are used to target agents by their labels. They are stored in the
//! `work_order_labels` table with the following structure:
//!
//! - `id`: UUID, primary key
//! - `work_order_id`: UUID, foreign key referencing the `work_orders` table
//! - `label`: VARCHAR(64), the label to match against agent labels
//! - `created_at`: Timestamp
//!
//! ## Usage
//!
//! Work order labels allow work orders to target agents that have any of the specified labels.
//! Matching is OR-based: if a work order has labels ["gpu", "production"], it will be available
//! to any agent that has at least one of those labels.
//!
//! ## Constraints
//!
//! - The `work_order_id` must be a valid, non-nil UUID.
//! - The `label` must be a non-empty string.
//! - The `label` must not exceed 64 characters.
//! - The `label` cannot contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a work order label in the database.
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
#[diesel(table_name = crate::schema::work_order_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkOrderLabel {
    /// Unique identifier for the work order label.
    pub id: Uuid,
    /// ID of the work order this label is associated with.
    pub work_order_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
    /// Timestamp when the label was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a new work order label to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::work_order_labels)]
pub struct NewWorkOrderLabel {
    /// ID of the work order this label is associated with.
    pub work_order_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

impl NewWorkOrderLabel {
    /// Creates a new `NewWorkOrderLabel` instance.
    ///
    /// # Parameters
    ///
    /// * `work_order_id`: UUID of the work order to associate the label with.
    /// * `label`: The label text. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewWorkOrderLabel)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(work_order_id: Uuid, label: String) -> Result<Self, String> {
        // Validate work_order_id
        if work_order_id.is_nil() {
            return Err("Invalid work order ID".to_string());
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

        Ok(NewWorkOrderLabel {
            work_order_id,
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_work_order_label_success() {
        let work_order_id = Uuid::new_v4();
        let label = "test-label".to_string();

        let result = NewWorkOrderLabel::new(work_order_id, label.clone());

        assert!(
            result.is_ok(),
            "NewWorkOrderLabel creation should succeed with valid inputs"
        );
        let new_label = result.unwrap();
        assert_eq!(
            new_label.work_order_id, work_order_id,
            "work_order_id should match the input value"
        );
        assert_eq!(new_label.label, label, "label should match the input value");
    }

    #[test]
    fn test_new_work_order_label_invalid_work_order_id() {
        let result = NewWorkOrderLabel::new(Uuid::nil(), "test-label".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderLabel creation should fail with nil work order ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid work order ID",
            "Error message should indicate invalid work order ID"
        );
    }

    #[test]
    fn test_new_work_order_label_empty_label() {
        let result = NewWorkOrderLabel::new(Uuid::new_v4(), "".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderLabel creation should fail with empty label"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot be empty",
            "Error message should indicate empty label"
        );
    }

    #[test]
    fn test_new_work_order_label_whitespace_label() {
        let result = NewWorkOrderLabel::new(Uuid::new_v4(), "   ".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderLabel creation should fail with whitespace-only label"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot be empty",
            "Error message should indicate empty label"
        );
    }

    #[test]
    fn test_new_work_order_label_too_long() {
        let long_label = "a".repeat(65);
        let result = NewWorkOrderLabel::new(Uuid::new_v4(), long_label);
        assert!(
            result.is_err(),
            "NewWorkOrderLabel creation should fail with label exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot exceed 64 characters",
            "Error message should indicate label is too long"
        );
    }

    #[test]
    fn test_new_work_order_label_max_length() {
        let max_length_label = "a".repeat(64);
        let result = NewWorkOrderLabel::new(Uuid::new_v4(), max_length_label);
        assert!(
            result.is_ok(),
            "NewWorkOrderLabel creation should succeed with 64-character label"
        );
    }

    #[test]
    fn test_new_work_order_label_with_whitespace() {
        let result = NewWorkOrderLabel::new(Uuid::new_v4(), "test label".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderLabel creation should fail with label containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Label cannot contain whitespace",
            "Error message should indicate label contains whitespace"
        );
    }
}
