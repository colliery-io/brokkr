use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use uuid::Uuid;
use sha2::{Sha256, Digest};

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

/// Generates a SHA-256 checksum for the given content.
fn generate_checksum(content: &str) -> String {
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

        let result = NewDeploymentObject::new(
            stack_id,
            yaml_content.clone(),
            is_deletion_marker,
        );

        assert!(result.is_ok(), "NewDeploymentObject creation should succeed with valid inputs");
        let new_obj = result.unwrap();
        assert_eq!(new_obj.stack_id, stack_id, "stack_id should match the input value");
        assert_eq!(new_obj.yaml_content, yaml_content, "yaml_content should match the input value");
        assert!(!new_obj.yaml_checksum.is_empty(), "yaml_checksum should not be empty");
        assert_eq!(new_obj.is_deletion_marker, is_deletion_marker, "is_deletion_marker should match the input value");
        assert!(!new_obj.uuid.is_nil(), "A non-nil UUID should be generated");
    }

    #[test]
    fn test_new_deployment_object_invalid_stack_id() {
        let result = NewDeploymentObject::new(
            Uuid::nil(),
            "key: value".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with nil stack ID");
        assert_eq!(result.unwrap_err(), "Invalid stack ID", "Error message should indicate invalid stack ID");
    }

    #[test]
    fn test_new_deployment_object_empty_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "".to_string(),
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with empty YAML content");
        assert_eq!(result.unwrap_err(), "YAML content cannot be empty", "Error message should indicate empty YAML content");
    }

    #[test]
    fn test_new_deployment_object_invalid_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "key: : value".to_string(),  // This is invalid YAML
            false,
        );
        assert!(result.is_err(), "NewDeploymentObject creation should fail with invalid YAML content");
        let err = result.unwrap_err();
        assert!(err.starts_with("Invalid YAML:"), "Error message should start with 'Invalid YAML:'. Got: {}", err);
    }

    #[test]
    fn test_checksum_generation() {
        let yaml_content = "key: value\nother_key: other_value".to_string();
        let checksum1 = generate_checksum(&yaml_content);
        let checksum2 = generate_checksum(&yaml_content);

        assert_eq!(checksum1, checksum2, "Checksums for the same content should be identical");
        assert!(!checksum1.is_empty(), "Checksum should not be empty");
    }
}