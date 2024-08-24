//! # Deployment Objects Module
//!
//! This module defines the data structures and operations for deployment objects in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for deployment objects is represented by the `DeploymentObject` struct:
//!
//! - `uuid`: Uuid - Unique identifier for the deployment object
//! - `sequence_id`: i64 - Sequential identifier for ordering deployment objects
//! - `stack_id`: Uuid - Identifier of the stack this deployment object belongs to
//! - `yaml_content`: String - The YAML content of the deployment object
//! - `yaml_checksum`: String - Checksum of the YAML content for integrity verification
//! - `deleted_at`: Option<DateTime<Utc>> - Timestamp when the object was soft-deleted (if applicable)
//! - `submitted_at`: DateTime<Utc> - Timestamp when the deployment object was submitted
//! - `is_deletion_marker`: bool - Flag indicating if this object marks a deletion
//!
//! The `NewDeploymentObject` struct is used for creating new deployment objects and contains
//! the same fields as `DeploymentObject`, except for `sequence_id` which is managed by the database.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a deployment object in the system.
///
/// This struct is used for querying existing deployment objects from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
#[diesel(primary_key(uuid))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeploymentObject {
    /// Unique identifier for the deployment object
    pub uuid: Uuid,
    /// Sequential identifier for ordering deployment objects
    pub sequence_id: i64,
    /// Identifier of the stack this deployment object belongs to
    pub stack_id: Uuid,
    /// The YAML content of the deployment object
    pub yaml_content: String,
    /// Checksum of the YAML content for integrity verification
    pub yaml_checksum: String,
    /// Timestamp when the object was soft-deleted (if applicable)
    pub deleted_at: Option<DateTime<Utc>>,
    /// Timestamp when the deployment object was submitted
    pub submitted_at: DateTime<Utc>,
    /// Flag indicating if this object marks a deletion
    pub is_deletion_marker: bool,
}

/// Represents a new deployment object to be inserted into the database.
///
/// This struct is used when creating new deployment objects.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
pub struct NewDeploymentObject {
    /// Unique identifier for the new deployment object
    pub uuid: Uuid,
    /// Identifier of the stack this deployment object belongs to
    pub stack_id: Uuid,
    /// The YAML content of the deployment object
    pub yaml_content: String,
    /// Checksum of the YAML content for integrity verification
    pub yaml_checksum: String,
    /// Timestamp when the deployment object was submitted
    pub submitted_at: DateTime<Utc>,
    /// Flag indicating if this object marks a deletion
    pub is_deletion_marker: bool,
}

impl NewDeploymentObject {
    /// Creates a new `NewDeploymentObject` instance.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - UUID of the stack this deployment object belongs to
    /// * `yaml_content` - YAML content of the deployment object
    /// * `yaml_checksum` - Checksum of the YAML content
    /// * `is_deletion_marker` - Flag indicating if this object marks a deletion
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewDeploymentObject` instance if successful, or an error message if validation fails.
    pub fn new(
        stack_id: Uuid,
        yaml_content: String,
        yaml_checksum: String,
        is_deletion_marker: bool,
    ) -> Result<Self, String> {
        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        // Validate yaml_content
        if yaml_content.trim().is_empty() {
            return Err("YAML content cannot be empty".to_string());
        }

        // Basic YAML structure validation
        if !yaml_content.contains(':') {
            return Err("Invalid YAML structure".to_string());
        }

        // Validate yaml_checksum
        if yaml_checksum.trim().is_empty() {
            return Err("YAML checksum cannot be empty".to_string());
        }

        Ok(NewDeploymentObject {
            uuid: Uuid::new_v4(),
            stack_id,
            yaml_content,
            yaml_checksum,
            submitted_at: Utc::now(),
            is_deletion_marker,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the successful creation of a NewDeploymentObject with valid input parameters.
    ///
    /// This test:
    /// 1. Creates a new NewDeploymentObject with specific stack_id, yaml_content, yaml_checksum, and is_deletion_marker.
    /// 2. Verifies that the creation is successful and returns an Ok result.
    /// 3. Checks that each field of the created object matches the input values.
    /// 4. Ensures that a non-nil UUID is generated for the new object.
    ///
    /// It validates that the NewDeploymentObject::new() method correctly sets all fields
    /// and generates a valid UUID for the new deployment object.
    fn test_new_deployment_object_success() {
        let stack_id = Uuid::new_v4();
        let yaml_content = "key: value\nother_key: other_value".to_string();
        let yaml_checksum = "abcdef123456".to_string();
        let is_deletion_marker = false;

        let result = NewDeploymentObject::new(
            stack_id,
            yaml_content.clone(),
            yaml_checksum.clone(),
            is_deletion_marker,
        );

        assert!(result.is_ok(), "NewDeploymentObject creation should succeed with valid inputs");
        let new_obj = result.unwrap();
        assert_eq!(new_obj.stack_id, stack_id, "stack_id should match the input value");
        assert_eq!(new_obj.yaml_content, yaml_content, "yaml_content should match the input value");
        assert_eq!(new_obj.yaml_checksum, yaml_checksum, "yaml_checksum should match the input value");
        assert_eq!(new_obj.is_deletion_marker, is_deletion_marker, "is_deletion_marker should match the input value");
        assert!(!new_obj.uuid.is_nil(), "A non-nil UUID should be generated");
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given an invalid (nil) stack ID.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the stack_id field and returns an appropriate error for a nil UUID.
    fn test_new_deployment_object_invalid_stack_id() {
        let result = NewDeploymentObject::new(
            Uuid::nil(),
            "key: value".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with nil stack ID");
        assert_eq!(result.unwrap_err(), "Invalid stack ID", "Error message should indicate invalid stack ID");
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given an empty YAML content string.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the yaml_content field and returns an appropriate error for empty content.
    fn test_new_deployment_object_empty_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with empty YAML content");
        assert_eq!(result.unwrap_err(), "YAML content cannot be empty", "Error message should indicate empty YAML content");
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given invalid YAML content.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the structure of the yaml_content and returns an appropriate error for invalid YAML.
    fn test_new_deployment_object_invalid_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "invalid yaml content".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with invalid YAML content");
        assert_eq!(result.unwrap_err(), "Invalid YAML structure", "Error message should indicate invalid YAML structure");
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given an empty checksum.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the yaml_checksum field and returns an appropriate error for an empty checksum.
    fn test_new_deployment_object_empty_checksum() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "key: value".to_string(),
            "".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with empty checksum");
        assert_eq!(result.unwrap_err(), "YAML checksum cannot be empty", "Error message should indicate empty YAML checksum");
    }
}