/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::dal::DAL;
use brokkr_models::models::agent_generator_registrations::{
    AgentGeneratorRegistration, NewAgentGeneratorRegistration,
};
use brokkr_models::schema::{agent_generator_registrations, agent_targets, stacks};
use diesel::dsl::exists;
use diesel::prelude::*;
use uuid::Uuid;

pub struct AgentGeneratorRegistrationsDAL<'a> {
    pub dal: &'a DAL,
}

impl AgentGeneratorRegistrationsDAL<'_> {
    /// Inserts a registration. Returns the created row.
    /// Callers should treat a UniqueViolation error as a 409 (already registered).
    pub fn create(
        &self,
        agent_id: Uuid,
        generator_id: Uuid,
    ) -> Result<AgentGeneratorRegistration, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let new = NewAgentGeneratorRegistration { agent_id, generator_id };
        diesel::insert_into(agent_generator_registrations::table)
            .values(&new)
            .get_result(conn)
    }

    /// O(1) existence check backed by the UNIQUE (agent_id, generator_id) index.
    pub fn is_registered(
        &self,
        agent_id: Uuid,
        generator_id: Uuid,
    ) -> Result<bool, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::select(exists(
            agent_generator_registrations::table
                .filter(agent_generator_registrations::agent_id.eq(agent_id))
                .filter(agent_generator_registrations::generator_id.eq(generator_id)),
        ))
        .get_result(conn)
    }

    /// All generators an agent is registered with.
    pub fn list_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<AgentGeneratorRegistration>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        agent_generator_registrations::table
            .filter(agent_generator_registrations::agent_id.eq(agent_id))
            .load::<AgentGeneratorRegistration>(conn)
    }

    /// All agents registered with a generator.
    pub fn list_for_generator(
        &self,
        generator_id: Uuid,
    ) -> Result<Vec<AgentGeneratorRegistration>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        agent_generator_registrations::table
            .filter(agent_generator_registrations::generator_id.eq(generator_id))
            .load::<AgentGeneratorRegistration>(conn)
    }

    /// Removes one registration. Returns rows deleted (0 or 1).
    pub fn delete(
        &self,
        agent_id: Uuid,
        generator_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(
            agent_generator_registrations::table
                .filter(agent_generator_registrations::agent_id.eq(agent_id))
                .filter(agent_generator_registrations::generator_id.eq(generator_id)),
        )
        .execute(conn)
    }

    /// Removes all agent_targets rows for the given agent where the target stack
    /// belongs to the given generator. Used as part of the deregistration cascade
    /// in DELETE /generators/:id/register.
    pub fn delete_agent_targets_for_generator(
        &self,
        agent_id: Uuid,
        generator_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let stack_ids: Vec<Uuid> = stacks::table
            .filter(stacks::generator_id.eq(generator_id))
            .filter(stacks::deleted_at.is_null())
            .select(stacks::id)
            .load(conn)?;

        diesel::delete(
            agent_targets::table
                .filter(agent_targets::agent_id.eq(agent_id))
                .filter(agent_targets::stack_id.eq_any(stack_ids)),
        )
        .execute(conn)
    }
}
