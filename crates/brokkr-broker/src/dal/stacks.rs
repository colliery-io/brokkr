/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Stack operations.
//!
//! This module provides functionality to interact with the database for Stack-related operations,
//! including creating, retrieving, updating, and deleting stacks, as well as filtering stacks
//! based on various criteria.

use crate::dal::FilterType;
use crate::dal::DAL;
use brokkr_models::models::stacks::{NewStack, Stack};
use brokkr_models::schema::{
    agent_annotations, agent_labels, agent_targets, stack_annotations, stack_labels, stacks,
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

impl<'a> StacksDAL<'a> {
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(stacks::table)
            .values(new_stack)
            .get_result(conn)
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stacks::table
            .filter(stacks::deleted_at.is_null())
            .load::<Stack>(conn)
    }

    /// Lists all stacks from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all Stacks (including deleted ones) on success, or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(stacks::table.filter(stacks::id.eq(stack_uuid)))
            .set(stacks::deleted_at.eq(Utc::now()))
            .execute(conn)
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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

                let mut all_matching_stacks: Option<HashSet<Stack>> = None;

                for (key, value) in annotations {
                    let matching_stacks: HashSet<Stack> = stacks::table
                        .inner_join(stack_annotations::table)
                        .filter(stacks::deleted_at.is_null())
                        .filter(stack_annotations::key.eq(key))
                        .filter(stack_annotations::value.eq(value))
                        .select(stacks::all_columns)
                        .load(conn)?
                        .into_iter()
                        .collect();

                    all_matching_stacks = match all_matching_stacks {
                        Some(stacks) => {
                            Some(stacks.intersection(&matching_stacks).cloned().collect())
                        }
                        None => Some(matching_stacks),
                    };

                    if let Some(ref stacks) = all_matching_stacks {
                        if stacks.is_empty() {
                            break;
                        }
                    }
                }

                Ok(
                    all_matching_stacks
                        .map_or_else(Vec::new, |stacks| stacks.into_iter().collect()),
                )
            }
        }
    }

    /// Retrieves all stacks associated with a specific agent based on its labels, annotations, and targets.
    ///
    /// This method uses OR logic when matching labels and annotations, meaning a stack will be included
    /// if it matches any of the agent's labels or annotations.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of associated Stacks on success, or a diesel::result::Error on failure.
    pub fn get_associated_stacks(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<Stack>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Get agent labels
        let labels: Vec<String> = agent_labels::table
            .filter(agent_labels::agent_id.eq(agent_id))
            .select(agent_labels::label)
            .load::<String>(conn)?;

        // Get agent annotations
        let annotations: Vec<(String, String)> = agent_annotations::table
            .filter(agent_annotations::agent_id.eq(agent_id))
            .select((agent_annotations::key, agent_annotations::value))
            .load::<(String, String)>(conn)?;

        // Get agent targets
        let targets: Vec<Uuid> = agent_targets::table
            .filter(agent_targets::agent_id.eq(agent_id))
            .select(agent_targets::stack_id)
            .load::<Uuid>(conn)?;

        let mut associated_stacks = HashSet::new();

        // Get stacks matching labels
        if !labels.is_empty() {
            let label_stacks = self.filter_by_labels(labels, FilterType::Or)?;

            associated_stacks.extend(label_stacks);
        }

        // Get stacks matching annotations
        if !annotations.is_empty() {
            let annotation_stacks = self.filter_by_annotations(annotations, FilterType::Or)?;

            associated_stacks.extend(annotation_stacks);
        }

        // Get stacks matching targets
        if !targets.is_empty() {
            let target_stacks: Vec<Stack> = stacks::table
                .filter(stacks::id.eq_any(targets))
                .filter(stacks::deleted_at.is_null())
                .load::<Stack>(conn)?;

            associated_stacks.extend(target_stacks);
        }

        // Convert HashSet to Vec, sort, and return
        let mut result: Vec<Stack> = associated_stacks.into_iter().collect();
        result.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(result)
    }
}
