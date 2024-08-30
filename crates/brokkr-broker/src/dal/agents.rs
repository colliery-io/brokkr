//! This module provides a Data Access Layer (DAL) for managing Agent entities in the database.
//!
//! It uses Diesel ORM for database operations and includes functionality for creating,
//! retrieving, updating, and soft-deleting agents.

use crate::dal::DAL;
use brokkr_models::models::{
    agents::{Agent, NewAgent},
    stacks::Stack,
    deployment_objects::DeploymentObject,
    };
use brokkr_models::schema::*;

use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, Nullable, Jsonb, Uuid, Array};

use uuid::Uuid as RustUuid;

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
    pub fn get(&self, uuid: RustUuid, include_deleted: bool) -> Result<Agent, diesel::result::Error> {
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
    pub fn soft_delete(&self, uuid: RustUuid) -> Result<(), diesel::result::Error> {
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
    pub fn update(&self, uuid: RustUuid, agent: &Agent) -> Result<Agent, diesel::result::Error> {
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
    pub fn update_heartbeat(&self, uuid: RustUuid) -> Result<Agent, diesel::result::Error> {
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
    pub fn update_status(&self, uuid: RustUuid, status: &str) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::id.eq(uuid)))
            .set(agents::status.eq(status))
            .get_result(conn)
    }


    /// Finds matching stacks and undeployed objects for a given agent.
    ///
    /// # Arguments
    ///
    /// * `agent_uuid` - The UUID of the agent to find matches for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of tuples (Stack, Vec<DeploymentObject>) on success,
    /// or a diesel::result::Error on failure.
    // pub fn get_undeployed_objects(&self, agent_uuid: RustUuid) {//-> Result<Vec<(Stack, Vec<DeploymentObject>)>, diesel::result::Error> {
    //     use brokkr_models::schema::agents::dsl::*;
    //     use brokkr_models::schema::agents::dsl::id as agent_id;
    //     use brokkr_models::schema::stacks::dsl::*;
    //     use brokkr_models::schema::stacks::dsl::labels as stacks_labels;
    //     let conn = &mut self.dal.pool.get().unwrap();


    //     let agent = agents
    //     .filter(agent_id.eq(agent_uuid))
    //     .first::<Agent>(conn)?;

        

        // let stacks = stacks.filter(
        //     stacks_labels.is_null()
        //         .and(agent.labels.is_not_nill())
        //         .and(array_contains_any(labels.cast(), agent.labels.cast()))
        // )
        
        // .filter(
        //     Stack::labels.is_not_null()
        //         .and(agent.labels.is_not_null())
        //         .and(array_contains_all(Stack::labels.cast(), agent.labels.cast()))
        //     .or(Stack::agent_target.cast::<Jsonb>().contains(
        //         serde_json::json!([{"agent": agent.name, "cluster": agent.cluster_name}])
        //     ))
        //     .or(
        //         Stack::annotations.is_not_null()
        //             .and(agent.annotations.is_not_null())
        //             .and(Stack::annotations.cast::<Jsonb>().contains(agent.annotations.cast()))
        //     )
        // )
        // .select(Stack::id);

    // }


}
