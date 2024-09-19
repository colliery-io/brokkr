//! # Stack Module
//!
//! This module defines structures and methods for managing stacks in the system.
//!
//! ## Data Model
//!
//! Stacks represent a collection of related deployment objects and configurations.
//! They are stored in the `stacks` table with the following structure:
//!
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMP, when the stack was created
//! - `updated_at`: TIMESTAMP, when the stack was last updated
//! - `deleted_at`: TIMESTAMP, for soft deletion support
//! - `name`: VARCHAR(255), name of the stack
//! - `description`: TEXT, optional description of the stack
//!
//! ## Usage
//!
//! Stacks are core entities in the system, representing a logical grouping of deployment objects.
//! They can be associated with agents, labeled, and annotated for better organization and management.
//!
//! ## Constraints
//!
//! - The `name` must be a non-empty string.
//! - The `description`, if provided, must not be an empty string.
//! - There should be a unique constraint on the `name` field.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a stack in the database.
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
#[diesel(table_name = crate::schema::stacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stack {
    /// Unique identifier for the stack.
    pub id: Uuid,
    /// Timestamp when the stack was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the stack was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp for soft deletion, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// Name of the stack.
    pub name: String,
    /// Optional description of the stack.
    pub description: Option<String>,
}

/// Represents a new stack to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
pub struct NewStack {
    /// Name of the stack.
    pub name: String,
    /// Optional description of the stack.
    pub description: Option<String>,
}

impl NewStack {
    /// Creates a new `NewStack` instance.
    ///
    /// # Parameters
    ///
    /// * `name`: Name of the stack. Must be a non-empty string.
    /// * `description`: Optional description of the stack. If provided, must not be an empty string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewStack)` if all parameters are valid,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(name: String, description: Option<String>) -> Result<Self, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err("Stack name cannot be empty".to_string());
        }

        // Validate description (if provided)
        if let Some(desc) = &description {
            if desc.trim().is_empty() {
                return Err("Stack description cannot be empty if provided".to_string());
            }
        }

        Ok(NewStack { name, description })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_success() {
        let name = "Test Stack".to_string();
        let description = Some("A test stack".to_string());

        let result = NewStack::new(name.clone(), description.clone());

        assert!(
            result.is_ok(),
            "NewStack creation should succeed with valid inputs"
        );
        let new_stack = result.unwrap();
        assert_eq!(new_stack.name, name, "name should match the input value");
        assert_eq!(
            new_stack.description, description,
            "description should match the input value"
        );
    }

    #[test]
    fn test_new_stack_empty_name() {
        let result = NewStack::new("".to_string(), None);
        assert!(
            result.is_err(),
            "NewStack creation should fail with empty name"
        );
        assert_eq!(
            result.unwrap_err(),
            "Stack name cannot be empty",
            "Error message should indicate empty name"
        );
    }

    #[test]
    fn test_new_stack_empty_description() {
        let result = NewStack::new("Valid Name".to_string(), Some("".to_string()));
        assert!(
            result.is_err(),
            "NewStack creation should fail with empty description"
        );
        assert_eq!(
            result.unwrap_err(),
            "Stack description cannot be empty if provided",
            "Error message should indicate empty description"
        );
    }
}
