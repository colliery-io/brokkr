/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for DeploymentObject operations.
//!
//! This module provides functionality to interact with deployment objects in the database,
//! including creating, retrieving, listing, and soft-deleting deployment objects.

use crate::dal::DAL;
use crate::utils::event_bus;
use brokkr_models::models::agent_targets::AgentTarget;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::models::webhooks::{
    BrokkrEvent, EVENT_DEPLOYMENT_CREATED, EVENT_DEPLOYMENT_DELETED,
};
use brokkr_models::schema::{
    agent_annotations, agent_events, agent_labels, agent_targets, deployment_objects,
    stack_annotations, stack_labels, stacks,
};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Data Access Layer for DeploymentObject operations.
pub struct DeploymentObjectsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl DeploymentObjectsDAL<'_> {
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
        let conn = &mut self.dal.conn()?;
        let deployment_object: DeploymentObject = diesel::insert_into(deployment_objects::table)
            .values(new_deployment_object)
            .get_result(conn)?;

        // Emit deployment.created event
        let event_data = serde_json::json!({
            "deployment_object_id": deployment_object.id,
            "stack_id": deployment_object.stack_id,
            "sequence_id": deployment_object.sequence_id,
            "created_at": deployment_object.created_at,
        });
        event_bus::emit_event(
            self.dal,
            &BrokkrEvent::new(EVENT_DEPLOYMENT_CREATED, event_data),
        );

        Ok(deployment_object)
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
        let conn = &mut self.dal.conn()?;
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
        let conn = &mut self.dal.conn()?;
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
        let conn = &mut self.dal.conn()?;
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
        let conn = &mut self.dal.conn()?;
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
        let conn = &mut self.dal.conn()?;

        // Get the deployment object first for event data
        let deployment_object: Option<DeploymentObject> = deployment_objects::table
            .filter(deployment_objects::id.eq(deployment_object_uuid))
            .first(conn)
            .optional()?;

        let rows = diesel::update(
            deployment_objects::table.filter(deployment_objects::id.eq(deployment_object_uuid)),
        )
        .set(deployment_objects::deleted_at.eq(Utc::now()))
        .execute(conn)?;

        if rows > 0 {
            // Emit deployment.deleted event
            let event_data = serde_json::json!({
                "deployment_object_id": deployment_object_uuid,
                "stack_id": deployment_object.map(|d| d.stack_id),
                "deleted_at": Utc::now(),
            });
            event_bus::emit_event(
                self.dal,
                &BrokkrEvent::new(EVENT_DEPLOYMENT_DELETED, event_data),
            );
        }

        Ok(rows)
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
        let conn = &mut self.dal.conn()?;
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

    /// Returns, per agent, the number of pending deployment objects — computed
    /// with a handful of bounded set-based queries rather than calling
    /// [`get_target_state_for_agent`] once per agent.
    ///
    /// The result exactly mirrors
    /// `get_target_state_for_agent(agent_id, false).len()` (the per-agent ground
    /// truth): for each stack the agent is responsible for, take that stack's
    /// latest non-deleted deployment object (max `sequence_id`), then drop any
    /// object for which the agent already has a (non-deleted) `agent_event`.
    ///
    /// Agent→stack responsibility mirrors `stacks::get_associated_stacks`: the
    /// UNION of hard targets (`agent_targets`), stacks sharing any label with the
    /// agent, and stacks sharing any `(key, value)` annotation with the agent.
    /// Only non-deleted stacks participate.
    ///
    /// # Returns
    ///
    /// Returns a Vec of `(agent_id, pending_count)` pairs, or a
    /// diesel::result::Error on failure.
    pub fn pending_counts_by_agent(&self) -> Result<Vec<(Uuid, i64)>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        // (a) Build the distinct set of (agent_id, stack_id) responsibilities
        // from the three association sources, over non-deleted stacks only.
        let mut associations: HashSet<(Uuid, Uuid)> = HashSet::new();

        // Hard targets.
        let target_pairs: Vec<(Uuid, Uuid)> = agent_targets::table
            .inner_join(stacks::table.on(stacks::id.eq(agent_targets::stack_id)))
            .filter(stacks::deleted_at.is_null())
            .select((agent_targets::agent_id, agent_targets::stack_id))
            .load::<(Uuid, Uuid)>(conn)?;
        associations.extend(target_pairs);

        // Shared label: agent_labels.label == stack_labels.label.
        let label_pairs: Vec<(Uuid, Uuid)> = agent_labels::table
            .inner_join(stack_labels::table.on(stack_labels::label.eq(agent_labels::label)))
            .inner_join(stacks::table.on(stacks::id.eq(stack_labels::stack_id)))
            .filter(stacks::deleted_at.is_null())
            .select((agent_labels::agent_id, stack_labels::stack_id))
            .load::<(Uuid, Uuid)>(conn)?;
        associations.extend(label_pairs);

        // Shared annotation: agent_annotations.(key,value) == stack_annotations.(key,value).
        let annotation_pairs: Vec<(Uuid, Uuid)> = agent_annotations::table
            .inner_join(
                stack_annotations::table.on(stack_annotations::key
                    .eq(agent_annotations::key)
                    .and(stack_annotations::value.eq(agent_annotations::value))),
            )
            .inner_join(stacks::table.on(stacks::id.eq(stack_annotations::stack_id)))
            .filter(stacks::deleted_at.is_null())
            .select((agent_annotations::agent_id, stack_annotations::stack_id))
            .load::<(Uuid, Uuid)>(conn)?;
        associations.extend(annotation_pairs);

        // (b) Latest non-deleted deployment object per stack (max sequence_id),
        // mirroring `get_latest_for_stack`. We load all (stack_id, id,
        // sequence_id) for non-deleted objects and reduce in memory.
        let object_rows: Vec<(Uuid, Uuid, i64)> = deployment_objects::table
            .filter(deployment_objects::deleted_at.is_null())
            .select((
                deployment_objects::stack_id,
                deployment_objects::id,
                deployment_objects::sequence_id,
            ))
            .load::<(Uuid, Uuid, i64)>(conn)?;
        // stack_id -> (object_id, sequence_id) of the max-sequence object.
        let mut latest_by_stack: HashMap<Uuid, (Uuid, i64)> = HashMap::new();
        for (stack_id, object_id, sequence_id) in object_rows {
            latest_by_stack
                .entry(stack_id)
                .and_modify(|cur| {
                    if sequence_id > cur.1 {
                        *cur = (object_id, sequence_id);
                    }
                })
                .or_insert((object_id, sequence_id));
        }

        // (c) Set of (agent_id, deployment_object_id) acknowledged via a
        // non-deleted agent_event.
        let event_pairs: Vec<(Uuid, Uuid)> = agent_events::table
            .filter(agent_events::deleted_at.is_null())
            .select((
                agent_events::agent_id,
                agent_events::deployment_object_id,
            ))
            .load::<(Uuid, Uuid)>(conn)?;
        let acknowledged: HashSet<(Uuid, Uuid)> = event_pairs.into_iter().collect();

        // Aggregate: for each (agent, stack), look up the stack's latest object;
        // if the agent has no event for it, count it as pending.
        let mut counts: HashMap<Uuid, i64> = HashMap::new();
        for (agent_id, stack_id) in associations {
            if let Some((object_id, _seq)) = latest_by_stack.get(&stack_id)
                && !acknowledged.contains(&(agent_id, *object_id))
            {
                *counts.entry(agent_id).or_insert(0) += 1;
            }
        }

        Ok(counts.into_iter().collect())
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
        let conn = &mut self.dal.conn()?;
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
        let conn = &mut self.dal.conn()?;

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
