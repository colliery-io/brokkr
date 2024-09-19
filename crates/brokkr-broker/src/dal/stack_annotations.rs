use crate::dal::DAL;
use brokkr_models::models::stack_annotations::{NewStackAnnotation, StackAnnotation};
use brokkr_models::schema::stack_annotations;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for Stack Annotation operations.
pub struct StackAnnotationsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> StackAnnotationsDAL<'a> {
    /// Creates a new stack annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - A reference to the NewStackAnnotation struct containing the annotation details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created StackAnnotation on success, or a diesel::result::Error on failure.
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
    /// Returns a Result containing an Option<StackAnnotation> if found, or a diesel::result::Error on failure.
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
    /// Returns a Result containing a Vec of StackAnnotations for the specified stack, or a diesel::result::Error on failure.
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
    /// * `updated_annotation` - A reference to the StackAnnotation struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated StackAnnotation on success, or a diesel::result::Error on failure.
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
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
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
    /// Returns a Result containing the number of affected rows on success, or a diesel::result::Error on failure.
    pub fn delete_all_for_stack(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_annotations::table.filter(stack_annotations::stack_id.eq(stack_id)))
            .execute(conn)
    }
}
