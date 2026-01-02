/*
 * Copyright (c) 2025 Dylan Storey
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
    pub fn list_by_agent(&self, agent_id: Uuid) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_health::table
            .filter(deployment_health::agent_id.eq(agent_id))
            .order(deployment_health::checked_at.desc())
            .load(conn)
    }

    /// Lists all health records for deployment objects in a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The stack UUID.
    ///
    /// # Returns
    ///
    /// Returns a list of health records for all deployment objects in the stack.
    pub fn list_by_stack(&self, stack_id: Uuid) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_health::table
            .inner_join(deployment_objects::table)
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .select(deployment_health::all_columns)
            .order(deployment_health::checked_at.desc())
            .load(conn)
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
    pub fn list_by_status(&self, status: &str) -> Result<Vec<DeploymentHealth>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        deployment_health::table
            .filter(deployment_health::status.eq(status))
            .order(deployment_health::checked_at.desc())
            .load(conn)
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::delete(deployment_health::table.filter(deployment_health::agent_id.eq(agent_id)))
            .execute(conn)
    }
}
