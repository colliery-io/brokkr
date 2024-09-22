use crate::dal::DAL;
use brokkr_models::models::generator::{Generator, NewGenerator};
use brokkr_models::schema::generators;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for Generator operations.
///
/// This module provides a set of methods to interact with the generators table in the database.
/// It includes operations for creating, retrieving, updating, and deleting generators,
/// as well as specialized queries for filtering and updating specific fields.
pub struct GeneratorsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> GeneratorsDAL<'a> {
    /// Creates a new generator in the database.
    ///
    /// # Arguments
    ///
    /// * `new_generator` - A reference to the NewGenerator struct containing the generator details.
    ///
    /// # Returns
    ///
    /// A Result containing the created Generator on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_generator: &NewGenerator) -> Result<Generator, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(generators::table)
            .values(new_generator)
            .get_result(conn)
    }

    /// Retrieves a non-deleted generator by its UUID.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<Generator> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(&self, generator_uuid: Uuid) -> Result<Option<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table
            .filter(generators::id.eq(generator_uuid))
            .filter(generators::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves a generator by its UUID, including deleted generators.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<Generator> if found (including deleted generators), or a diesel::result::Error on failure.
    pub fn get_including_deleted(
        &self,
        generator_uuid: Uuid,
    ) -> Result<Option<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table
            .filter(generators::id.eq(generator_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted generators from the database.
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of all non-deleted Generators on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table
            .filter(generators::deleted_at.is_null())
            .load::<Generator>(conn)
    }

    /// Lists all generators from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of all Generators (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table.load::<Generator>(conn)
    }

    /// Updates an existing generator in the database.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to update.
    /// * `updated_generator` - A reference to the Generator struct containing the updated details.
    ///
    /// # Returns
    ///
    /// A Result containing the updated Generator on success, or a diesel::result::Error on failure.
    pub fn update(
        &self,
        generator_uuid: Uuid,
        updated_generator: &Generator,
    ) -> Result<Generator, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators::table.filter(generators::id.eq(generator_uuid)))
            .set(updated_generator)
            .get_result(conn)
    }

    /// Soft deletes a generator by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - The UUID of the generator to soft delete.
    ///
    /// # Returns
    ///
    /// A Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, generator_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators::table.filter(generators::id.eq(generator_id)))
            .set(generators::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Hard deletes a generator from the database.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to hard delete.
    ///
    /// # Returns
    ///
    /// A Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn hard_delete(&self, generator_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(generators::table.filter(generators::id.eq(generator_uuid))).execute(conn)
    }

    /// Updates the pak_hash for a generator.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to update.
    /// * `new_pak_hash` - The new pak_hash value.
    ///
    /// # Returns
    ///
    /// A Result containing the updated Generator on success, or a diesel::result::Error on failure.
    pub fn update_pak_hash(
        &self,
        generator_uuid: Uuid,
        new_pak_hash: String,
    ) -> Result<Generator, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators::table.filter(generators::id.eq(generator_uuid)))
            .set(generators::pak_hash.eq(new_pak_hash))
            .get_result(conn)
    }

    /// Updates the last_active_at timestamp for a generator and sets is_active to true.
    ///
    /// # Arguments
    ///
    /// * `generator_uuid` - The UUID of the generator to update.
    ///
    /// # Returns
    ///
    /// A Result containing the updated Generator on success, or a diesel::result::Error on failure.
    pub fn update_last_active(
        &self,
        generator_uuid: Uuid,
    ) -> Result<Generator, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators::table.filter(generators::id.eq(generator_uuid)))
            .set((
                generators::last_active_at.eq(Utc::now()),
                generators::is_active.eq(true),
            ))
            .get_result(conn)
    }

    /// Retrieves a non-deleted generator by its name.
    ///
    /// # Arguments
    ///
    /// * `generator_name` - The name of the generator to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<Generator> if found, or a diesel::result::Error on failure.
    pub fn get_by_name(
        &self,
        generator_name: &str,
    ) -> Result<Option<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table
            .filter(generators::name.eq(generator_name))
            .filter(generators::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves non-deleted generators by their active status.
    ///
    /// # Arguments
    ///
    /// * `active` - The active status to filter by.
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of matching Generators on success, or a diesel::result::Error on failure.
    pub fn get_by_active_status(
        &self,
        active: bool,
    ) -> Result<Vec<Generator>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators::table
            .filter(generators::is_active.eq(active))
            .filter(generators::deleted_at.is_null())
            .load::<Generator>(conn)
    }
}
