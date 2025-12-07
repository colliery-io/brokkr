/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Template Target Module
//!
//! This module defines structures and methods for managing template targets in the system.
//!
//! ## Data Model
//!
//! Template targets represent the association between templates and stacks. They track
//! which stacks a template is compatible with based on label matching at the time
//! of template creation.
//!
//! - `id`: UUID, primary key
//! - `template_id`: UUID, foreign key referencing the `stack_templates` table
//! - `stack_id`: UUID, foreign key referencing the `stacks` table
//! - `created_at`: TIMESTAMP, when the target association was created
//!
//! ## Usage
//!
//! Template targets are used to define which stacks a template can be instantiated into.
//! When a template is created with labels, the system computes compatible stacks and
//! stores these associations. This enables efficient querying of valid template-stack
//! combinations without recomputing label matches on every request.
//!
//! ## Constraints
//!
//! - Both `template_id` and `stack_id` must be valid, non-nil UUIDs.
//! - There is a unique constraint on the combination of `template_id` and `stack_id` to prevent
//!   duplicate associations.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a template target in the database.
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
#[diesel(table_name = crate::schema::template_targets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TemplateTarget {
    /// Unique identifier for the template target.
    pub id: Uuid,
    /// ID of the template associated with this target.
    pub template_id: Uuid,
    /// ID of the stack associated with this target.
    pub stack_id: Uuid,
    /// Timestamp when the target association was created.
    pub created_at: DateTime<Utc>,
}

/// Represents a new template target to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::template_targets)]
pub struct NewTemplateTarget {
    /// ID of the template to associate with a stack.
    pub template_id: Uuid,
    /// ID of the stack to associate with a template.
    pub stack_id: Uuid,
}

impl NewTemplateTarget {
    /// Creates a new `NewTemplateTarget` instance.
    ///
    /// # Parameters
    ///
    /// * `template_id`: UUID of the template to associate with a stack.
    /// * `stack_id`: UUID of the stack to associate with a template.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewTemplateTarget)` if both UUIDs are valid and non-nil,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(template_id: Uuid, stack_id: Uuid) -> Result<Self, String> {
        // Validate template_id
        if template_id.is_nil() {
            return Err("Invalid template ID".to_string());
        }

        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        Ok(NewTemplateTarget {
            template_id,
            stack_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_template_target_success() {
        let template_id = Uuid::new_v4();
        let stack_id = Uuid::new_v4();

        let result = NewTemplateTarget::new(template_id, stack_id);

        assert!(
            result.is_ok(),
            "NewTemplateTarget creation should succeed with valid inputs"
        );
        let new_target = result.unwrap();
        assert_eq!(
            new_target.template_id, template_id,
            "template_id should match the input value"
        );
        assert_eq!(
            new_target.stack_id, stack_id,
            "stack_id should match the input value"
        );
    }

    #[test]
    fn test_new_template_target_invalid_template_id() {
        let result = NewTemplateTarget::new(Uuid::nil(), Uuid::new_v4());
        assert!(
            result.is_err(),
            "NewTemplateTarget creation should fail with nil template ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid template ID",
            "Error message should indicate invalid template ID"
        );
    }

    #[test]
    fn test_new_template_target_invalid_stack_id() {
        let result = NewTemplateTarget::new(Uuid::new_v4(), Uuid::nil());
        assert!(
            result.is_err(),
            "NewTemplateTarget creation should fail with nil stack ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid stack ID",
            "Error message should indicate invalid stack ID"
        );
    }
}
