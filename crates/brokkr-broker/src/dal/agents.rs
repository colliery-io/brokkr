//! This module provides a Data Access Layer (DAL) for managing Agent entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, updating, and soft-deleting agents.

use crate::dal::DAL;
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::schema::agents;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Represents the Data Access Layer for Agent-related operations.
pub struct AgentsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> AgentsDAL<'a> {
    /// Creates a new agent in the database.
    ///
    /// # Arguments
    ///
    /// * `new_agent` - A reference to the NewAgent struct containing the agent details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created Agent on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_agent: &NewAgent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::insert_into(agents::table)
            .values(new_agent)
            .get_result(conn)
    }

    /// Retrieves an agent by its UUID, excluding soft-deleted agents.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the agent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Agent if found, or a diesel::result::Error if not found or on failure.
    #[allow(unused_variables)]
    pub fn get(&self, uuid: Uuid, include_deleted: bool) -> Result<Agent, diesel::result::Error> {
        use brokkr_models::schema::agents::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();

        let mut query = agents.filter(id.eq(uuid)).into_boxed();

        if !include_deleted {
            query = query.filter(deleted_at.is_null());
        }

        query.first(conn)
    }

    /// Soft deletes an agent by setting its deleted_at timestamp.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the agent to soft delete.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or a diesel::result::Error on failure.
    #[allow(unused_variables)]
    pub fn soft_delete(&self, uuid: Uuid) -> Result<(), diesel::result::Error> {
        use brokkr_models::schema::agents::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        let now = Utc::now().naive_utc();
        let result = diesel::update(agents.filter(id.eq(uuid)))
            .set(deleted_at.eq(now))
            .execute(conn);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Lists all agents, optionally including soft-deleted ones.
    ///
    /// # Arguments
    ///
    /// * `include_deleted` - A boolean flag to determine whether to include soft-deleted agents.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of Agents on success, or a diesel::result::Error on failure.
    pub fn list(&self, include_deleted: bool) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        let mut query = agents::table.into_boxed();

        if !include_deleted {
            query = query.filter(agents::deleted_at.is_null());
        }

        query.select(agents::all_columns).load::<Agent>(conn)
    }

    /// Updates an existing agent.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the agent to update.
    /// * `agent` - A reference to the Agent struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Agent on success, or a diesel::result::Error on failure.
    pub fn update(&self, uuid: Uuid, agent: &Agent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::id.eq(uuid)))
            .set(agent)
            .get_result(conn)
    }

    /// Updates an agent's last heartbeat timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the agent to update.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Agent on success, or a diesel::result::Error on failure.
    pub fn update_heartbeat(&self, uuid: Uuid) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::id.eq(uuid)))
            .set(agents::last_heartbeat.eq(diesel::dsl::now))
            .get_result(conn)
    }

    /// Updates an agent's status.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the agent to update.
    /// * `status` - A string slice representing the new status.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Agent on success, or a diesel::result::Error on failure.
    pub fn update_status(&self, uuid: Uuid, status: &str) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::id.eq(uuid)))
            .set(agents::status.eq(status))
            .get_result(conn)
    }
}
