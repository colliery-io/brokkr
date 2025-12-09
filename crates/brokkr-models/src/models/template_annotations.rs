/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Template Annotation Module
//!
//! This module defines structures and methods for managing template annotations.
//! Template annotations are key-value pairs used for targeting validation.
//!
//! ## Constraints
//!
//! - The `template_id` must be a valid, non-nil UUID.
//! - The `key` must be a non-empty string, max 64 characters, no whitespace.
//! - The `value` must be a non-empty string, max 64 characters, no whitespace.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a template annotation in the database.
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
#[diesel(table_name = crate::schema::template_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TemplateAnnotation {
    /// Unique identifier for the template annotation.
    pub id: Uuid,
    /// ID of the template this annotation is associated with.
    pub template_id: Uuid,
    /// The annotation key (max 64 characters, no whitespace).
    pub key: String,
    /// The annotation value (max 64 characters, no whitespace).
    pub value: String,
    /// Timestamp when the annotation was created.
    pub created_at: DateTime<Utc>,
}

/// Represents a new template annotation to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::template_annotations)]
pub struct NewTemplateAnnotation {
    /// ID of the template this annotation is associated with.
    pub template_id: Uuid,
    /// The annotation key (max 64 characters, no whitespace).
    pub key: String,
    /// The annotation value (max 64 characters, no whitespace).
    pub value: String,
}

impl NewTemplateAnnotation {
    /// Creates a new `NewTemplateAnnotation` instance.
    ///
    /// # Parameters
    ///
    /// * `template_id`: UUID of the template to associate the annotation with.
    /// * `key`: The annotation key. Must be non-empty, max 64 characters, no whitespace.
    /// * `value`: The annotation value. Must be non-empty, max 64 characters, no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewTemplateAnnotation)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(template_id: Uuid, key: String, value: String) -> Result<Self, String> {
        // Validate template_id
        if template_id.is_nil() {
            return Err("Invalid template ID".to_string());
        }

        // Validate key
        if key.trim().is_empty() {
            return Err("Annotation key cannot be empty".to_string());
        }
        if key.len() > 64 {
            return Err("Annotation key cannot exceed 64 characters".to_string());
        }
        if key.contains(char::is_whitespace) {
            return Err("Annotation key cannot contain whitespace".to_string());
        }

        // Validate value
        if value.trim().is_empty() {
            return Err("Annotation value cannot be empty".to_string());
        }
        if value.len() > 64 {
            return Err("Annotation value cannot exceed 64 characters".to_string());
        }
        if value.contains(char::is_whitespace) {
            return Err("Annotation value cannot contain whitespace".to_string());
        }

        Ok(NewTemplateAnnotation {
            template_id,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_template_annotation_success() {
        let template_id = Uuid::new_v4();
        let key = "team".to_string();
        let value = "platform".to_string();

        let result = NewTemplateAnnotation::new(template_id, key.clone(), value.clone());

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.template_id, template_id);
        assert_eq!(annotation.key, key);
        assert_eq!(annotation.value, value);
    }

    #[test]
    fn test_new_template_annotation_invalid_template_id() {
        let result = NewTemplateAnnotation::new(Uuid::nil(), "key".to_string(), "value".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid template ID");
    }

    #[test]
    fn test_new_template_annotation_empty_key() {
        let result =
            NewTemplateAnnotation::new(Uuid::new_v4(), "".to_string(), "value".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotation key cannot be empty");
    }

    #[test]
    fn test_new_template_annotation_empty_value() {
        let result = NewTemplateAnnotation::new(Uuid::new_v4(), "key".to_string(), "".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Annotation value cannot be empty");
    }

    #[test]
    fn test_new_template_annotation_key_with_whitespace() {
        let result =
            NewTemplateAnnotation::new(Uuid::new_v4(), "has space".to_string(), "value".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Annotation key cannot contain whitespace"
        );
    }

    #[test]
    fn test_new_template_annotation_value_with_whitespace() {
        let result =
            NewTemplateAnnotation::new(Uuid::new_v4(), "key".to_string(), "has space".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Annotation value cannot contain whitespace"
        );
    }

    #[test]
    fn test_new_template_annotation_key_too_long() {
        let long_key = "a".repeat(65);
        let result =
            NewTemplateAnnotation::new(Uuid::new_v4(), long_key, "value".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Annotation key cannot exceed 64 characters"
        );
    }

    #[test]
    fn test_new_template_annotation_value_too_long() {
        let long_value = "a".repeat(65);
        let result =
            NewTemplateAnnotation::new(Uuid::new_v4(), "key".to_string(), long_value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Annotation value cannot exceed 64 characters"
        );
    }
}
