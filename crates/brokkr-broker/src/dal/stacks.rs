use crate::dal::DAL;
use brokkr_models::models::stacks::{Stack, NewStack};
use brokkr_models::schema::stacks;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for Stack operations.
pub struct StacksDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
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
    /// Returns a Result containing the created Stack on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_stack: &NewStack) -> Result<Stack, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(stacks::table)
            .values(new_stack)
            .get_result(conn)
    }

    /// Retrieves a non-deleted stack by its UUID.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Stack> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(&self, stack_uuid: Uuid) -> Result<Option<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::id.eq(stack_uuid))
            .filter(stacks::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves a stack by its UUID, including deleted stacks.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Stack> if found (including deleted stacks), or a diesel::result::Error on failure.
    pub fn get_including_deleted(&self, stack_uuid: Uuid) -> Result<Option<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::id.eq(stack_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted stacks from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted Stacks on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)
    }

    /// Lists all stacks from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all Stacks (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table.load::<Stack>(conn)
    }

    /// Updates an existing stack in the database.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to update.
    /// * `updated_stack` - A reference to the Stack struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Stack on success, or a diesel::result::Error on failure.
    pub fn update(&self, stack_uuid: Uuid, updated_stack: &Stack) -> Result<Stack, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .set(updated_stack)
            .get_result(conn)
    }

    /// Soft deletes a stack by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .set(stacks::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Searches for non-deleted stacks by name.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to match against stack names.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching non-deleted Stacks on success, or a diesel::result::Error on failure.
    pub fn search(&self, query: &str) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::name.ilike(format!("%{}%", query)))
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)
    }

    /// Searches for all stacks by name, including deleted ones.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to match against stack names.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all matching Stacks (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn search_all(&self, query: &str) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::name.ilike(format!("%{}%", query)))
            .load::<Stack>(conn)
    }

    /// Hard deletes a stack from the database.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to hard delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn hard_delete(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .execute(conn)
    }
}