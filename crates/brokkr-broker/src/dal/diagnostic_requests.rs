/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for DiagnosticRequest operations.
//!
//! This module provides functionality to interact with the diagnostic_requests table.
//! It includes methods for creating, claiming, completing, and querying diagnostic requests.

use crate::dal::DAL;
use brokkr_models::models::diagnostic_requests::{
    DiagnosticRequest, NewDiagnosticRequest, UpdateDiagnosticRequest,
};
use brokkr_models::schema::diagnostic_requests;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for DiagnosticRequest operations.
pub struct DiagnosticRequestsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl DiagnosticRequestsDAL<'_> {
    /// Creates a new diagnostic request.
    ///
    /// # Arguments
    ///
    /// * `new_request` - The diagnostic request to create.
    ///
    /// # Returns
    ///
    /// Returns the created DiagnosticRequest record.
    pub fn create(
        &self,
        new_request: &NewDiagnosticRequest,
    ) -> Result<DiagnosticRequest, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(diagnostic_requests::table)
            .values(new_request)
            .get_result(conn)
    }

    /// Gets a diagnostic request by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the diagnostic request if found.
    pub fn get(&self, id: Uuid) -> Result<Option<DiagnosticRequest>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diagnostic_requests::table
            .filter(diagnostic_requests::id.eq(id))
            .first(conn)
            .optional()
    }

    /// Gets all pending diagnostic requests for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent UUID.
    ///
    /// # Returns
    ///
    /// Returns a list of pending diagnostic requests for the agent.
    pub fn get_pending_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<DiagnosticRequest>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diagnostic_requests::table
            .filter(diagnostic_requests::agent_id.eq(agent_id))
            .filter(diagnostic_requests::status.eq("pending"))
            .filter(diagnostic_requests::expires_at.gt(Utc::now()))
            .order(diagnostic_requests::created_at.asc())
            .load(conn)
    }

    /// Claims a diagnostic request for processing.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated diagnostic request if successful.
    pub fn claim(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let update = UpdateDiagnosticRequest {
            status: Some("claimed".to_string()),
            claimed_at: Some(Utc::now()),
            completed_at: None,
        };

        diesel::update(diagnostic_requests::table.filter(diagnostic_requests::id.eq(id)))
            .set(&update)
            .get_result(conn)
    }

    /// Marks a diagnostic request as completed.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated diagnostic request if successful.
    pub fn complete(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let update = UpdateDiagnosticRequest {
            status: Some("completed".to_string()),
            claimed_at: None,
            completed_at: Some(Utc::now()),
        };

        diesel::update(diagnostic_requests::table.filter(diagnostic_requests::id.eq(id)))
            .set(&update)
            .get_result(conn)
    }

    /// Marks a diagnostic request as failed.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated diagnostic request if successful.
    pub fn fail(&self, id: Uuid) -> Result<DiagnosticRequest, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let update = UpdateDiagnosticRequest {
            status: Some("failed".to_string()),
            claimed_at: None,
            completed_at: Some(Utc::now()),
        };

        diesel::update(diagnostic_requests::table.filter(diagnostic_requests::id.eq(id)))
            .set(&update)
            .get_result(conn)
    }

    /// Lists all diagnostic requests for a specific deployment object.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_id` - The deployment object UUID.
    ///
    /// # Returns
    ///
    /// Returns a list of diagnostic requests.
    pub fn list_by_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<Vec<DiagnosticRequest>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diagnostic_requests::table
            .filter(diagnostic_requests::deployment_object_id.eq(deployment_object_id))
            .order(diagnostic_requests::created_at.desc())
            .load(conn)
    }

    /// Expires all pending requests that have passed their expiry time.
    ///
    /// # Returns
    ///
    /// Returns the number of requests expired.
    pub fn expire_old_requests(&self) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(
            diagnostic_requests::table
                .filter(diagnostic_requests::status.eq("pending"))
                .filter(diagnostic_requests::expires_at.lt(Utc::now())),
        )
        .set(diagnostic_requests::status.eq("expired"))
        .execute(conn)
    }

    /// Deletes expired and completed requests older than the given age in hours.
    ///
    /// # Arguments
    ///
    /// * `max_age_hours` - Maximum age in hours for completed/expired requests.
    ///
    /// # Returns
    ///
    /// Returns the number of requests deleted.
    pub fn cleanup_old_requests(&self, max_age_hours: i64) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);

        diesel::delete(
            diagnostic_requests::table
                .filter(
                    diagnostic_requests::status
                        .eq("expired")
                        .or(diagnostic_requests::status.eq("completed"))
                        .or(diagnostic_requests::status.eq("failed")),
                )
                .filter(diagnostic_requests::created_at.lt(cutoff)),
        )
        .execute(conn)
    }

    /// Deletes a diagnostic request by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The diagnostic request UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of records deleted.
    pub fn delete(&self, id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::delete(diagnostic_requests::table.filter(diagnostic_requests::id.eq(id)))
            .execute(conn)
    }
}
