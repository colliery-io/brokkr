/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for DeploymentHealth operations.
//!
//! This module provides functionality to interact with the deployment_health table in the database.
//! It includes methods for upserting health status, querying health by agent/deployment/stack,
//! and aggregating health across deployments.

use crate::dal::DAL;
use brokkr_models::models::deployment_health::{DeploymentHealth, NewDeploymentHealth};
use brokkr_models::schema::deployment_health;
use brokkr_models::schema::deployment_objects;
use diesel::prelude::*;
use diesel::upsert::excluded;
use std::collections::HashMap;
use uuid::Uuid;

/// Data Access Layer for DeploymentHealth operations.
pub struct DeploymentHealthDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl DeploymentHealthDAL<'_> {
    /// Upserts a deployment health record.
    ///
    /// If a record already exists for the agent+deployment_object combination,
    /// it will be updated. Otherwise, a new record will be created.
    ///
    /// # Arguments
    ///
    /// * `new_health` - The health record to upsert.
    ///
    /// # Returns
    ///
    /// Returns the upserted DeploymentHealth record.
    pub fn upsert(
        &self,
        new_health: &NewDeploymentHealth,
    ) -> Result<DeploymentHealth, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        diesel::insert_into(deployment_health::table)
            .values(new_health)
            .on_conflict((
                deployment_health::agent_id,
                deployment_health::deployment_object_id,
            ))
            .do_update()
            .set((
                deployment_health::status.eq(excluded(deployment_health::status)),
                deployment_health::summary.eq(excluded(deployment_health::summary)),
                deployment_health::checked_at.eq(excluded(deployment_health::checked_at)),
            ))
            .get_result(conn)
    }

    /// Upserts multiple deployment health records in a batch.
    ///
    /// # Arguments
    ///
    /// * `health_records` - A slice of health records to upsert.
    ///
    /// # Returns
    ///
    /// Returns the number of records affected.
    pub fn upsert_batch(
        &self,
        health_records: &[NewDeploymentHealth],
    ) -> Result<usize, diesel::result::Error> {
        if health_records.is_empty() {
            return Ok(0);
        }

        let conn = &mut self.dal.conn()?;

        diesel::insert_into(deployment_health::table)
            .values(health_records)
            .on_conflict((
                deployment_health::agent_id,
                deployment_health::deployment_object_id,
            ))
            .do_update()
            .set((
                deployment_health::status.eq(excluded(deployment_health::status)),
                deployment_health::summary.eq(excluded(deployment_health::summary)),
                deployment_health::checked_at.eq(excluded(deployment_health::checked_at)),
            ))
            .execute(conn)
    }

    /// Keeps the records an agent may write: the deployment object is not
    /// deleted, and the agent serves its stack (`StacksDAL::associations`).
    ///
    /// An agent can send a report after its stack is deleted or after it lost
    /// the stack. Such a report is dropped, not refused: the rest of the batch
    /// is good, and the agent stops by itself on its next read of what it
    /// serves (BROKKR-T-0394).
    pub fn retain_served(
        &self,
        records: Vec<NewDeploymentHealth>,
    ) -> Result<Vec<NewDeploymentHealth>, diesel::result::Error> {
        if records.is_empty() {
            return Ok(records);
        }
        let object_ids: Vec<Uuid> = records.iter().map(|r| r.deployment_object_id).collect();
        let stack_of: HashMap<Uuid, Uuid> = {
            let conn = &mut self.dal.conn()?;
            deployment_objects::table
                .filter(deployment_objects::id.eq_any(&object_ids))
                .filter(deployment_objects::deleted_at.is_null())
                .select((deployment_objects::id, deployment_objects::stack_id))
                .load::<(Uuid, Uuid)>(conn)?
                .into_iter()
                .collect()
        };
        let mut served: HashMap<Uuid, std::collections::HashSet<(Uuid, Uuid)>> = HashMap::new();
        let mut kept = Vec::with_capacity(records.len());
        for record in records {
            let Some(stack_id) = stack_of.get(&record.deployment_object_id) else {
                continue;
            };
            let pairs = match served.entry(record.agent_id) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => entry.insert(
                    self.dal
                        .stacks()
                        .associations(Some(record.agent_id), None)?,
                ),
            };
            if pairs.contains(&(record.agent_id, *stack_id)) {
                kept.push(record);
            }
        }
        Ok(kept)
    }

    /// Gets the health record for a specific agent and deployment object.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent UUID.
    /// * `deployment_object_id` - The deployment object UUID.
    ///
    /// # Returns
    ///
    /// Returns the health record if found.
    pub fn get_by_agent_and_deployment(
        &self,
        agent_id: Uuid,
        deployment_object_id: Uuid,
    ) -> Result<Option<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        deployment_health::table
            .filter(deployment_health::agent_id.eq(agent_id))
            .filter(deployment_health::deployment_object_id.eq(deployment_object_id))
            .first(conn)
            .optional()
    }

    /// Gets the health record by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The health record UUID.
    ///
    /// # Returns
    ///
    /// Returns the health record if found.
    pub fn get(&self, id: Uuid) -> Result<Option<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        deployment_health::table
            .filter(deployment_health::id.eq(id))
            .first(conn)
            .optional()
    }

    /// Lists all health records for a specific deployment object (across all agents).
    ///
    /// # Arguments
    ///
    /// * `deployment_object_id` - The deployment object UUID.
    ///
    /// # Returns
    ///
    /// Returns a list of health records.
    pub fn list_by_deployment_object(
        &self,
        deployment_object_id: Uuid,
    ) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        deployment_health::table
            .filter(deployment_health::deployment_object_id.eq(deployment_object_id))
            .order(deployment_health::checked_at.desc())
            .load(conn)
    }

    /// Lists all health records for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent UUID.
    ///
    /// # Returns
    ///
    /// Returns a list of health records.
    pub fn list_by_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        deployment_health::table
            .filter(deployment_health::agent_id.eq(agent_id))
            .order(deployment_health::checked_at.desc())
            .load(conn)
    }

    /// Lists the current health records of a stack: the records of the
    /// agents that serve the stack now, for its latest deployment object.
    ///
    /// A record counts only while its agent serves its object
    /// (`DeploymentObjectsDAL::served_pairs`). The record of a replaced
    /// object, of a deleted stack, or of an agent that no longer serves the
    /// stack stays in the table but is not current (BROKKR-T-0393).
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The stack UUID.
    ///
    /// # Returns
    ///
    /// Returns the current health records of the stack, newest check first.
    pub fn list_by_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let served = self.dal.deployment_objects().served_pairs()?;
        let conn = &mut self.dal.conn()?;

        let mut records: Vec<DeploymentHealth> = deployment_health::table
            .inner_join(deployment_objects::table)
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .select(deployment_health::all_columns)
            .order(deployment_health::checked_at.desc())
            .load(conn)?;
        records.retain(|r| served.contains(&(r.agent_id, r.deployment_object_id)));
        Ok(records)
    }

    /// Lists all health records with a specific status.
    ///
    /// # Arguments
    ///
    /// * `status` - The health status to filter by (e.g., "degraded", "failing").
    ///
    /// # Returns
    ///
    /// Returns a list of health records matching the status.
    pub fn list_by_status(
        &self,
        status: &str,
    ) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        deployment_health::table
            .filter(deployment_health::status.eq(status))
            .order(deployment_health::checked_at.desc())
            .load(conn)
    }

    /// Returns the counts of current deployment-health records grouped by
    /// `(agent_id, status)`, with no per-agent fan-out.
    ///
    /// Useful for the fleet rollup, which needs per-agent counts of `failing`
    /// and `degraded` health records without issuing one query per agent.
    ///
    /// A record counts only while its agent serves its object
    /// (`DeploymentObjectsDAL::served_pairs`): an agent reports on those
    /// objects only, so any other record is the last word on an object the
    /// agent no longer has (BROKKR-T-0393).
    ///
    /// # Returns
    ///
    /// Returns a Vec of `(agent_id, status, count)` tuples, or a
    /// diesel::result::Error on failure.
    pub fn status_counts_by_agent(
        &self,
    ) -> Result<Vec<(Uuid, String, i64)>, diesel::result::Error> {
        let served = self.dal.deployment_objects().served_pairs()?;
        let conn = &mut self.dal.conn()?;

        let rows: Vec<(Uuid, Uuid, String)> = deployment_health::table
            .select((
                deployment_health::agent_id,
                deployment_health::deployment_object_id,
                deployment_health::status,
            ))
            .load(conn)?;

        let mut counts: HashMap<(Uuid, String), i64> = HashMap::new();
        for (agent_id, object_id, status) in rows {
            if served.contains(&(agent_id, object_id)) {
                *counts.entry((agent_id, status)).or_insert(0) += 1;
            }
        }
        Ok(counts
            .into_iter()
            .map(|((agent_id, status), count)| (agent_id, status, count))
            .collect())
    }

    /// Deletes the health record for a specific agent and deployment object.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent UUID.
    /// * `deployment_object_id` - The deployment object UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of records deleted.
    pub fn delete_by_agent_and_deployment(
        &self,
        agent_id: Uuid,
        deployment_object_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        diesel::delete(
            deployment_health::table
                .filter(deployment_health::agent_id.eq(agent_id))
                .filter(deployment_health::deployment_object_id.eq(deployment_object_id)),
        )
        .execute(conn)
    }

    /// Deletes all health records for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of records deleted.
    pub fn delete_by_agent(&self, agent_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        diesel::delete(deployment_health::table.filter(deployment_health::agent_id.eq(agent_id)))
            .execute(conn)
    }
}
