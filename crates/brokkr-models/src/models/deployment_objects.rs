//! # Deployment Object Module
//!
//! This module defines structures and methods for managing deployment objects in the system.
//!
//! ## Data Model
//!
//! Deployment objects represent individual deployments or changes to a stack's configuration.
//! They are stored in the `deployment_objects` table with the following structure:
//!
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMP, when the deployment object was created
//! - `updated_at`: TIMESTAMP, when the deployment object was last updated
//! - `deleted_at`: TIMESTAMP, for soft deletion support
//! - `sequence_id`: BIGSERIAL, auto-incrementing sequence number for ordering
//! - `stack_id`: UUID, foreign key referencing the `stacks` table
//! - `yaml_content`: TEXT, the YAML content of the deployment
//! - `yaml_checksum`: VARCHAR(64), SHA-256 checksum of the YAML content
//! - `submitted_at`: TIMESTAMP, when the deployment was submitted
//! - `is_deletion_marker`: BOOLEAN, indicates if this object marks a deletion
//! - `generator_id`: UUID, ID of the generator associated with this deployment object
//!
//! ## Usage
//!
//! Deployment objects are used to track changes and updates to stack configurations over time.
//! They provide a historical record of deployments and can be used for rollbacks, audits, and
//! tracking the evolution of a stack's configuration.
//!
//! ## Constraints
//!
//! - The `stack_id` must be a valid, non-nil UUID.
//! - The `yaml_content` must not be empty.
//! - The `yaml_checksum` is automatically generated from the `yaml_content`.
//! - Deployment objects are designed to be immutable after creation, with exceptions for soft deletion.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a deployment object in the database.
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
#[diesel(table_name = crate::schema::deployment_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeploymentObject {
    /// Unique identifier for the deployment object.
    pub id: Uuid,
    /// Timestamp when the deployment object was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the deployment object was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp for soft deletion, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// Auto-incrementing sequence number for ordering.
    pub sequence_id: i64,
    /// ID of the stack this deployment object belongs to.
    pub stack_id: Uuid,
    /// YAML content of the deployment.
    pub yaml_content: String,
    /// SHA-256 checksum of the YAML content.
    pub yaml_checksum: String,
    /// Timestamp when the deployment was submitted.
    pub submitted_at: DateTime<Utc>,
    /// Indicates if this object marks a deletion.
    pub is_deletion_marker: bool,
    /// ID of the generator associated with this deployment object.
    pub generator_id: Uuid,
}

/// Represents a new deployment object to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
pub struct NewDeploymentObject {
    /// ID of the stack this deployment object belongs to.
    pub stack_id: Uuid,
    /// YAML content of the deployment.
    pub yaml_content: String,
    /// SHA-256 checksum of the YAML content.
    pub yaml_checksum: String,
    /// Indicates if this object marks a deletion.
    pub is_deletion_marker: bool,
    /// ID of the generator associated with this deployment object.
    pub generator_id: Uuid,
}

impl NewDeploymentObject {
    /// Creates a new `NewDeploymentObject` instance.
    ///
    /// # Parameters
    ///
    /// * `stack_id`: UUID of the stack this deployment object belongs to.
    /// * `yaml_content`: YAML content of the deployment. Must be a non-empty string.
    /// * `is_deletion_marker`: Boolean indicating if this object marks a deletion.
    /// * `generator_id`: UUID of the generator associated with this deployment object.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewDeploymentObject)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        stack_id: Uuid,
        yaml_content: String,
        is_deletion_marker: bool,
        generator_id: Uuid,
    ) -> Result<Self, String> {
        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        // Validate yaml_content
        if yaml_content.trim().is_empty() {
            return Err("YAML content cannot be empty".to_string());
        }

        // Generate SHA-256 checksum
        let yaml_checksum = generate_checksum(&yaml_content);

        Ok(NewDeploymentObject {
            stack_id,
            yaml_content,
            yaml_checksum,
            is_deletion_marker,
            generator_id,
        })
    }
}

/// Helper function to generate SHA-256 checksum for YAML content.
fn generate_checksum(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deployment_object_success() {
        let stack_id = Uuid::new_v4();
        let yaml_content = "key: value\nother_key: other_value".to_string();
        let is_deletion_marker = false;
        let generator_id = Uuid::new_v4();

        let result = NewDeploymentObject::new(stack_id, yaml_content.clone(), is_deletion_marker, generator_id);

        assert!(
            result.is_ok(),
            "NewDeploymentObject creation should succeed with valid inputs"
        );
        let new_obj = result.unwrap();
        assert_eq!(new_obj.stack_id, stack_id);
        assert_eq!(new_obj.yaml_content, yaml_content);
        assert_eq!(new_obj.is_deletion_marker, is_deletion_marker);
        assert_eq!(new_obj.generator_id, generator_id);
    }

    #[test]
    fn test_new_deployment_object_invalid_stack_id() {
        let result = NewDeploymentObject::new(Uuid::nil(), "key: value".to_string(), false, Uuid::nil());
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
    fn test_new_deployment_object_empty_yaml() {
        let result = NewDeploymentObject::new(Uuid::new_v4(), "".to_string(), false, Uuid::nil());
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
}
