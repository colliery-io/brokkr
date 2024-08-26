//! This module provides a Data Access Layer (DAL) for managing Stack entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, updating, and soft-deleting stacks.

use crate::dal::DAL;
use brokkr_models::models::stacks::{NewStack, Stack};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Represents the Data Access Layer for Stack-related operations.
pub struct StacksDAL<'a> {
    /// Reference to the main DAL instance.
    pub(crate) dal: &'a DAL,
}

impl<'a> StacksDAL<'a> {
    /// Creates a new stack in the database.
    ///
    /// # Arguments
    ///
    /// * `new_stack` - A reference to the NewStack struct containing the stack details.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the created Stack on success, or an error on failure.
    pub fn create(&self, new_stack: &NewStack) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(stacks)
            .values(new_stack)
            .get_result(conn)
    }

    /// Retrieves a stack by its ID.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the Stack if found, or an error if not found or on failure.
    pub fn get_by_id(&self, stack_id: Uuid) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.filter(id.eq(stack_id)).first(conn)
    }

    /// Retrieves all stacks from the database.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing a Vec of all Stacks on success, or an error on failure.
    pub fn get_all(&self) -> QueryResult<Vec<Stack>> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.load(conn)
    }

    /// Updates an existing stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to update.
    /// * `updated_stack` - A reference to the Stack struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the updated Stack on success, or an error on failure.
    pub fn update(&self, stack_id: Uuid, updated_stack: &Stack) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(stacks.filter(id.eq(stack_id)))
            .set(updated_stack)
            .get_result(conn)
    }

    /// Soft deletes a stack by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing the soft-deleted Stack on success, or an error on failure.
    pub fn soft_delete(&self, stack_id: Uuid) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(stacks.filter(id.eq(stack_id)))
            .set(deleted_at.eq(Some(Utc::now())))
            .get_result(conn)
    }

    /// Retrieves all active (non-deleted) stacks from the database.
    ///
    /// # Returns
    ///
    /// Returns a QueryResult containing a Vec of active Stacks on success, or an error on failure.
    pub fn get_active(&self) -> QueryResult<Vec<Stack>> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.filter(deleted_at.is_null()).load(conn)
    }
}
