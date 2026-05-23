/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! DAL for the short-lived `agent_k8s_events` telemetry table.

use brokkr_models::models::agent_k8s_events::{AgentK8sEvent, NewAgentK8sEvent};
use brokkr_models::schema::agent_k8s_events;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::DAL;

pub struct AgentK8sEventsDAL<'a> {
    pub dal: &'a DAL,
}

impl AgentK8sEventsDAL<'_> {
    pub fn create(
        &self,
        new_event: &NewAgentK8sEvent,
    ) -> Result<AgentK8sEvent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_k8s_events::table)
            .values(new_event)
            .get_result(conn)
    }

    /// Paginated list of events for a stack within the retained window,
    /// newest-first. Used by the REST history endpoint (WS-10).
    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AgentK8sEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_k8s_events::table
            .filter(agent_k8s_events::stack_id.eq(stack_id))
            .filter(agent_k8s_events::created_at.ge(since))
            .order(agent_k8s_events::created_at.desc())
            .limit(limit)
            .load::<AgentK8sEvent>(conn)
    }

    /// Delete rows older than `cutoff`. Returns row count.
    pub fn evict_older_than(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_k8s_events::table.filter(agent_k8s_events::created_at.lt(cutoff)))
            .execute(conn)
    }

    /// Total row count (diagnostics / metrics).
    pub fn count(&self) -> Result<i64, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_k8s_events::table.count().get_result(conn)
    }
}
