//! Data Access Layer for DeploymentObject operations.
//!
//! This module provides functionality to interact with deployment objects in the database,
//! including creating, retrieving, listing, and soft-deleting deployment objects.

use crate::dal::DAL;
use brokkr_models::models::agent_targets::AgentTarget;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::schema::agent_targets;
use brokkr_models::schema::deployment_objects;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for DeploymentObject operations.
pub struct DeploymentObjectsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl<'a> DeploymentObjectsDAL<'a> {
    /// Creates a new deployment object in the database.
    ///
    /// # Arguments
    ///
    /// * `new_deployment_object` - A reference to the NewDeploymentObject struct containing the deployment object details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created DeploymentObject on success, or a diesel::result::Error on failure.
    pub fn create(
        &self,
        new_deployment_object: &NewDeploymentObject,
    ) -> Result<DeploymentObject, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(deployment_objects::table)
            .values(new_deployment_object)
            .get_result(conn)
    }

    /// Retrieves a non-deleted deployment object by its UUID.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found (and not deleted), or a diesel::result::Error on failure.
    pub fn get(
        &self,
        deployment_object_uuid: Uuid,
    ) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::id.eq(deployment_object_uuid))
            .filter(deployment_objects::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves a deployment object by its UUID, including deleted objects.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found (including deleted objects), or a diesel::result::Error on failure.
    pub fn get_including_deleted(
        &self,
        deployment_object_uuid: Uuid,
    ) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::id.eq(deployment_object_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted deployment objects for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted DeploymentObjects for the specified stack on success, or a diesel::result::Error on failure.
    pub fn list_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)
    }

    /// Lists all deployment objects for a specific stack, including deleted ones.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to list deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all DeploymentObjects for the specified stack (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)
    }

    /// Soft deletes a deployment object by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `deployment_object_uuid` - The UUID of the deployment object to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(
        &self,
        deployment_object_uuid: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(
            deployment_objects::table.filter(deployment_objects::id.eq(deployment_object_uuid)),
        )
        .set(deployment_objects::deleted_at.eq(Utc::now()))
        .execute(conn)
    }

    /// Retrieves the latest non-deleted deployment object for a specific stack.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to retrieve the latest deployment object for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<DeploymentObject> if found, or a diesel::result::Error on failure.
    pub fn get_latest_for_stack(
        &self,
        stack_id: Uuid,
    ) -> Result<Option<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::stack_id.eq(stack_id))
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .first(conn)
            .optional()
    }

    /// Retrieves a list of undeployed objects for an agent based on its responsibilities.
    ///
    /// This method performs the following steps:
    /// 1. Get the list of stacks the agent is responsible for
    /// 2. Get all deployment objects for these stacks
    /// 3. Filter out objects that have been deployed (have corresponding agent events) if include_deployed is false
    /// 4. Sort by sequence_id in descending order
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to get undeployed objects for.
    /// * `include_deployed` - Whether to include objects that have already been deployed.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of DeploymentObjects for the agent,
    /// sorted by sequence_id in descending order (most recent first), or a diesel::result::Error on failure.
    pub fn get_target_state_for_agent(
        &self,
        agent_id: Uuid,
        include_deployed: bool,
    ) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        // Step 1: Get the list of stacks the agent is responsible for
        let responsible_stacks = self.dal.stacks().get_associated_stacks(agent_id)?;

        // Step 2: Get all deployment objects for these stacks
        let mut all_objects = Vec::new();
        for stack in responsible_stacks {
            let stack_objects = self.get_latest_for_stack(stack.id)?;
            all_objects.extend(stack_objects);
        }

        // Step 3: Filter out objects that have been deployed (have corresponding agent events) if include_deployed is false
        let objects = if include_deployed {
            all_objects
        } else {
            let deployed_object_ids = self
                .dal
                .agent_events()
                .get_events(None, Some(agent_id))?
                .into_iter()
                .map(|event| event.deployment_object_id)
                .collect::<Vec<Uuid>>();

            all_objects
                .into_iter()
                .filter(|obj| !deployed_object_ids.contains(&obj.id))
                .collect::<Vec<DeploymentObject>>()
        };

        // Step 4: Sort by sequence_id in descending order
        let mut sorted_objects = objects;
        sorted_objects.sort_by(|a, b| b.sequence_id.cmp(&a.sequence_id));

        Ok(sorted_objects)
    }

    /// Searches for deployment objects by checksum.
    ///
    /// # Arguments
    ///
    /// * `checksum` - The checksum to search for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of DeploymentObjects that match the checksum,
    /// or a diesel::result::Error on failure.
    pub fn search(
        &self,
        yaml_checksum: &str,
    ) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        deployment_objects::table
            .filter(deployment_objects::yaml_checksum.eq(yaml_checksum))
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)
    }

    /// Retrieves applicable deployment objects for a given agent.
    ///
    /// This method fetches deployment objects based on the agent's targets and filters out deleted objects.
    /// The results are sorted by sequence_id in descending order.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to retrieve applicable deployment objects for.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of DeploymentObjects that are applicable for the agent,
    /// sorted by sequence_id in descending order (most recent first), or a diesel::result::Error on failure.
    pub fn get_desired_state_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<DeploymentObject>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let agent_targets = agent_targets::table
            .filter(agent_targets::agent_id.eq(agent_id))
            .load::<AgentTarget>(conn)?;

        let applicable_objects = deployment_objects::table
            .filter(
                deployment_objects::stack_id
                    .eq_any(agent_targets.iter().map(|target| target.stack_id)),
            )
            .filter(deployment_objects::deleted_at.is_null())
            .order(deployment_objects::sequence_id.desc())
            .load::<DeploymentObject>(conn)?;

        Ok(applicable_objects)
    }
}
