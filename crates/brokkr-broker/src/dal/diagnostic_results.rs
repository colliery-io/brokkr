/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for DiagnosticResult operations.
//!
//! This module provides functionality to interact with the diagnostic_results table.
//! It includes methods for creating and querying diagnostic results.

use crate::dal::DAL;
use brokkr_models::models::diagnostic_results::{DiagnosticResult, NewDiagnosticResult};
use brokkr_models::schema::diagnostic_results;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for DiagnosticResult operations.
pub struct DiagnosticResultsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl DiagnosticResultsDAL<'_> {
    /// Creates a new diagnostic result.
    ///
    /// # Arguments
    ///
    /// * `new_result` - The diagnostic result to create.
    ///
    /// # Returns
    ///
    /// Returns the created DiagnosticResult record.
    pub fn create(
        &self,
        new_result: &NewDiagnosticResult,
    ) -> Result<DiagnosticResult, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(diagnostic_results::table)
            .values(new_result)
            .get_result(conn)
    }

    /// Gets a diagnostic result by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic result UUID.
    ///
    /// # Returns
    ///
    /// Returns the diagnostic result if found.
    pub fn get(&self, id: Uuid) -> Result<Option<DiagnosticResult>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diagnostic_results::table
            .filter(diagnostic_results::id.eq(id))
            .first(conn)
            .optional()
    }

    /// Gets the diagnostic result for a specific request.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the diagnostic result if found.
    pub fn get_by_request(
        &self,
        request_id: Uuid,
    ) -> Result<Option<DiagnosticResult>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diagnostic_results::table
            .filter(diagnostic_results::request_id.eq(request_id))
            .first(conn)
            .optional()
    }

    /// Deletes a diagnostic result by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic result UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of records deleted.
    pub fn delete(&self, id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::delete(diagnostic_results::table.filter(diagnostic_results::id.eq(id)))
            .execute(conn)
    }

    /// Deletes all diagnostic results for a specific request.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of records deleted.
    pub fn delete_by_request(&self, request_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::delete(
            diagnostic_results::table.filter(diagnostic_results::request_id.eq(request_id)),
        )
        .execute(conn)
    }
}
