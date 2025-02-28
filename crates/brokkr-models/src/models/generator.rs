//! # Generator Module
//!
//! This module defines structures and methods for managing generators in the Brokkr system.
//!
//! ## Data Model
//!
//! Generators are entities responsible for creating and managing stacks. They are stored in the `generators` table
//! with the following structure:
//!
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMPTZ, creation timestamp
//! - `updated_at`: TIMESTAMPTZ, last update timestamp
//! - `deleted_at`: TIMESTAMPTZ, soft deletion timestamp (nullable)
//! - `name`: VARCHAR(255), name of the generator
//! - `description`: TEXT, optional description of the generator
//! - `pak_hash`: TEXT, hash of the Pre-Authentication Key (PAK) for the generator (nullable)
//! - `last_active_at`: TIMESTAMPTZ, timestamp of the last activity (nullable)
//! - `is_active`: BOOLEAN, indicates if the generator is currently active
//!
//! ## Usage
//!
//! Generators are used to create and manage stacks in the Brokkr system. They can be authenticated
//! using their PAK and can perform operations on their associated stacks.
//!
//! ## Constraints
//!
//! - The `name` must be a non-empty string and cannot exceed 255 characters.
//! - The `pak_hash` must be a valid hash when provided.
//! - The `is_active` flag determines whether the generator can perform operations.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a generator in the Brokkr system.
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
#[diesel(table_name = crate::schema::generators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Generator {
    /// Unique identifier for the generator.
    pub id: Uuid,
    /// Timestamp of when the generator was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the generator was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp of when the generator was deleted, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// Name of the generator.
    pub name: String,
    /// Optional description of the generator.
    pub description: Option<String>,
    /// Hash of the Pre-Authentication Key (PAK) for the generator.
    #[serde(skip_serializing, skip_deserializing)]
    pub pak_hash: Option<String>,
    /// Timestamp of when the generator was last active.
    pub last_active_at: Option<DateTime<Utc>>,
    /// Indicates whether the generator is currently active.
    pub is_active: bool,
}

/// Represents the data required to create a new generator.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::generators)]
pub struct NewGenerator {
    /// Name of the new generator.
    pub name: String,
    /// Optional description of the new generator.
    pub description: Option<String>,
}

impl NewGenerator {
    /// Creates a new `NewGenerator` instance.
    ///
    /// # Parameters
    ///
    /// * `name`: The name for the generator. Must be non-empty and not exceed 255 characters.
    /// * `description`: An optional description for the generator.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewGenerator)` if all parameters are valid, otherwise returns an `Err` with a description of the validation failure.
    pub fn new(name: String, description: Option<String>) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Generator name cannot be empty".to_string());
        }

        if name.len() > 255 {
            return Err("Generator name cannot exceed 255 characters".to_string());
        }

        Ok(NewGenerator { name, description })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests successful creation of a new generator.
    #[test]
    fn test_new_generator_success() {
        let name = "Test Generator".to_string();
        let description = Some("A test generator".to_string());

        let result = NewGenerator::new(name.clone(), description.clone());

        assert!(
            result.is_ok(),
            "NewGenerator creation should succeed with valid inputs"
        );
        let new_generator = result.unwrap();
        assert_eq!(new_generator.name, name);
        assert_eq!(new_generator.description, description);
    }

    /// Tests failure when creating a new generator with an empty name.
    #[test]
    fn test_new_generator_empty_name() {
        let result = NewGenerator::new("".to_string(), None);
        assert!(
            result.is_err(),
            "NewGenerator creation should fail with empty name"
        );
        assert_eq!(
            result.unwrap_err(),
            "Generator name cannot be empty",
            "Error message should indicate empty name"
        );
    }
}
