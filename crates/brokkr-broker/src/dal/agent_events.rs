//! This module provides a Data Access Layer (DAL) for managing AgentEvent entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, listing, and soft-deleting agent events.

use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::schema::agent_events;
use chrono::Utc;
use crate::dal::DAL;

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
        let conn = &mut self.dal.pool.get().unwrap();
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
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events::table.load::<AgentEvent>(conn)
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
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events
            .filter(uuid.eq(event_uuid))
            .filter(deleted_at.is_null())
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
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agent_events.filter(uuid.eq(event_uuid)))
            .set(deleted_at.eq(Utc::now().naive_utc()))
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
    pub fn get_including_deleted(&self, event_uuid: Uuid) -> Result<Option<AgentEvent>, diesel::result::Error> {
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events
            .filter(uuid.eq(event_uuid))
            .first(conn)
            .optional()
    }
}