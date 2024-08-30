//! # Labels Module
//!
//! This module defines the data structures and operations for labels in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for labels is represented by the `Label` struct:
//!
//! - `id`: Uuid - Unique identifier for the label
//! - `object_id`: Uuid - Identifier of the object this label is associated with
//! - `object_type`: String - Type of the object this label is associated with
//! - `label`: String - The label text

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a label in the system.
///
/// This struct is used for querying existing labels from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LabelDB {
    /// Unique identifier for the label
    pub id: Uuid,
    /// Identifier of the object this label is associated with
    pub object_id: Uuid,
    /// Type of the object this label is associated with
    pub object_type: String,
    /// The label text
    pub label: String,
}

/// Represents a new label to be inserted into the database.
///
/// This struct is used when creating new labels.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::labels)]
pub struct NewLabelDB {
    /// Identifier of the object this label is associated with
    pub object_id: Uuid,
    /// Type of the object this label is associated with
    pub object_type: String,
    /// The label text
    pub label: String,
}

impl NewLabelDB {
    /// Creates a new `NewLabel` instance.
    ///
    /// # Arguments
    ///
    /// * `object_id` - UUID of the object this label is associated with
    /// * `object_type` - Type of the object this label is associated with
    /// * `label` - The label text
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewLabel` instance if successful, or an error message if validation fails.
    pub fn new(object_id: Uuid, object_type: String, label: String) -> Result<Self, String> {
        // Validate object_id
        if object_id.is_nil() {
            return Err("Invalid object ID".to_string());
        }

        // Validate object_type
        if object_type.trim().is_empty() {
            return Err("Object type cannot be empty".to_string());
        }

        // Validate label
        if label.trim().is_empty() {
            return Err("Label cannot be empty".to_string());
        }

        Ok(NewLabelDB {
            object_id,
            object_type,
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_label_success() {
        let object_id = Uuid::new_v4();
        let object_type = "stack".to_string();
        let label = "production".to_string();

        let result = NewLabelDB::new(object_id, object_type.clone(), label.clone());

        assert!(result.is_ok(), "NewLabel creation should succeed with valid inputs");
        let new_label = result.unwrap();
        assert_eq!(new_label.object_id, object_id, "object_id should match the input value");
        assert_eq!(new_label.object_type, object_type, "object_type should match the input value");
        assert_eq!(new_label.label, label, "label should match the input value");
    }

    #[test]
    fn test_new_label_invalid_object_id() {
        let result = NewLabelDB::new(Uuid::nil(), "stack".to_string(), "production".to_string());
        assert!(result.is_err(), "NewLabel creation should fail with nil object ID");
        assert_eq!(result.unwrap_err(), "Invalid object ID", "Error message should indicate invalid object ID");
    }

    #[test]
    fn test_new_label_empty_object_type() {
        let result = NewLabelDB::new(Uuid::new_v4(), "".to_string(), "production".to_string());
        assert!(result.is_err(), "NewLabel creation should fail with empty object type");
        assert_eq!(result.unwrap_err(), "Object type cannot be empty", "Error message should indicate empty object type");
    }

    #[test]
    fn test_new_label_empty_label() {
        let result = NewLabelDB::new(Uuid::new_v4(), "stack".to_string(), "".to_string());
        assert!(result.is_err(), "NewLabel creation should fail with empty label");
        assert_eq!(result.unwrap_err(), "Label cannot be empty", "Error message should indicate empty label");
    }
}