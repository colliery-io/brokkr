/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for AgentEvent operations.
//!
//! This module provides functionality to interact with the agent_events table in the database.
//! It includes methods for creating, retrieving, updating, and deleting agent events, as well as
//! listing events with various filtering options.

use crate::dal::DAL;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::schema::agent_events;
use brokkr_models::schema::deployment_objects;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for AgentEvent operations.
pub struct AgentEventsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl AgentEventsDAL<'_> {
    /// Creates a new agent event in the database.
    ///
    /// # Arguments
    ///
    /// * `new_event` - A reference to the NewAgentEvent struct containing the event details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created AgentEvent on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_event: &NewAgentEvent) -> Result<AgentEvent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_events::table)
            .values(new_event)
            .get_result(conn)
    }

    /// Retrieves a non-deleted agent event by its UUID.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<AgentEvent> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(&self, event_uuid: Uuid) -> Result<Option<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_events::table
            .filter(agent_events::id.eq(event_uuid))
            .filter(agent_events::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves an agent event by its UUID, including deleted events.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<AgentEvent> if found (including deleted events), or a diesel::result::Error on failure.
    pub fn get_including_deleted(
        &self,
        event_uuid: Uuid,
    ) -> Result<Option<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_events::table
            .filter(agent_events::id.eq(event_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted agent events from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted AgentEvents on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_events::table
            .filter(agent_events::deleted_at.is_null())
            .load::<AgentEvent>(conn)
    }

    /// Lists all agent events from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all AgentEvents (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_events::table.load::<AgentEvent>(conn)
    }

    /// Lists agent events from the database with optional filtering by stack and agent.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - Optional UUID to filter events by stack.
    /// * `agent_id` - Optional UUID to filter events by agent.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of filtered AgentEvents on success, or a diesel::result::Error on failure.
    pub fn get_events(
        &self,
        stack_id: Option<Uuid>,
        agent_id: Option<Uuid>,
    ) -> Result<Vec<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = agent_events::table
            .inner_join(deployment_objects::table)
            .into_boxed();

        if let Some(s_id) = stack_id {
            query = query.filter(deployment_objects::stack_id.eq(s_id));
        }

        if let Some(a_id) = agent_id {
            query = query.filter(agent_events::agent_id.eq(a_id));
        }

        query = query
            .filter(agent_events::deleted_at.is_null())
            .order(agent_events::created_at.desc());

        query
            .select(agent_events::all_columns)
            .load::<AgentEvent>(conn)
    }

    /// Updates an existing agent event in the database.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to update.
    /// * `updated_event` - A reference to the AgentEvent struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated AgentEvent on success, or a diesel::result::Error on failure.
    pub fn update(
        &self,
        event_uuid: Uuid,
        updated_event: &AgentEvent,
    ) -> Result<AgentEvent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agent_events::table.filter(agent_events::id.eq(event_uuid)))
            .set(updated_event)
            .get_result(conn)
    }

    /// Soft deletes an agent event by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, event_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agent_events::table.filter(agent_events::id.eq(event_uuid)))
            .set(agent_events::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Hard deletes an agent event from the database.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to hard delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn hard_delete(&self, event_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_events::table.filter(agent_events::id.eq(event_uuid))).execute(conn)
    }
}
