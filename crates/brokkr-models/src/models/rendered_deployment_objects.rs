/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Rendered Deployment Object Module
//!
//! This module defines structures and methods for tracking the provenance of
//! deployment objects created from templates.
//!
//! ## Data Model
//!
//! Rendered deployment objects track the relationship between a deployment object
//! and the template instance that created it. This provides provenance information
//! for auditing, debugging, and reconstruction purposes.
//!
//! - `id`: UUID, primary key
//! - `deployment_object_id`: UUID, foreign key referencing the `deployment_objects` table
//! - `template_id`: UUID, foreign key referencing the `stack_templates` table
//! - `template_version`: INTEGER, version of the template used (snapshot)
//! - `template_parameters`: TEXT, JSON string of parameters used for rendering
//! - `created_at`: TIMESTAMP, when the rendering occurred
//!
//! ## Usage
//!
//! When a template is instantiated with parameters, the system:
//! 1. Validates parameters against the template's JSON Schema
//! 2. Renders the Tera template with the parameters
//! 3. Creates a deployment object with the rendered YAML
//! 4. Creates a rendered_deployment_object record to track the provenance
//!
//! This allows reconstruction of how any deployment object was created and
//! enables re-rendering with different parameters if needed.
//!
//! ## Constraints
//!
//! - `deployment_object_id` must be a valid, non-nil UUID.
//! - `template_id` must be a valid, non-nil UUID.
//! - `template_version` must be at least 1.
//! - `template_parameters` must be a valid JSON string.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a rendered deployment object provenance record in the database.
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
    ToSchema,
)]
#[diesel(table_name = crate::schema::rendered_deployment_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RenderedDeploymentObject {
    /// Unique identifier for this provenance record.
    pub id: Uuid,
    /// ID of the deployment object that was created.
    pub deployment_object_id: Uuid,
    /// ID of the template used to create the deployment object.
    pub template_id: Uuid,
    /// Version of the template at the time of rendering (snapshot).
    pub template_version: i32,
    /// JSON string of parameters used for rendering.
    pub template_parameters: String,
    /// Timestamp when the rendering occurred.
    pub created_at: DateTime<Utc>,
}

/// Represents a new rendered deployment object provenance record to be inserted.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::rendered_deployment_objects)]
pub struct NewRenderedDeploymentObject {
    /// ID of the deployment object that was created.
    pub deployment_object_id: Uuid,
    /// ID of the template used to create the deployment object.
    pub template_id: Uuid,
    /// Version of the template at the time of rendering (snapshot).
    pub template_version: i32,
    /// JSON string of parameters used for rendering.
    pub template_parameters: String,
}

impl NewRenderedDeploymentObject {
    /// Creates a new `NewRenderedDeploymentObject` instance.
    ///
    /// # Parameters
    ///
    /// * `deployment_object_id`: UUID of the deployment object created from this rendering.
    /// * `template_id`: UUID of the template used for rendering.
    /// * `template_version`: Version number of the template used.
    /// * `template_parameters`: JSON string of the parameters used for rendering.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewRenderedDeploymentObject)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        deployment_object_id: Uuid,
        template_id: Uuid,
        template_version: i32,
        template_parameters: String,
    ) -> Result<Self, String> {
        // Validate deployment_object_id
        if deployment_object_id.is_nil() {
            return Err("Invalid deployment object ID".to_string());
        }

        // Validate template_id
        if template_id.is_nil() {
            return Err("Invalid template ID".to_string());
        }

        // Validate template_version
        if template_version < 1 {
            return Err("Template version must be at least 1".to_string());
        }

        // Validate template_parameters is valid JSON
        if serde_json::from_str::<serde_json::Value>(&template_parameters).is_err() {
            return Err("Template parameters must be valid JSON".to_string());
        }

        Ok(NewRenderedDeploymentObject {
            deployment_object_id,
            template_id,
            template_version,
            template_parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rendered_deployment_object_success() {
        let deployment_object_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let template_version = 1;
        let template_parameters = r#"{"namespace": "default", "replicas": 3}"#.to_string();

        let result = NewRenderedDeploymentObject::new(
            deployment_object_id,
            template_id,
            template_version,
            template_parameters.clone(),
        );

        assert!(
            result.is_ok(),
            "NewRenderedDeploymentObject creation should succeed with valid inputs"
        );
        let record = result.unwrap();
        assert_eq!(record.deployment_object_id, deployment_object_id);
        assert_eq!(record.template_id, template_id);
        assert_eq!(record.template_version, template_version);
        assert_eq!(record.template_parameters, template_parameters);
    }

    #[test]
    fn test_new_rendered_deployment_object_invalid_deployment_object_id() {
        let result = NewRenderedDeploymentObject::new(
            Uuid::nil(),
            Uuid::new_v4(),
            1,
            "{}".to_string(),
        );
        assert!(
            result.is_err(),
            "Should fail with nil deployment object ID"
        );
        assert_eq!(result.unwrap_err(), "Invalid deployment object ID");
    }

    #[test]
    fn test_new_rendered_deployment_object_invalid_template_id() {
        let result = NewRenderedDeploymentObject::new(
            Uuid::new_v4(),
            Uuid::nil(),
            1,
            "{}".to_string(),
        );
        assert!(result.is_err(), "Should fail with nil template ID");
        assert_eq!(result.unwrap_err(), "Invalid template ID");
    }

    #[test]
    fn test_new_rendered_deployment_object_invalid_version() {
        let result = NewRenderedDeploymentObject::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            0,
            "{}".to_string(),
        );
        assert!(result.is_err(), "Should fail with version less than 1");
        assert_eq!(result.unwrap_err(), "Template version must be at least 1");
    }

    #[test]
    fn test_new_rendered_deployment_object_invalid_json() {
        let result = NewRenderedDeploymentObject::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            "not valid json".to_string(),
        );
        assert!(result.is_err(), "Should fail with invalid JSON");
        assert_eq!(result.unwrap_err(), "Template parameters must be valid JSON");
    }

    #[test]
    fn test_new_rendered_deployment_object_empty_json_object() {
        let result = NewRenderedDeploymentObject::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            "{}".to_string(),
        );
        assert!(result.is_ok(), "Empty JSON object should be valid");
    }
}
