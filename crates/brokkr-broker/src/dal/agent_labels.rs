/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for AgentLabel operations.
//!
//! This module provides functionality to interact with the agent_labels table in the database.
//! It includes methods for creating, retrieving, listing, and deleting agent labels, as well as
//! checking for label existence.

use crate::dal::DAL;
use brokkr_models::models::agent_labels::{AgentLabel, NewAgentLabel};
use brokkr_models::schema::agent_labels;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for AgentLabel operations.
pub struct AgentLabelsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> AgentLabelsDAL<'a> {
    /// Creates a new agent label in the database.
    ///
    /// # Arguments
    ///
    /// * `new_label` - A reference to the NewAgentLabel struct containing the label details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created AgentLabel on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_label: &NewAgentLabel) -> Result<AgentLabel, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_labels::table)
            .values(new_label)
            .get_result(conn)
    }

    /// Retrieves an agent label by its ID.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the agent label to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<AgentLabel> if found, or a diesel::result::Error on failure.
    pub fn get(&self, label_id: Uuid) -> Result<Option<AgentLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_labels::table
            .filter(agent_labels::id.eq(label_id))
            .first(conn)
            .optional()
    }

    /// Lists all labels for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose labels to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of AgentLabels on success, or a diesel::result::Error on failure.
    pub fn list_for_agent(&self, agent_id: Uuid) -> Result<Vec<AgentLabel>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_labels::table
            .filter(agent_labels::agent_id.eq(agent_id))
            .load::<AgentLabel>(conn)
    }

    /// Deletes an agent label from the database.
    ///
    /// # Arguments
    ///
    /// * `label_id` - The UUID of the agent label to delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn delete(&self, label_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_labels::table.filter(agent_labels::id.eq(label_id))).execute(conn)
    }

    /// Deletes all labels for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent whose labels to delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows on success, or a diesel::result::Error on failure.
    pub fn delete_all_for_agent(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_labels::table.filter(agent_labels::agent_id.eq(agent_id)))
            .execute(conn)
    }

    /// Checks if a label exists for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent.
    /// * `label` - The label to check for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a boolean (true if the label exists, false otherwise) on success,
    /// or a diesel::result::Error on failure.
    pub fn label_exists(&self, agent_id: Uuid, label: &str) -> Result<bool, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_labels::table
            .filter(agent_labels::agent_id.eq(agent_id))
            .filter(agent_labels::label.eq(label))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count > 0)
    }
}
