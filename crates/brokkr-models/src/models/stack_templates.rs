/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Stack Template Module
//!
//! This module defines structures and methods for managing stack templates in the system.
//!
//! ## Data Model
//!
//! Stack templates are reusable definitions for deployment objects with parameter placeholders.
//! They use Tera templating syntax and JSON Schema for parameter validation.
//!
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMP, when the template was created
//! - `updated_at`: TIMESTAMP, when the template was last updated
//! - `deleted_at`: TIMESTAMP, for soft deletion support
//! - `generator_id`: UUID, nullable - NULL means system template (admin-only)
//! - `name`: VARCHAR(255), name of the template
//! - `description`: TEXT, optional description
//! - `version`: INTEGER, version number (auto-incremented per name+generator_id)
//! - `template_content`: TEXT, Tera template content
//! - `parameters_schema`: TEXT, JSON Schema for parameter validation
//! - `checksum`: VARCHAR(64), SHA-256 checksum of template_content
//!
//! ## Constraints
//!
//! - The `name` must be a non-empty string.
//! - The `template_content` must be valid Tera syntax.
//! - The `parameters_schema` must be valid JSON Schema.
//! - Unique constraint on (generator_id, name, version).

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a stack template in the database.
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
#[diesel(table_name = crate::schema::stack_templates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StackTemplate {
    /// Unique identifier for the template.
    pub id: Uuid,
    /// Timestamp when the template was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the template was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp for soft deletion, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// Generator ID - NULL for system templates (admin-only).
    pub generator_id: Option<Uuid>,
    /// Name of the template.
    pub name: String,
    /// Optional description of the template.
    pub description: Option<String>,
    /// Version number (auto-incremented per name+generator_id).
    pub version: i32,
    /// Tera template content.
    pub template_content: String,
    /// JSON Schema for parameter validation.
    pub parameters_schema: String,
    /// SHA-256 checksum of template_content.
    pub checksum: String,
}

/// Represents a new stack template to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::stack_templates)]
pub struct NewStackTemplate {
    /// Generator ID - NULL for system templates (admin-only).
    pub generator_id: Option<Uuid>,
    /// Name of the template.
    pub name: String,
    /// Optional description of the template.
    pub description: Option<String>,
    /// Version number.
    pub version: i32,
    /// Tera template content.
    pub template_content: String,
    /// JSON Schema for parameter validation.
    pub parameters_schema: String,
    /// SHA-256 checksum of template_content.
    pub checksum: String,
}

impl NewStackTemplate {
    /// Creates a new `NewStackTemplate` instance.
    ///
    /// # Parameters
    ///
    /// * `generator_id`: Optional generator ID. NULL means system template.
    /// * `name`: Name of the template. Must be non-empty.
    /// * `description`: Optional description. If provided, must not be empty.
    /// * `version`: Version number for this template.
    /// * `template_content`: Tera template content.
    /// * `parameters_schema`: JSON Schema as a string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewStackTemplate)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    ///
    /// # Note
    ///
    /// This constructor performs basic validation. Tera syntax and JSON Schema
    /// validation should be performed separately before calling this.
    pub fn new(
        generator_id: Option<Uuid>,
        name: String,
        description: Option<String>,
        version: i32,
        template_content: String,
        parameters_schema: String,
    ) -> Result<Self, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        // Validate description (if provided)
        if let Some(desc) = &description {
            if desc.trim().is_empty() {
                return Err("Template description cannot be empty if provided".to_string());
            }
        }

        // Validate template_content is not empty
        if template_content.trim().is_empty() {
            return Err("Template content cannot be empty".to_string());
        }

        // Validate parameters_schema is not empty
        if parameters_schema.trim().is_empty() {
            return Err("Parameters schema cannot be empty".to_string());
        }

        // Validate version is positive
        if version < 1 {
            return Err("Version must be at least 1".to_string());
        }

        // Generate checksum
        let checksum = generate_checksum(&template_content);

        Ok(NewStackTemplate {
            generator_id,
            name,
            description,
            version,
            template_content,
            parameters_schema,
            checksum,
        })
    }
}

/// Generates a SHA-256 checksum for the given content.
pub fn generate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_template_success() {
        let result = NewStackTemplate::new(
            Some(Uuid::new_v4()),
            "test-template".to_string(),
            Some("A test template".to_string()),
            1,
            "apiVersion: v1\nkind: ConfigMap".to_string(),
            r#"{"type": "object"}"#.to_string(),
        );

        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.name, "test-template");
        assert_eq!(template.version, 1);
        assert!(!template.checksum.is_empty());
    }

    #[test]
    fn test_new_stack_template_system_template() {
        let result = NewStackTemplate::new(
            None, // System template
            "system-template".to_string(),
            None,
            1,
            "content".to_string(),
            "{}".to_string(),
        );

        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.generator_id.is_none());
    }

    #[test]
    fn test_new_stack_template_empty_name() {
        let result = NewStackTemplate::new(
            None,
            "".to_string(),
            None,
            1,
            "content".to_string(),
            "{}".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Template name cannot be empty");
    }

    #[test]
    fn test_new_stack_template_empty_content() {
        let result = NewStackTemplate::new(
            None,
            "name".to_string(),
            None,
            1,
            "".to_string(),
            "{}".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Template content cannot be empty");
    }

    #[test]
    fn test_new_stack_template_invalid_version() {
        let result = NewStackTemplate::new(
            None,
            "name".to_string(),
            None,
            0,
            "content".to_string(),
            "{}".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Version must be at least 1");
    }

    #[test]
    fn test_generate_checksum() {
        let content = "test content";
        let checksum = generate_checksum(content);

        // SHA-256 produces 64 hex characters
        assert_eq!(checksum.len(), 64);

        // Same content should produce same checksum
        let checksum2 = generate_checksum(content);
        assert_eq!(checksum, checksum2);

        // Different content should produce different checksum
        let checksum3 = generate_checksum("different content");
        assert_ne!(checksum, checksum3);
    }
}
