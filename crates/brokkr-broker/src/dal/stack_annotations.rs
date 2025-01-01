//! Data Access Layer for Stack Annotation operations.
//!
//! This module provides functionality to interact with stack annotations in the database,
//! including creating, retrieving, updating, and deleting annotations.

use crate::dal::DAL;
use brokkr_models::models::stack_annotations::{NewStackAnnotation, StackAnnotation};
use brokkr_models::schema::stack_annotations;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for Stack Annotations.
pub struct StackAnnotationsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> StackAnnotationsDAL<'a> {
    /// Creates a new stack annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - The new annotation details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `StackAnnotation` or a database error.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn create(
        &self,
        new_annotation: &NewStackAnnotation,
    ) -> Result<StackAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(stack_annotations::table)
            .values(new_annotation)
            .get_result(conn)
    }

    /// Retrieves a stack annotation by its ID.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<StackAnnotation>` if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn get(
        &self,
        annotation_id: Uuid,
    ) -> Result<Option<StackAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_annotations::table
            .filter(stack_annotations::id.eq(annotation_id))
            .first(conn)
            .optional()
    }

    /// Lists all annotations for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack whose annotations to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `StackAnnotation`s associated with the specified stack.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<StackAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_annotations::table
            .filter(stack_annotations::stack_id.eq(stack_id))
            .load::<StackAnnotation>(conn)
    }

    /// Updates an existing stack annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to update.
    /// * `updated_annotation` - The updated annotation details.
    ///
    /// # Returns
    ///
    /// The updated `StackAnnotation`.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn update(
        &self,
        annotation_id: Uuid,
        updated_annotation: &StackAnnotation,
    ) -> Result<StackAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(stack_annotations::table.filter(stack_annotations::id.eq(annotation_id)))
            .set(updated_annotation)
            .get_result(conn)
    }

    /// Deletes a stack annotation from the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1).
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete(&self, annotation_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_annotations::table.filter(stack_annotations::id.eq(annotation_id)))
            .execute(conn)
    }

    /// Deletes all annotations for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack whose annotations to delete.
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
        diesel::delete(stack_annotations::table.filter(stack_annotations::stack_id.eq(stack_id)))
            .execute(conn)
    }
}
