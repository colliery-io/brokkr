//! # Deployment Objects Module
//!
//! This module defines the data structures and operations for deployment objects in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for deployment objects is represented by the `DeploymentObject` struct:
//!
//! - `id`: Uuid - Unique identifier for the deployment object
//! - `created_at`: DateTime<Utc> - Timestamp when the deployment object was created
//! - `updated_at`: DateTime<Utc> - Timestamp when the deployment object was last updated
//! - `deleted_at`: Option<DateTime<Utc>> - Timestamp when the object was soft-deleted (if applicable)
//! - `sequence_id`: i64 - Sequential identifier for ordering deployment objects
//! - `stack_id`: Uuid - Identifier of the stack this deployment object belongs to
//! - `yaml_content`: String - The YAML content of the deployment object
//! - `yaml_checksum`: String - Checksum of the YAML content for integrity verification
//! - `submitted_at`: DateTime<Utc> - Timestamp when the deployment object was submitted
//! - `is_deletion_marker`: bool - Flag indicating if this object marks a deletion
//!
//! The `NewDeploymentObject` struct is used for creating new deployment objects and contains a subset of the fields
//! from `DeploymentObject`: `stack_id`, `yaml_content`, `yaml_checksum`, and `is_deletion_marker`. The other fields are
//! managed by the database or set after creation.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Represents a deployment object in the system.
///
/// This struct is used for querying existing deployment objects from the database.
#[derive(
    Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = crate::schema::deployment_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeploymentObjectDB {
    /// Unique identifier for the deployment object
    pub id: Uuid,
    /// Timestamp when the deployment object was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the deployment object was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the object was soft-deleted (if applicable)
    pub deleted_at: Option<DateTime<Utc>>,
    /// Sequential identifier for ordering deployment objects
    pub sequence_id: i64,
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

/// Represents a new deployment object to be inserted into the database.
///
/// This struct is used when creating new deployment objects.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
pub struct NewDeploymentObjectDB {
    /// Identifier of the stack this deployment object belongs to
    pub stack_id: Uuid,
    /// The YAML content of the deployment object
    pub yaml_content: String,
    /// Checksum of the YAML content for integrity verification
    pub yaml_checksum: String,
    /// Flag indicating if this object marks a deletion
    pub is_deletion_marker: bool,
}

impl NewDeploymentObjectDB {
    /// Creates a new `NewDeploymentObject` instance.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - UUID of the stack this deployment object belongs to
    /// * `yaml_content` - YAML content of the deployment object
    /// * `is_deletion_marker` - Flag indicating if this object marks a deletion
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewDeploymentObject` instance if successful, or an error message if validation fails.
    pub fn new(
        stack_id: Uuid,
        yaml_content: String,
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

        // Attempt to parse the YAML
        if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
            return Err(format!("Invalid YAML: {}", e));
        }

        // Generate SHA-256 checksum
        let yaml_checksum = generate_checksum(&yaml_content);

        Ok(NewDeploymentObjectDB {
            stack_id,
            yaml_content,
            yaml_checksum,
            is_deletion_marker,
        })
    }
}

/// Generates a SHA-256 checksum for the given content.
///
/// # Arguments
///
/// * `content` - The string content to generate a checksum for
///
/// # Returns
///
/// A string representation of the SHA-256 checksum

fn generate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the successful creation of a NewDeploymentObject with valid input parameters.
    ///
    /// This test:
    /// 1. Creates a new NewDeploymentObject with specific stack_id, yaml_content, and is_deletion_marker.
    /// 2. Verifies that each field of the created object matches the input values.
    /// 3. Checks that the yaml_checksum is generated and not empty.
    ///
    /// It ensures that the NewDeploymentObject::new() method correctly sets all fields
    /// and properly generates the checksum.
    fn test_new_deployment_object_success() {
        let stack_id = Uuid::new_v4();
        let yaml_content = "key: value\nother_key: other_value".to_string();
        let is_deletion_marker = false;

        let result = NewDeploymentObjectDB::new(stack_id, yaml_content.clone(), is_deletion_marker);

        assert!(
            result.is_ok(),
            "NewDeploymentObject creation should succeed with valid inputs"
        );
        let new_obj = result.unwrap();
        assert_eq!(
            new_obj.stack_id, stack_id,
            "stack_id should match the input value"
        );
        assert_eq!(
            new_obj.yaml_content, yaml_content,
            "yaml_content should match the input value"
        );
        assert!(
            !new_obj.yaml_checksum.is_empty(),
            "yaml_checksum should not be empty"
        );
        assert_eq!(
            new_obj.is_deletion_marker, is_deletion_marker,
            "is_deletion_marker should match the input value"
        );
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given an invalid stack_id.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the stack_id and returns an appropriate error.
    fn test_new_deployment_object_invalid_stack_id() {
        let result = NewDeploymentObjectDB::new(Uuid::nil(), "key: value".to_string(), false);
        assert!(
            result.is_err(),
            "NewDeploymentObject creation should fail with nil stack ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid stack ID",
            "Error message should indicate invalid stack ID"
        );
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given empty YAML content.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the yaml_content and returns an appropriate error if it's empty.
    fn test_new_deployment_object_empty_yaml() {
        let result = NewDeploymentObjectDB::new(Uuid::new_v4(), "".to_string(), false);
        assert!(
            result.is_err(),
            "NewDeploymentObject creation should fail with empty YAML content"
        );
        assert_eq!(
            result.unwrap_err(),
            "YAML content cannot be empty",
            "Error message should indicate empty YAML content"
        );
    }

    #[test]
    /// Tests that NewDeploymentObject creation fails when given invalid YAML content.
    ///
    /// This test ensures that the NewDeploymentObject::new() method properly
    /// validates the yaml_content and returns an appropriate error if it's not valid YAML.
    fn test_new_deployment_object_invalid_yaml() {
        let result = NewDeploymentObjectDB::new(
            Uuid::new_v4(),
            "key: : value".to_string(), // This is invalid YAML
            false,
        );
        assert!(
            result.is_err(),
            "NewDeploymentObject creation should fail with invalid YAML content"
        );
        let err = result.unwrap_err();
        assert!(
            err.starts_with("Invalid YAML:"),
            "Error message should start with 'Invalid YAML:'. Got: {}",
            err
        );
    }

    #[test]
    /// Tests that the checksum generation produces consistent and non-empty results.
    ///
    /// This test ensures that the generate_checksum() function:
    /// 1. Produces the same checksum for identical content
    /// 2. Produces a non-empty checksum

    fn test_checksum_generation() {
        let yaml_content = "key: value\nother_key: other_value".to_string();
        let checksum1 = generate_checksum(&yaml_content);
        let checksum2 = generate_checksum(&yaml_content);

        assert_eq!(
            checksum1, checksum2,
            "Checksums for the same content should be identical"
        );
        assert!(!checksum1.is_empty(), "Checksum should not be empty");
    }
}
