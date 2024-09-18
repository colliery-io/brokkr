//! # Stack Annotation Module
//! 
//! This module defines structures and methods for managing stack annotations in the system.
//! 
//! ## Data Model
//! 
//! Stack annotations are key-value pairs associated with stacks, providing additional metadata.
//! They are stored in the `stack_annotations` table with the following structure:
//! 
//! - `id`: UUID, primary key
//! - `stack_id`: UUID, foreign key referencing the `stacks` table
//! - `key`: VARCHAR(255), the annotation key
//! - `value`: TEXT, the annotation value
//! 
//! ## Usage
//! 
//! Stack annotations can be used to add metadata to stacks. This metadata can be used for 
//! filtering, grouping, or providing additional information about the stack that doesn't fit 
//! into the main stack structure.
//! 
//! ## Constraints
//! 
//! - The `stack_id` must be a valid, non-nil UUID.
//! - Both `key` and `value` must be non-empty strings.
//! - Both `key` and `value` must not exceed 64 characters.
//! - Neither `key` nor `value` can contain whitespace.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a stack annotation in the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::stack_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StackAnnotation {
    /// Unique identifier for the annotation.
    pub id: Uuid,
    /// ID of the stack this annotation belongs to.
    pub stack_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String
}

/// Represents a new stack annotation to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stack_annotations)]
pub struct NewStackAnnotation {
    /// ID of the stack this annotation belongs to.
    pub stack_id: Uuid,
    /// Key of the annotation (max 64 characters, no whitespace).
    pub key: String,
    /// Value of the annotation (max 64 characters, no whitespace).
    pub value: String,
}

impl NewStackAnnotation {
    /// Creates a new `NewStackAnnotation` instance.
    ///
    /// # Parameters
    ///
    /// * `stack_id`: UUID of the stack to associate the annotation with.
    /// * `key`: The key for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    /// * `value`: The value for the annotation. Must be non-empty, max 64 characters, and contain no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewStackAnnotation)` if all parameters are valid, 
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        stack_id: Uuid,
        key: String,
        value: String,
    ) -> Result<Self, String> {
        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
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

        Ok(NewStackAnnotation {
            stack_id,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_annotation_success() {
        let stack_id = Uuid::new_v4();
        let key = "test-key".to_string();
        let value = "test-value".to_string();

        let result = NewStackAnnotation::new(stack_id, key.clone(), value.clone());

        assert!(result.is_ok(), "NewStackAnnotation creation should succeed with valid inputs");
        let new_annotation = result.unwrap();
        assert_eq!(new_annotation.stack_id, stack_id, "stack_id should match the input value");
        assert_eq!(new_annotation.key, key, "key should match the input value");
        assert_eq!(new_annotation.value, value, "value should match the input value");
    }

    #[test]
    fn test_new_stack_annotation_invalid_stack_id() {
        let result = NewStackAnnotation::new(Uuid::nil(), "test-key".to_string(), "test-value".to_string());
        assert!(result.is_err(), "NewStackAnnotation creation should fail with nil stack ID");
        assert_eq!(result.unwrap_err(), "Invalid stack ID", "Error message should indicate invalid stack ID");
    }

    #[test]
    fn test_new_stack_annotation_empty_key() {
        let result = NewStackAnnotation::new(Uuid::new_v4(), "".to_string(), "test-value".to_string());
        assert!(result.is_err(), "NewStackAnnotation creation should fail with empty key");
        assert_eq!(result.unwrap_err(), "Key cannot be empty", "Error message should indicate empty key");
    }

    #[test]
    fn test_new_stack_annotation_empty_value() {
        let result = NewStackAnnotation::new(Uuid::new_v4(), "test-key".to_string(), "".to_string());
        assert!(result.is_err(), "NewStackAnnotation creation should fail with empty value");
        assert_eq!(result.unwrap_err(), "Value cannot be empty", "Error message should indicate empty value");
    }

    #[test]
    fn test_new_stack_annotation_key_too_long() {
        let long_key = "a".repeat(65);
        let result = NewStackAnnotation::new(Uuid::new_v4(), long_key, "test-value".to_string());
        assert!(result.is_err(), "NewStackAnnotation creation should fail with key exceeding 64 characters");
        assert_eq!(result.unwrap_err(), "Key cannot exceed 64 characters", "Error message should indicate key is too long");
    }

    #[test]
    fn test_new_stack_annotation_value_too_long() {
        let long_value = "a".repeat(65);
        let result = NewStackAnnotation::new(Uuid::new_v4(), "test-key".to_string(), long_value);
        assert!(result.is_err(), "NewStackAnnotation creation should fail with value exceeding 64 characters");
        assert_eq!(result.unwrap_err(), "Value cannot exceed 64 characters", "Error message should indicate value is too long");
    }

    #[test]
    fn test_new_stack_annotation_key_with_whitespace() {
        let key_with_space = "test key".to_string();
        let result = NewStackAnnotation::new(Uuid::new_v4(), key_with_space, "test-value".to_string());
        assert!(result.is_err(), "NewStackAnnotation creation should fail with key containing whitespace");
        assert_eq!(result.unwrap_err(), "Key cannot contain whitespace", "Error message should indicate key contains whitespace");
    }

    #[test]
    fn test_new_stack_annotation_value_with_whitespace() {
        let value_with_space = "test value".to_string();
        let result = NewStackAnnotation::new(Uuid::new_v4(), "test-key".to_string(), value_with_space);
        assert!(result.is_err(), "NewStackAnnotation creation should fail with value containing whitespace");
        assert_eq!(result.unwrap_err(), "Value cannot contain whitespace", "Error message should indicate value contains whitespace");
    }
}