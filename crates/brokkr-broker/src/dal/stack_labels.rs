/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Stack Label operations.
//!
//! This module provides functionality to interact with stack labels in the database,
//! including creating, retrieving, listing, and deleting labels.

use crate::dal::DAL;
use brokkr_models::models::stack_labels::{NewStackLabel, StackLabel};
use brokkr_models::schema::stack_labels;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for Stack Labels.
pub struct StackLabelsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> StackLabelsDAL<'a> {
    /// Creates a new stack label in the database.
    ///
    /// # Arguments
    ///
    /// * `new_label` - The new label details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `StackLabel` or a database error.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn create(&self, new_label: &NewStackLabel) -> Result<StackLabel, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(stack_labels::table)
            .values(new_label)
            .get_result(conn)
    }

    /// Retrieves a stack label by its ID.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the label to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<StackLabel>` if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn get(&self, label_id: Uuid) -> Result<Option<StackLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_labels::table
            .filter(stack_labels::id.eq(label_id))
            .first(conn)
            .optional()
    }

    /// Lists all labels for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack whose labels to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `StackLabel`s associated with the specified stack.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list_for_stack(&self, stack_id: Uuid) -> Result<Vec<StackLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_labels::table
            .filter(stack_labels::stack_id.eq(stack_id))
            .load::<StackLabel>(conn)
    }

    /// Deletes a stack label from the database.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the label to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1).
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete(&self, label_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_labels::table.filter(stack_labels::id.eq(label_id))).execute(conn)
    }

    /// Deletes all labels for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack whose labels to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete_all_for_stack(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_labels::table.filter(stack_labels::stack_id.eq(stack_id)))
            .execute(conn)
    }
}
