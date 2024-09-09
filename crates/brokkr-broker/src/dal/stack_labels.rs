use crate::dal::DAL;
use brokkr_models::models::stack_labels::{StackLabel, NewStackLabel};
use brokkr_models::schema::stack_labels;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for StackLabel operations.
pub struct StackLabelsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> StackLabelsDAL<'a> {
    /// Creates a new stack label in the database.
    ///
    /// # Arguments
    ///
    /// * `new_label` - A reference to the NewStackLabel struct containing the label details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created StackLabel on success, or a diesel::result::Error on failure.
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
    /// * `label_id` - The UUID of the stack label to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<StackLabel> if found, or a diesel::result::Error on failure.
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
    /// * `stack_id` - The UUID of the stack to get labels for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of StackLabels for the specified stack, or a diesel::result::Error on failure.
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
    /// * `label_id` - The UUID of the stack label to delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn delete(&self, label_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_labels::table.filter(stack_labels::id.eq(label_id)))
            .execute(conn)
    }

    /// Deletes all labels for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to delete labels for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows on success, or a diesel::result::Error on failure.
    pub fn delete_all_for_stack(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_labels::table.filter(stack_labels::stack_id.eq(stack_id)))
            .execute(conn)
    }

    /// Searches for stack labels by label text.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to match against label text.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching StackLabels on success, or a diesel::result::Error on failure.
    pub fn search(&self, query: &str) -> Result<Vec<StackLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_labels::table
            .filter(stack_labels::label.ilike(format!("%{}%", query)))
            .load::<StackLabel>(conn)
    }
}