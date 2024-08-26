//! This module provides a Data Access Layer (DAL) for managing AgentEvent entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, listing, and soft-deleting agent events.

use crate::dal::DAL;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::schema::agent_events;
use brokkr_models::schema::deployment_objects;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Represents the Data Access Layer for AgentEvent-related operations.
pub struct AgentEventsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> AgentEventsDAL<'a> {
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

    /// Lists all agent events from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all AgentEvents on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<AgentEvent>, diesel::result::Error> {
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

    /// Soft deletes an agent event by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `event_uuid` - The UUID of the agent event to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing () on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, event_uuid: Uuid) -> Result<(), diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(agent_events::table.filter(agent_events::id.eq(event_uuid)))
            .set(agent_events::deleted_at.eq(Utc::now().naive_utc()))
            .execute(conn)
            .map(|_| ())
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
}
