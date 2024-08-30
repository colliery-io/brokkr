//! # Annotations Module
//!
//! This module defines the data structures and operations for annotations in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for annotations is represented by the `Annotation` struct:
//!
//! - `id`: Uuid - Unique identifier for the annotation
//! - `object_id`: Uuid - Identifier of the object this annotation is associated with
//! - `object_type`: String - Type of the object this annotation is associated with
//! - `key`: String - The key of the annotation
//! - `value`: String - The value of the annotation

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an annotation in the system.
///
/// This struct is used for querying existing annotations from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AnnotationDB {
    /// Unique identifier for the annotation
    pub id: Uuid,
    /// Identifier of the object this annotation is associated with
    pub object_id: Uuid,
    /// Type of the object this annotation is associated with
    pub object_type: String,
    /// The key of the annotation
    pub key: String,
    /// The value of the annotation
    pub value: String,
}

/// Represents a new annotation to be inserted into the database.
///
/// This struct is used when creating new annotations.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::annotations)]
pub struct NewAnnotationDB {
    /// Identifier of the object this annotation is associated with
    pub object_id: Uuid,
    /// Type of the object this annotation is associated with
    pub object_type: String,
    /// The key of the annotation
    pub key: String,
    /// The value of the annotation
    pub value: String,
}

impl NewAnnotationDB {
    /// Creates a new `NewAnnotation` instance.
    ///
    /// # Arguments
    ///
    /// * `object_id` - UUID of the object this annotation is associated with
    /// * `object_type` - Type of the object this annotation is associated with
    /// * `key` - The key of the annotation
    /// * `value` - The value of the annotation
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewAnnotation` instance if successful, or an error message if validation fails.
    pub fn new(object_id: Uuid, object_type: String, key: String, value: String) -> Result<Self, String> {
        // Validate object_id
        if object_id.is_nil() {
            return Err("Invalid object ID".to_string());
        }

        // Validate object_type
        if object_type.trim().is_empty() {
            return Err("Object type cannot be empty".to_string());
        }

        // Validate key
        if key.trim().is_empty() {
            return Err("Annotation key cannot be empty".to_string());
        }

        // Validate value
        if value.trim().is_empty() {
            return Err("Annotation value cannot be empty".to_string());
        }

        Ok(NewAnnotationDB {
            object_id,
            object_type,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_annotation_success() {
        let object_id = Uuid::new_v4();
        let object_type = "stack".to_string();
        let key = "environment".to_string();
        let value = "production".to_string();

        let result = NewAnnotationDB::new(object_id, object_type.clone(), key.clone(), value.clone());

        assert!(result.is_ok(), "NewAnnotation creation should succeed with valid inputs");
        let new_annotation = result.unwrap();
        assert_eq!(new_annotation.object_id, object_id, "object_id should match the input value");
        assert_eq!(new_annotation.object_type, object_type, "object_type should match the input value");
        assert_eq!(new_annotation.key, key, "key should match the input value");
        assert_eq!(new_annotation.value, value, "value should match the input value");
    }

    #[test]
    fn test_new_annotation_invalid_object_id() {
        let result = NewAnnotationDB::new(Uuid::nil(), "stack".to_string(), "key".to_string(), "value".to_string());
        assert!(result.is_err(), "NewAnnotation creation should fail with nil object ID");
        assert_eq!(result.unwrap_err(), "Invalid object ID", "Error message should indicate invalid object ID");
    }

    #[test]
    fn test_new_annotation_empty_object_type() {
        let result = NewAnnotationDB::new(Uuid::new_v4(), "".to_string(), "key".to_string(), "value".to_string());
        assert!(result.is_err(), "NewAnnotation creation should fail with empty object type");
        assert_eq!(result.unwrap_err(), "Object type cannot be empty", "Error message should indicate empty object type");
    }

    #[test]
    fn test_new_annotation_empty_key() {
        let result = NewAnnotationDB::new(Uuid::new_v4(), "stack".to_string(), "".to_string(), "value".to_string());
        assert!(result.is_err(), "NewAnnotation creation should fail with empty key");
        assert_eq!(result.unwrap_err(), "Annotation key cannot be empty", "Error message should indicate empty key");
    }

    #[test]
    fn test_new_annotation_empty_value() {
        let result = NewAnnotationDB::new(Uuid::new_v4(), "stack".to_string(), "key".to_string(), "".to_string());
        assert!(result.is_err(), "NewAnnotation creation should fail with empty value");
        assert_eq!(result.unwrap_err(), "Annotation value cannot be empty", "Error message should indicate empty value");
    }
}