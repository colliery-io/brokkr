/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for AgentTarget operations.
//!
//! This module provides functionality to interact with the agent_targets table
//! in the database, allowing CRUD operations on AgentTarget entities.

use crate::dal::DAL;
use brokkr_models::models::agent_targets::{AgentTarget, NewAgentTarget};
use brokkr_models::schema::agent_targets;
use diesel::prelude::*;
use uuid::Uuid;

/// Handles database operations for AgentTarget entities.
pub struct AgentTargetsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl AgentTargetsDAL<'_> {
    /// Creates a new agent target in the database.
    ///
    /// # Arguments
    ///
    /// * `new_target` - The new agent target details to be inserted.
    ///
    /// # Returns
    ///
    /// The created `AgentTarget` on success, or a `diesel::result::Error` on failure.
    pub fn create(
        &self,
        new_target: &NewAgentTarget,
    ) -> Result<AgentTarget, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_targets::table)
            .values(new_target)
            .get_result(conn)
    }

    /// Retrieves an agent target by its ID.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The UUID of the agent target to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<AgentTarget>` if found, or a `diesel::result::Error` on failure.
    pub fn get(&self, target_id: Uuid) -> Result<Option<AgentTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_targets::table
            .filter(agent_targets::id.eq(target_id))
            .first(conn)
            .optional()
    }

    /// Lists all agent targets from the database.
    ///
    /// # Returns
    ///
    /// A `Vec<AgentTarget>` containing all agent targets, or a `diesel::result::Error` on failure.
    pub fn list(&self) -> Result<Vec<AgentTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_targets::table.load::<AgentTarget>(conn)
    }

    /// Lists all agent targets for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to list targets for.
    ///
    /// # Returns
    ///
    /// A `Vec<AgentTarget>` for the specified agent, or a `diesel::result::Error` on failure.
    pub fn list_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<AgentTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_targets::table
            .filter(agent_targets::agent_id.eq(agent_id))
            .load::<AgentTarget>(conn)
    }

    /// Lists all agent targets for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list targets for.
    ///
    /// # Returns
    ///
    /// A `Vec<AgentTarget>` for the specified stack, or a `diesel::result::Error` on failure.
    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<AgentTarget>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_targets::table
            .filter(agent_targets::stack_id.eq(stack_id))
            .load::<AgentTarget>(conn)
    }

    /// Deletes an agent target from the database.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The UUID of the agent target to delete.
    ///
    /// # Returns
    ///
    /// The number of affected rows (0 or 1) on success, or a `diesel::result::Error` on failure.
    pub fn delete(&self, target_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_targets::table.filter(agent_targets::id.eq(target_id))).execute(conn)
    }

    /// Deletes all agent targets for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to delete targets for.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_agent(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_targets::table.filter(agent_targets::agent_id.eq(agent_id)))
            .execute(conn)
    }

    /// Deletes all agent targets for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to delete targets for.
    ///
    /// # Returns
    ///
    /// The number of affected rows on success, or a `diesel::result::Error` on failure.
    pub fn delete_for_stack(&self, stack_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_targets::table.filter(agent_targets::stack_id.eq(stack_id)))
            .execute(conn)
    }
}
