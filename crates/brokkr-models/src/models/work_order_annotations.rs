/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Work Order Annotation Module
//!
//! This module defines structures and methods for managing work order annotations in the system.
//!
//! ## Data Model
//!
//! Work order annotations are key-value pairs used to target agents by their annotations.
//! They are stored in the `work_order_annotations` table with the following structure:
//!
//! - `id`: UUID, primary key
//! - `work_order_id`: UUID, foreign key referencing the `work_orders` table
//! - `key`: VARCHAR(64), the annotation key
//! - `value`: VARCHAR(64), the annotation value
//! - `created_at`: Timestamp
//!
//! ## Usage
//!
//! Work order annotations allow work orders to target agents that have any of the specified
//! key-value pairs. Matching is OR-based: if a work order has annotations {"region": "us-east",
//! "tier": "production"}, it will be available to any agent that has at least one of those
//! key-value pairs.
//!
//! ## Constraints
//!
//! - The `work_order_id` must be a valid, non-nil UUID.
//! - Both `key` and `value` must be non-empty strings.
//! - Both `key` and `value` must not exceed 64 characters.
//! - Neither `key` nor `value` can contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a work order annotation in the database.
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
#[diesel(table_name = crate::schema::work_order_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkOrderAnnotation {
    /// Unique identifier for the annotation.
    pub id: Uuid,
    /// ID of the work order this annotation belongs to.
    pub work_order_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String,
    /// Timestamp when the annotation was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a new work order annotation to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::work_order_annotations)]
pub struct NewWorkOrderAnnotation {
    /// ID of the work order this annotation belongs to.
    pub work_order_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String,
}

impl NewWorkOrderAnnotation {
    /// Creates a new `NewWorkOrderAnnotation` instance.
    ///
    /// # Parameters
    ///
    /// * `work_order_id`: UUID of the work order to associate the annotation with.
    /// * `key`: The key for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    /// * `value`: The value for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewWorkOrderAnnotation)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(work_order_id: Uuid, key: String, value: String) -> Result<Self, String> {
        // Validate work_order_id
        if work_order_id.is_nil() {
            return Err("Invalid work order ID".to_string());
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

        Ok(NewWorkOrderAnnotation {
            work_order_id,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_work_order_annotation_success() {
        let work_order_id = Uuid::new_v4();
        let key = "test-key".to_string();
        let value = "test-value".to_string();

        let result = NewWorkOrderAnnotation::new(work_order_id, key.clone(), value.clone());

        assert!(
            result.is_ok(),
            "NewWorkOrderAnnotation creation should succeed with valid inputs"
        );
        let new_annotation = result.unwrap();
        assert_eq!(
            new_annotation.work_order_id, work_order_id,
            "work_order_id should match the input value"
        );
        assert_eq!(new_annotation.key, key, "key should match the input value");
        assert_eq!(
            new_annotation.value, value,
            "value should match the input value"
        );
    }

    #[test]
    fn test_new_work_order_annotation_invalid_work_order_id() {
        let result = NewWorkOrderAnnotation::new(
            Uuid::nil(),
            "test-key".to_string(),
            "test-value".to_string(),
        );
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with nil work order ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid work order ID",
            "Error message should indicate invalid work order ID"
        );
    }

    #[test]
    fn test_new_work_order_annotation_empty_key() {
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), "".to_string(), "test-value".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with empty key"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot be empty",
            "Error message should indicate empty key"
        );
    }

    #[test]
    fn test_new_work_order_annotation_empty_value() {
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), "test-key".to_string(), "".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with empty value"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot be empty",
            "Error message should indicate empty value"
        );
    }

    #[test]
    fn test_new_work_order_annotation_key_too_long() {
        let long_key = "a".repeat(65);
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), long_key, "test-value".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with key exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot exceed 64 characters",
            "Error message should indicate key is too long"
        );
    }

    #[test]
    fn test_new_work_order_annotation_value_too_long() {
        let long_value = "a".repeat(65);
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), "test-key".to_string(), long_value);
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with value exceeding 64 characters"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot exceed 64 characters",
            "Error message should indicate value is too long"
        );
    }

    #[test]
    fn test_new_work_order_annotation_key_with_whitespace() {
        let key_with_space = "test key".to_string();
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), key_with_space, "test-value".to_string());
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with key containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Key cannot contain whitespace",
            "Error message should indicate key contains whitespace"
        );
    }

    #[test]
    fn test_new_work_order_annotation_value_with_whitespace() {
        let value_with_space = "test value".to_string();
        let result =
            NewWorkOrderAnnotation::new(Uuid::new_v4(), "test-key".to_string(), value_with_space);
        assert!(
            result.is_err(),
            "NewWorkOrderAnnotation creation should fail with value containing whitespace"
        );
        assert_eq!(
            result.unwrap_err(),
            "Value cannot contain whitespace",
            "Error message should indicate value contains whitespace"
        );
    }

    #[test]
    fn test_new_work_order_annotation_max_length() {
        let max_key = "a".repeat(64);
        let max_value = "b".repeat(64);
        let result = NewWorkOrderAnnotation::new(Uuid::new_v4(), max_key, max_value);
        assert!(
            result.is_ok(),
            "NewWorkOrderAnnotation creation should succeed with 64-character key and value"
        );
    }
}
