/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Template Label Module
//!
//! This module defines structures and methods for managing template labels.
//! Template labels are used for targeting validation - templates with labels
//! can only be instantiated into stacks that have matching labels.
//!
//! ## Constraints
//!
//! - The `template_id` must be a valid, non-nil UUID.
//! - The `label` must be a non-empty string.
//! - The `label` must not exceed 64 characters.
//! - The `label` cannot contain whitespace.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a template label in the database.
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
#[diesel(table_name = crate::schema::template_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TemplateLabel {
    /// Unique identifier for the template label.
    pub id: Uuid,
    /// ID of the template this label is associated with.
    pub template_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
    /// Timestamp when the label was created.
    pub created_at: DateTime<Utc>,
}

/// Represents a new template label to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::template_labels)]
pub struct NewTemplateLabel {
    /// ID of the template this label is associated with.
    pub template_id: Uuid,
    /// The label text (max 64 characters, no whitespace).
    pub label: String,
}

impl NewTemplateLabel {
    /// Creates a new `NewTemplateLabel` instance.
    ///
    /// # Parameters
    ///
    /// * `template_id`: UUID of the template to associate the label with.
    /// * `label`: The label text. Must be non-empty, max 64 characters, no whitespace.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewTemplateLabel)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(template_id: Uuid, label: String) -> Result<Self, String> {
        // Validate template_id
        if template_id.is_nil() {
            return Err("Invalid template ID".to_string());
        }

        // Validate label
        if label.trim().is_empty() {
            return Err("Label cannot be empty".to_string());
        }

        // Check label length
        if label.len() > 64 {
            return Err("Label cannot exceed 64 characters".to_string());
        }

        // Check whitespace
        if label.contains(char::is_whitespace) {
            return Err("Label cannot contain whitespace".to_string());
        }

        Ok(NewTemplateLabel { template_id, label })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_template_label_success() {
        let template_id = Uuid::new_v4();
        let label = "env=prod".to_string();

        let result = NewTemplateLabel::new(template_id, label.clone());

        assert!(result.is_ok());
        let new_label = result.unwrap();
        assert_eq!(new_label.template_id, template_id);
        assert_eq!(new_label.label, label);
    }

    #[test]
    fn test_new_template_label_invalid_template_id() {
        let result = NewTemplateLabel::new(Uuid::nil(), "test-label".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid template ID");
    }

    #[test]
    fn test_new_template_label_empty_label() {
        let result = NewTemplateLabel::new(Uuid::new_v4(), "".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Label cannot be empty");
    }

    #[test]
    fn test_new_template_label_whitespace_label() {
        let result = NewTemplateLabel::new(Uuid::new_v4(), "has space".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Label cannot contain whitespace");
    }

    #[test]
    fn test_new_template_label_too_long() {
        let long_label = "a".repeat(65);
        let result = NewTemplateLabel::new(Uuid::new_v4(), long_label);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Label cannot exceed 64 characters");
    }

    #[test]
    fn test_new_template_label_max_length() {
        let max_label = "a".repeat(64);
        let result = NewTemplateLabel::new(Uuid::new_v4(), max_label);
        assert!(result.is_ok());
    }
}
