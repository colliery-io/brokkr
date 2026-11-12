/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Stack operations.
//!
//! This module provides functionality to interact with the database for Stack-related operations,
//! including creating, retrieving, updating, and deleting stacks, as well as filtering stacks
//! based on various criteria.

use crate::dal::DAL;
use crate::dal::FilterType;
use crate::utils::event_bus;
use brokkr_models::models::stacks::{NewStack, Stack};
use brokkr_models::models::webhooks::{BrokkrEvent, EVENT_STACK_CREATED, EVENT_STACK_DELETED};
use brokkr_models::schema::{
    agent_annotations, agent_generator_registrations, agent_labels, agent_targets,
    stack_annotations, stack_labels, stacks,
};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Data Access Layer for Stack operations.
pub struct StacksDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl StacksDAL<'_> {
    /// Creates a new stack in the database.
    ///
    /// # Arguments
    ///
    /// * `new_stack` - A reference to the NewStack struct containing the stack details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created Stack on success, or a diesel::result::Error on failure.
    pub fn create(&self, new_stack: &NewStack) -> Result<Stack, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let stack: Stack = diesel::insert_into(stacks::table)
            .values(new_stack)
            .get_result(conn)?;

        // Emit stack.created event
        let event_data = serde_json::json!({
            "stack_id": stack.id,
            "name": stack.name,
            "description": stack.description,
            "created_at": stack.created_at,
        });
        event_bus::emit_event(self.dal, &BrokkrEvent::new(EVENT_STACK_CREATED, event_data));

        Ok(stack)
    }

    /// Retrieves non-deleted stacks by their UUIDs.
    ///
    /// # Arguments
    ///
    /// * `stack_uuids` - A Vec of UUIDs of the stacks to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec<Stack> of found non-deleted stacks, or a diesel::result::Error on failure.
    pub fn get(&self, stack_uuids: Vec<Uuid>) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        stacks::table
            .filter(stacks::id.eq_any(stack_uuids))
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)
    }

    /// Retrieves a stack by its UUID, including deleted stacks.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<Stack> if found (including deleted stacks), or a diesel::result::Error on failure.
    pub fn get_including_deleted(
        &self,
        stack_uuid: Uuid,
    ) -> Result<Option<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        stacks::table
            .filter(stacks::id.eq(stack_uuid))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted stacks from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted Stacks on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        stacks::table
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)
    }

    /// Lists all non-deleted stacks owned by a specific generator.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - The UUID of the generator whose stacks to list.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of non-deleted Stacks where `generator_id`
    /// matches, or a diesel::result::Error on failure.
    pub fn list_for_generator(
        &self,
        generator_id: Uuid,
    ) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        stacks::table
            .filter(stacks::deleted_at.is_null())
            .filter(stacks::generator_id.eq(generator_id))
            .load::<Stack>(conn)
    }

    /// Lists all stacks from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all Stacks (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        stacks::table.load::<Stack>(conn)
    }

    /// Updates an existing stack in the database.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to update.
    /// * `updated_stack` - A reference to the Stack struct containing the updated details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated Stack on success, or a diesel::result::Error on failure.
    pub fn update(
        &self,
        stack_uuid: Uuid,
        updated_stack: &Stack,
    ) -> Result<Stack, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::update(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .set(updated_stack)
            .get_result(conn)
    }

    /// Soft deletes a stack by setting its deleted_at timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn soft_delete(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let rows = diesel::update(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .set(stacks::deleted_at.eq(Utc::now()))
            .execute(conn)?;

        if rows > 0 {
            // Emit stack.deleted event
            let event_data = serde_json::json!({
                "stack_id": stack_uuid,
                "deleted_at": Utc::now(),
            });
            event_bus::emit_event(self.dal, &BrokkrEvent::new(EVENT_STACK_DELETED, event_data));
        }

        Ok(rows)
    }

    /// Hard deletes a stack from the database.
    ///
    /// # Arguments
    ///
    /// * `stack_uuid` - The UUID of the stack to hard delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1) on success, or a diesel::result::Error on failure.
    pub fn hard_delete(&self, stack_uuid: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(stacks::table.filter(stacks::id.eq(stack_uuid))).execute(conn)
    }

    /// Filters stacks by labels.
    ///
    /// # Arguments
    ///
    /// * `labels` - A vector of label strings to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple labels.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching Stacks on success, or a diesel::result::Error on failure.
    pub fn filter_by_labels(
        &self,
        labels: Vec<String>,
        filter_type: FilterType,
    ) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        match filter_type {
            FilterType::And => {
                let mut query = stacks::table
                    .filter(stacks::deleted_at.is_null())
                    .into_boxed();

                for label in &labels {
                    let subquery = stack_labels::table
                        .filter(stack_labels::stack_id.eq(stacks::id))
                        .filter(stack_labels::label.eq(label));
                    query = query.filter(diesel::dsl::exists(subquery));
                }

                query
                    .select(stacks::all_columns)
                    .distinct()
                    .load::<Stack>(conn)
            }
            FilterType::Or => stacks::table
                .inner_join(stack_labels::table)
                .filter(stacks::deleted_at.is_null())
                .filter(stack_labels::label.eq_any(labels))
                .select(stacks::all_columns)
                .distinct()
                .load::<Stack>(conn),
        }
    }

    /// Filters stacks by annotations.
    ///
    /// # Arguments
    ///
    /// * `annotations` - A vector of (key, value) pairs to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple annotations.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching Stacks on success, or a diesel::result::Error on failure.
    pub fn filter_by_annotations(
        &self,
        annotations: Vec<(String, String)>,
        filter_type: FilterType,
    ) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        match filter_type {
            FilterType::Or => {
                let mut all_matching_stacks = HashSet::new();

                for (key, value) in annotations {
                    let matching_stacks: Vec<Stack> = stacks::table
                        .inner_join(stack_annotations::table)
                        .filter(stacks::deleted_at.is_null())
                        .filter(stack_annotations::key.eq(key))
                        .filter(stack_annotations::value.eq(value))
                        .select(stacks::all_columns)
                        .load(conn)?;

                    all_matching_stacks.extend(matching_stacks);
                }

                Ok(all_matching_stacks.into_iter().collect())
            }
            FilterType::And => {
                if annotations.is_empty() {
                    return Ok(Vec::new());
                }

                // Use SQL EXISTS subqueries for efficient AND filtering (single query)
                let mut query = stacks::table
                    .filter(stacks::deleted_at.is_null())
                    .into_boxed();

                for (key, value) in &annotations {
                    let subquery = stack_annotations::table
                        .filter(stack_annotations::stack_id.eq(stacks::id))
                        .filter(stack_annotations::key.eq(key))
                        .filter(stack_annotations::value.eq(value));
                    query = query.filter(diesel::dsl::exists(subquery));
                }

                query
                    .select(stacks::all_columns)
                    .distinct()
                    .load::<Stack>(conn)
            }
        }
    }

    /// The pairs `(agent_id, stack_id)` where the agent serves the stack. This
    /// is the one rule for "does this agent serve this stack"; each reader of
    /// that question calls it (BROKKR-T-0395).
    ///
    /// An agent serves a stack when one of these holds:
    ///
    /// * it has a target for the stack (`agent_targets`);
    /// * it shares a label with the stack;
    /// * it shares a `(key, value)` annotation with the stack.
    ///
    /// Registration is the consent boundary (BROKKR-T-0287): a label or an
    /// annotation match counts only for a stack whose owning generator the
    /// agent is registered with. A generator declares which labels it pushes
    /// to; an agent's registrations declare which generators it accepts stacks
    /// from. Targets are exempt: they are gated at creation time (403
    /// `agent_not_registered`). Only non-deleted stacks count.
    ///
    /// # Arguments
    ///
    /// * `agent` - Only the pairs of this agent, or of every agent.
    /// * `stack` - Only the pairs of this stack, or of every stack.
    pub fn associations(
        &self,
        agent: Option<Uuid>,
        stack: Option<Uuid>,
    ) -> Result<HashSet<(Uuid, Uuid)>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let mut pairs: HashSet<(Uuid, Uuid)> = HashSet::new();

        // Targets.
        let mut by_target = agent_targets::table
            .inner_join(stacks::table.on(stacks::id.eq(agent_targets::stack_id)))
            .filter(stacks::deleted_at.is_null())
            .select((agent_targets::agent_id, agent_targets::stack_id))
            .into_boxed();
        if let Some(agent_id) = agent {
            by_target = by_target.filter(agent_targets::agent_id.eq(agent_id));
        }
        if let Some(stack_id) = stack {
            by_target = by_target.filter(agent_targets::stack_id.eq(stack_id));
        }
        pairs.extend(by_target.load::<(Uuid, Uuid)>(conn)?);

        // A shared label, with a stack of a generator the agent is registered with.
        let mut by_label = agent_labels::table
            .inner_join(stack_labels::table.on(stack_labels::label.eq(agent_labels::label)))
            .inner_join(stacks::table.on(stacks::id.eq(stack_labels::stack_id)))
            .inner_join(
                agent_generator_registrations::table.on(agent_generator_registrations::agent_id
                    .eq(agent_labels::agent_id)
                    .and(agent_generator_registrations::generator_id.eq(stacks::generator_id))),
            )
            .filter(stacks::deleted_at.is_null())
            .select((agent_labels::agent_id, stack_labels::stack_id))
            .into_boxed();
        if let Some(agent_id) = agent {
            by_label = by_label.filter(agent_labels::agent_id.eq(agent_id));
        }
        if let Some(stack_id) = stack {
            by_label = by_label.filter(stack_labels::stack_id.eq(stack_id));
        }
        pairs.extend(by_label.load::<(Uuid, Uuid)>(conn)?);

        // A shared annotation, with a stack of a generator the agent is
        // registered with.
        let mut by_annotation = agent_annotations::table
            .inner_join(
                stack_annotations::table.on(stack_annotations::key
                    .eq(agent_annotations::key)
                    .and(stack_annotations::value.eq(agent_annotations::value))),
            )
            .inner_join(stacks::table.on(stacks::id.eq(stack_annotations::stack_id)))
            .inner_join(
                agent_generator_registrations::table.on(agent_generator_registrations::agent_id
                    .eq(agent_annotations::agent_id)
                    .and(agent_generator_registrations::generator_id.eq(stacks::generator_id))),
            )
            .filter(stacks::deleted_at.is_null())
            .select((agent_annotations::agent_id, stack_annotations::stack_id))
            .into_boxed();
        if let Some(agent_id) = agent {
            by_annotation = by_annotation.filter(agent_annotations::agent_id.eq(agent_id));
        }
        if let Some(stack_id) = stack {
            by_annotation = by_annotation.filter(stack_annotations::stack_id.eq(stack_id));
        }
        pairs.extend(by_annotation.load::<(Uuid, Uuid)>(conn)?);

        Ok(pairs)
    }

    /// Whether the agent serves the stack, by the rule of [`Self::associations`].
    pub fn agent_serves(
        &self,
        agent_id: Uuid,
        stack_id: Uuid,
    ) -> Result<bool, diesel::result::Error> {
        Ok(!self
            .associations(Some(agent_id), Some(stack_id))?
            .is_empty())
    }

    /// The agents that serve the stack, by the rule of [`Self::associations`],
    /// sorted by id.
    pub fn agents_serving(&self, stack_id: Uuid) -> Result<Vec<Uuid>, diesel::result::Error> {
        let mut agents: Vec<Uuid> = self
            .associations(None, Some(stack_id))?
            .into_iter()
            .map(|(agent_id, _)| agent_id)
            .collect();
        agents.sort();
        Ok(agents)
    }

    /// Retrieves the stacks that an agent serves, by the rule of
    /// [`Self::associations`]: its targets, and the stacks that share a label or
    /// an annotation with it, within the generators it is registered with.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of associated Stacks, sorted by id, on
    /// success, or a diesel::result::Error on failure.
    pub fn get_associated_stacks(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<Stack>, diesel::result::Error> {
        let stack_ids: Vec<Uuid> = self
            .associations(Some(agent_id), None)?
            .into_iter()
            .map(|(_, stack_id)| stack_id)
            .collect();
        if stack_ids.is_empty() {
            return Ok(Vec::new());
        }

        let conn = &mut self.dal.conn()?;
        let mut result: Vec<Stack> = stacks::table
            .filter(stacks::id.eq_any(stack_ids))
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)?;
        result.sort_by_key(|a| a.id);

        Ok(result)
    }
}
