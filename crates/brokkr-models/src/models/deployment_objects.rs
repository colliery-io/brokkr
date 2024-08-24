// src/models/deployment_objects.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
#[diesel(primary_key(uuid))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeploymentObject {
    pub uuid: Uuid,
    pub sequence_id: i64,
    pub stack_id: Uuid,
    pub yaml_content: String,
    pub yaml_checksum: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub submitted_at: DateTime<Utc>,
    pub is_deletion_marker: bool,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::deployment_objects)]
pub struct NewDeploymentObject {
    pub uuid: Uuid,
    pub stack_id: Uuid,
    pub yaml_content: String,
    pub yaml_checksum: String,
    pub submitted_at: DateTime<Utc>,
    pub is_deletion_marker: bool,
}

impl NewDeploymentObject {
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

        assert!(result.is_ok());
        let new_obj = result.unwrap();
        assert_eq!(new_obj.stack_id, stack_id);
        assert_eq!(new_obj.yaml_content, yaml_content);
        assert_eq!(new_obj.yaml_checksum, yaml_checksum);
        assert_eq!(new_obj.is_deletion_marker, is_deletion_marker);
        assert!(!new_obj.uuid.is_nil());
    }

    #[test]
    fn test_new_deployment_object_invalid_stack_id() {
        let result = NewDeploymentObject::new(
            Uuid::nil(),
            "key: value".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid stack ID");
    }

    #[test]
    fn test_new_deployment_object_empty_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "YAML content cannot be empty");
    }

    #[test]
    fn test_new_deployment_object_invalid_yaml() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "invalid yaml content".to_string(),
            "checksum".to_string(),
            false,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid YAML structure");
    }

    #[test]
    fn test_new_deployment_object_empty_checksum() {
        let result = NewDeploymentObject::new(
            Uuid::new_v4(),
            "key: value".to_string(),
            "".to_string(),
            false,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "YAML checksum cannot be empty");
    }

}