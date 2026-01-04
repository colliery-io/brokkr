/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Agent Annotation operations.
//!
//! This module provides functionality to interact with agent annotations in the database,
//! including creating, retrieving, updating, and deleting annotations.

use crate::dal::DAL;
use brokkr_models::models::agent_annotations::{AgentAnnotation, NewAgentAnnotation};
use brokkr_models::schema::agent_annotations;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for Agent Annotations.
pub struct AgentAnnotationsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl AgentAnnotationsDAL<'_> {
    /// Creates a new agent annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - The new annotation details.
    ///
    /// # Returns
    ///
    /// The created `AgentAnnotation` or a database error.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn create(
        &self,
        new_annotation: &NewAgentAnnotation,
    ) -> Result<AgentAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_annotations::table)
            .values(new_annotation)
            .get_result(conn)
    }

    /// Retrieves an agent annotation by its ID.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<AgentAnnotation>` if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn get(
        &self,
        annotation_id: Uuid,
    ) -> Result<Option<AgentAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_annotations::table
            .filter(agent_annotations::id.eq(annotation_id))
            .first(conn)
            .optional()
    }

    /// Lists all annotations for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose annotations to retrieve.
    ///
    /// # Returns
    ///
    /// A `Vec<AgentAnnotation>` containing all annotations for the specified agent.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<AgentAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_annotations::table
            .filter(agent_annotations::agent_id.eq(agent_id))
            .load::<AgentAnnotation>(conn)
    }

    /// Lists all agent annotations in the database.
    ///
    /// # Returns
    ///
    /// A `Vec<AgentAnnotation>` containing all agent annotations.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn list(&self) -> Result<Vec<AgentAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_annotations::table.load::<AgentAnnotation>(conn)
    }

    /// Updates an existing agent annotation in the database.
    ///
    /// # Arguments
    ///
    /// * `annotation_id` - The UUID of the annotation to update.
    /// * `updated_annotation` - The updated annotation details.
    ///
    /// # Returns
    ///
    /// The updated `AgentAnnotation`.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn update(
        &self,
        annotation_id: Uuid,
        updated_annotation: &AgentAnnotation,
    ) -> Result<AgentAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agent_annotations::table.filter(agent_annotations::id.eq(annotation_id)))
            .set(updated_annotation)
            .get_result(conn)
    }

    /// Deletes an agent annotation from the database.
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
        diesel::delete(agent_annotations::table.filter(agent_annotations::id.eq(annotation_id)))
            .execute(conn)
    }

    /// Deletes all annotations for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose annotations to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete_all_for_agent(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_annotations::table.filter(agent_annotations::agent_id.eq(agent_id)))
            .execute(conn)
    }

    /// Deletes a specific annotation for an agent using a single indexed query.
    ///
    /// This is more efficient than fetching all annotations and filtering client-side.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent.
    /// * `key` - The annotation key to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1).
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the database operation fails.
    pub fn delete_by_agent_and_key(
        &self,
        agent_id: Uuid,
        key: &str,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            agent_annotations::table
                .filter(agent_annotations::agent_id.eq(agent_id))
                .filter(agent_annotations::key.eq(key)),
        )
        .execute(conn)
    }
}
