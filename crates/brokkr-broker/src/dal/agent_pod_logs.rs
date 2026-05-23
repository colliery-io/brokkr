/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! DAL for the short-lived `agent_pod_logs` telemetry table.

use brokkr_models::models::agent_pod_logs::{AgentPodLog, NewAgentPodLog};
use brokkr_models::schema::agent_pod_logs;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::DAL;

pub struct AgentPodLogsDAL<'a> {
    pub dal: &'a DAL,
}

impl AgentPodLogsDAL<'_> {
    pub fn create(&self, new_line: &NewAgentPodLog) -> Result<AgentPodLog, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(agent_pod_logs::table)
            .values(new_line)
            .get_result(conn)
    }

    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AgentPodLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_pod_logs::table
            .filter(agent_pod_logs::stack_id.eq(stack_id))
            .filter(agent_pod_logs::created_at.ge(since))
            .order(agent_pod_logs::created_at.desc())
            .limit(limit)
            .load::<AgentPodLog>(conn)
    }

    pub fn evict_older_than(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(agent_pod_logs::table.filter(agent_pod_logs::created_at.lt(cutoff)))
            .execute(conn)
    }

    pub fn count(&self) -> Result<i64, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        agent_pod_logs::table.count().get_result(conn)
    }
}
