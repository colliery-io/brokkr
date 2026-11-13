/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for WorkOrder operations.
//!
//! This module provides functionality to interact with the work_orders, work_order_log,
//! and work_order_targets tables in the database. It includes methods for creating,
//! claiming, completing, and managing work orders through their lifecycle.
//!
//! ## Work Order Lifecycle
//!
//! 1. Create: Work order is created with status PENDING
//! 2. Claim: Agent claims work order, status changes to CLAIMED
//! 3. Complete: Agent reports completion, record moves to work_order_log
//!    - On failure with retries remaining: status changes to RETRY_PENDING
//!    - After backoff: status resets to PENDING
//!
//! ## Stale Claim Detection
//!
//! Work orders that have been claimed but not completed within `claim_timeout_seconds`
//! are considered stale and can be reclaimed by other agents.

use crate::dal::DAL;
use crate::utils::event_bus;
use brokkr_models::models::webhooks::{
    BrokkrEvent, EVENT_WORKORDER_CLAIMED, EVENT_WORKORDER_COMPLETED, EVENT_WORKORDER_CREATED,
    EVENT_WORKORDER_FAILED,
};
use brokkr_models::models::work_order_annotations::{NewWorkOrderAnnotation, WorkOrderAnnotation};
use brokkr_models::models::work_order_labels::{NewWorkOrderLabel, WorkOrderLabel};
use brokkr_models::models::work_orders::{
    NewWorkOrder, NewWorkOrderLog, NewWorkOrderTarget, WORK_ORDER_STATUS_CLAIMED,
    WORK_ORDER_STATUS_PENDING, WORK_ORDER_STATUS_RETRY_PENDING, WorkOrder, WorkOrderLog,
    WorkOrderTarget,
};
use brokkr_models::schema::{
    agent_annotations, agent_generator_registrations, agent_labels, work_order_annotations,
    work_order_labels, work_order_log, work_order_targets, work_orders,
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Data Access Layer for WorkOrder operations.
pub struct WorkOrdersDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl WorkOrdersDAL<'_> {
    // =========================================================================
    // WORK ORDER CRUD OPERATIONS
    // =========================================================================

    /// Creates a new work order in the database.
    ///
    /// # Arguments
    ///
    /// * `new_work_order` - A reference to the NewWorkOrder struct.
    ///
    /// # Returns
    ///
    /// Returns the created WorkOrder on success, or a diesel::result::Error on failure.
    pub fn create(
        &self,
        new_work_order: &NewWorkOrder,
    ) -> Result<WorkOrder, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let work_order: WorkOrder = diesel::insert_into(work_orders::table)
            .values(new_work_order)
            .get_result(conn)?;

        // Emit workorder.created event
        let event_data = serde_json::json!({
            "work_order_id": work_order.id,
            "work_type": work_order.work_type,
            "status": work_order.status,
            "created_at": work_order.created_at,
        });
        event_bus::emit_event(
            self.dal,
            &BrokkrEvent::new(EVENT_WORKORDER_CREATED, event_data),
        );

        Ok(work_order)
    }

    /// Retrieves a work order by its UUID.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order to retrieve.
    ///
    /// # Returns
    ///
    /// Returns an Option<WorkOrder> if found, or a diesel::result::Error on failure.
    pub fn get(&self, work_order_id: Uuid) -> Result<Option<WorkOrder>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_orders::table
            .filter(work_orders::id.eq(work_order_id))
            .first(conn)
            .optional()
    }

    /// Lists all work orders from the database.
    ///
    /// # Returns
    ///
    /// Returns a Vec of all WorkOrders on success, or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<WorkOrder>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_orders::table
            .order(work_orders::created_at.desc())
            .load::<WorkOrder>(conn)
    }

    /// Lists work orders filtered by status and/or work type.
    ///
    /// # Arguments
    ///
    /// * `status` - Optional status filter (PENDING, CLAIMED, RETRY_PENDING).
    /// * `work_type` - Optional work type filter (e.g., "build").
    ///
    /// # Returns
    ///
    /// Returns a Vec of filtered WorkOrders on success, or a diesel::result::Error on failure.
    pub fn list_filtered(
        &self,
        status: Option<&str>,
        work_type: Option<&str>,
    ) -> Result<Vec<WorkOrder>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let mut query = work_orders::table.into_boxed();

        if let Some(s) = status {
            query = query.filter(work_orders::status.eq(s));
        }

        if let Some(wt) = work_type {
            query = query.filter(work_orders::work_type.eq(wt));
        }

        query
            .order(work_orders::created_at.desc())
            .load::<WorkOrder>(conn)
    }

    /// Lists the work orders that a generator owns, with the filters of
    /// [`Self::list_filtered`]. The owner is part of the query, so a generator
    /// never loads the work orders of another owner (BROKKR-A-0013).
    pub fn list_owned(
        &self,
        owner_generator_id: Uuid,
        status: Option<&str>,
        work_type: Option<&str>,
    ) -> Result<Vec<WorkOrder>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let mut query = work_orders::table
            .filter(work_orders::owner_generator_id.eq(owner_generator_id))
            .into_boxed();

        if let Some(s) = status {
            query = query.filter(work_orders::status.eq(s));
        }

        if let Some(wt) = work_type {
            query = query.filter(work_orders::work_type.eq(wt));
        }

        query
            .order(work_orders::created_at.desc())
            .load::<WorkOrder>(conn)
    }

    /// Deletes a work order by its UUID (hard delete).
    ///
    /// Note: This also deletes associated work_order_targets via CASCADE.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order to delete.
    ///
    /// # Returns
    ///
    /// Returns the number of affected rows on success, or a diesel::result::Error on failure.
    pub fn delete(&self, work_order_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(work_orders::table.filter(work_orders::id.eq(work_order_id))).execute(conn)
    }

    // =========================================================================
    // CLAIM OPERATIONS
    // =========================================================================

    /// Returns the `(agent_id, work_order_id)` pairs of the PENDING work orders
    /// that each agent can have. This is the one rule for "can this agent have
    /// this work order" (BROKKR-T-0403); the pending list, the claim and the
    /// fleet count all read it.
    ///
    /// An agent can have a pending work order if ANY of these holds (OR logic):
    /// - the agent is a hard target (`work_order_targets`), OR
    /// - the agent shares a label with the work order, OR
    /// - the agent shares a `(key, value)` annotation with the work order.
    ///
    /// A work order that a generator owns (`owner_generator_id`) reaches only
    /// the agents registered with that generator: registration is the consent
    /// boundary (BROKKR-A-0009, BROKKR-A-0013). The gate is on each of the
    /// three legs, the hard target too: nothing removes a target row when an
    /// agent is deregistered. A work order with no owner is an admin's and has
    /// no gate.
    ///
    /// # Arguments
    ///
    /// * `agent` - Only this agent, or every agent.
    /// * `work_order` - Only this work order, or every pending work order.
    pub fn matches(
        &self,
        agent: Option<Uuid>,
        work_order: Option<Uuid>,
    ) -> Result<HashSet<(Uuid, Uuid)>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        // (agent_id, work_order_id, owner) of each leg, before the gate.
        let mut candidates: Vec<(Uuid, Uuid, Option<Uuid>)> = Vec::new();

        // 1. Hard targets.
        let mut targets = work_order_targets::table
            .inner_join(work_orders::table)
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .select((
                work_order_targets::agent_id,
                work_orders::id,
                work_orders::owner_generator_id,
            ))
            .into_boxed();
        if let Some(agent_id) = agent {
            targets = targets.filter(work_order_targets::agent_id.eq(agent_id));
        }
        if let Some(work_order_id) = work_order {
            targets = targets.filter(work_orders::id.eq(work_order_id));
        }
        candidates.extend(targets.load::<(Uuid, Uuid, Option<Uuid>)>(conn)?);

        // 2. Labels: agent_labels joined to work_order_labels on the label.
        let mut labels = agent_labels::table
            .inner_join(
                work_order_labels::table.on(work_order_labels::label.eq(agent_labels::label)),
            )
            .inner_join(work_orders::table.on(work_orders::id.eq(work_order_labels::work_order_id)))
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .select((
                agent_labels::agent_id,
                work_orders::id,
                work_orders::owner_generator_id,
            ))
            .into_boxed();
        if let Some(agent_id) = agent {
            labels = labels.filter(agent_labels::agent_id.eq(agent_id));
        }
        if let Some(work_order_id) = work_order {
            labels = labels.filter(work_orders::id.eq(work_order_id));
        }
        candidates.extend(labels.load::<(Uuid, Uuid, Option<Uuid>)>(conn)?);

        // 3. Annotations: agent_annotations joined to work_order_annotations
        //    on the (key, value) pair.
        let mut annotations = agent_annotations::table
            .inner_join(
                work_order_annotations::table.on(work_order_annotations::key
                    .eq(agent_annotations::key)
                    .and(work_order_annotations::value.eq(agent_annotations::value))),
            )
            .inner_join(
                work_orders::table.on(work_orders::id.eq(work_order_annotations::work_order_id)),
            )
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .select((
                agent_annotations::agent_id,
                work_orders::id,
                work_orders::owner_generator_id,
            ))
            .into_boxed();
        if let Some(agent_id) = agent {
            annotations = annotations.filter(agent_annotations::agent_id.eq(agent_id));
        }
        if let Some(work_order_id) = work_order {
            annotations = annotations.filter(work_orders::id.eq(work_order_id));
        }
        candidates.extend(annotations.load::<(Uuid, Uuid, Option<Uuid>)>(conn)?);

        // The gate: a generator's work order, only for its registered agents.
        let owners: HashSet<Uuid> = candidates.iter().filter_map(|c| c.2).collect();
        let registered: HashSet<(Uuid, Uuid)> = if owners.is_empty() {
            HashSet::new()
        } else {
            let owners: Vec<Uuid> = owners.into_iter().collect();
            let mut registrations = agent_generator_registrations::table
                .filter(agent_generator_registrations::generator_id.eq_any(owners))
                .select((
                    agent_generator_registrations::agent_id,
                    agent_generator_registrations::generator_id,
                ))
                .into_boxed();
            if let Some(agent_id) = agent {
                registrations =
                    registrations.filter(agent_generator_registrations::agent_id.eq(agent_id));
            }
            registrations
                .load::<(Uuid, Uuid)>(conn)?
                .into_iter()
                .collect()
        };

        Ok(candidates
            .into_iter()
            .filter(|(agent_id, _, owner)| match owner {
                Some(generator_id) => registered.contains(&(*agent_id, *generator_id)),
                None => true,
            })
            .map(|(agent_id, work_order_id, _)| (agent_id, work_order_id))
            .collect())
    }

    /// The agents of `agent_ids` that are not registered with `owner`. A
    /// generator cannot name one of them as a target of its work order
    /// (BROKKR-T-0403). The order of `agent_ids` is kept.
    pub fn unregistered_targets(
        &self,
        owner: Uuid,
        agent_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, diesel::result::Error> {
        if agent_ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = &mut self.dal.conn()?;
        let registered: HashSet<Uuid> = agent_generator_registrations::table
            .filter(agent_generator_registrations::generator_id.eq(owner))
            .filter(agent_generator_registrations::agent_id.eq_any(agent_ids))
            .select(agent_generator_registrations::agent_id)
            .load::<Uuid>(conn)?
            .into_iter()
            .collect();
        Ok(agent_ids
            .iter()
            .copied()
            .filter(|id| !registered.contains(id))
            .collect())
    }

    /// Lists pending work orders that are claimable by a specific agent: the
    /// work orders that [`Self::matches`] gives for the agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent requesting claimable work orders.
    /// * `work_type` - Optional work type filter.
    ///
    /// # Returns
    ///
    /// Returns a Vec of claimable WorkOrders on success, or a diesel::result::Error on failure.
    pub fn list_pending_for_agent(
        &self,
        agent_id: Uuid,
        work_type: Option<&str>,
    ) -> Result<Vec<WorkOrder>, diesel::result::Error> {
        let ids: Vec<Uuid> = self
            .matches(Some(agent_id), None)?
            .into_iter()
            .map(|(_, work_order_id)| work_order_id)
            .collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let conn = &mut self.dal.conn()?;
        let mut query = work_orders::table
            .filter(work_orders::id.eq_any(&ids))
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .into_boxed();

        if let Some(wt) = work_type {
            query = query.filter(work_orders::work_type.eq(wt));
        }

        query
            .order(work_orders::created_at.asc())
            .load::<WorkOrder>(conn)
    }

    /// Returns the number of CLAIMED work orders per claiming agent in a single
    /// grouped query (no per-agent fan-out).
    ///
    /// Counts rows `WHERE status='CLAIMED' AND claimed_by IS NOT NULL` grouped
    /// by `claimed_by`. Agents with no claims are absent from the result.
    ///
    /// # Returns
    ///
    /// Returns a Vec of `(agent_id, claimed_count)` pairs, or a
    /// diesel::result::Error on failure.
    pub fn claimed_counts_by_agent(&self) -> Result<Vec<(Uuid, i64)>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_orders::table
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_CLAIMED))
            .filter(work_orders::claimed_by.is_not_null())
            .group_by(work_orders::claimed_by)
            .select((
                work_orders::claimed_by.assume_not_null(),
                diesel::dsl::count_star(),
            ))
            .load::<(Uuid, i64)>(conn)
    }

    /// Returns, per agent, the number of distinct PENDING work orders that the
    /// agent can have: the pairs of [`Self::matches`] for every agent, counted
    /// per agent. So the count is the length of [`Self::list_pending_for_agent`]
    /// for each agent, with no query per agent.
    ///
    /// # Returns
    ///
    /// Returns a Vec of `(agent_id, pending_count)` pairs, or a
    /// diesel::result::Error on failure.
    pub fn pending_counts_by_agent(&self) -> Result<Vec<(Uuid, i64)>, diesel::result::Error> {
        let mut counts: std::collections::HashMap<Uuid, i64> = std::collections::HashMap::new();
        for (agent_id, _work_order_id) in self.matches(None, None)? {
            *counts.entry(agent_id).or_insert(0) += 1;
        }
        Ok(counts.into_iter().collect())
    }

    /// Atomically claims a work order for an agent.
    ///
    /// This operation will only succeed if:
    /// - The work order exists
    /// - The work order status is PENDING
    /// - The agent can have the work order: [`Self::matches`] has the pair
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order to claim.
    /// * `agent_id` - The UUID of the agent claiming the work order.
    ///
    /// # Returns
    ///
    /// Returns the updated WorkOrder on success, or a diesel::result::Error on failure.
    /// Returns NotFound if the work order doesn't exist or can't be claimed.
    pub fn claim(
        &self,
        work_order_id: Uuid,
        agent_id: Uuid,
    ) -> Result<WorkOrder, diesel::result::Error> {
        // The one rule: target, label or annotation, inside the registrations
        // of the owner when a generator owns the work order.
        if !self
            .matches(Some(agent_id), Some(work_order_id))?
            .contains(&(agent_id, work_order_id))
        {
            return Err(diesel::result::Error::NotFound);
        }

        let conn = &mut self.dal.conn()?;

        // Atomically update the work order status to CLAIMED
        let now = Utc::now();
        let work_order: WorkOrder = diesel::update(
            work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING)),
        )
        .set((
            work_orders::status.eq(WORK_ORDER_STATUS_CLAIMED),
            work_orders::claimed_by.eq(agent_id),
            work_orders::claimed_at.eq(now),
        ))
        .get_result(conn)?;

        // Emit workorder.claimed event
        let event_data = serde_json::json!({
            "work_order_id": work_order.id,
            "work_type": work_order.work_type,
            "agent_id": agent_id,
            "claimed_at": now,
        });
        event_bus::emit_event(
            self.dal,
            &BrokkrEvent::new(EVENT_WORKORDER_CLAIMED, event_data),
        );

        Ok(work_order)
    }

    /// Releases a claimed work order back to PENDING status.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order to release.
    /// * `agent_id` - The UUID of the agent releasing the work order (must match claimed_by).
    ///
    /// # Returns
    ///
    /// Returns the updated WorkOrder on success, or a diesel::result::Error on failure.
    pub fn release(
        &self,
        work_order_id: Uuid,
        agent_id: Uuid,
    ) -> Result<WorkOrder, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        diesel::update(
            work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .filter(work_orders::status.eq(WORK_ORDER_STATUS_CLAIMED))
                .filter(work_orders::claimed_by.eq(agent_id)),
        )
        .set((
            work_orders::status.eq(WORK_ORDER_STATUS_PENDING),
            work_orders::claimed_by.eq(None::<Uuid>),
            work_orders::claimed_at.eq(None::<chrono::DateTime<Utc>>),
        ))
        .get_result(conn)
    }

    // =========================================================================
    // COMPLETION OPERATIONS
    // =========================================================================

    /// Completes a work order successfully and moves it to the log.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order to complete.
    /// * `result_message` - Optional result message (e.g., image digest).
    ///
    /// # Returns
    ///
    /// Returns the created WorkOrderLog entry on success, or a diesel::result::Error on failure.
    pub fn complete_success(
        &self,
        work_order_id: Uuid,
        result_message: Option<String>,
    ) -> Result<WorkOrderLog, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let log_result: WorkOrderLog = conn.transaction(|conn| {
            // Get the work order
            let work_order: WorkOrder = work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .first(conn)?;

            // Create log entry
            let log_entry = NewWorkOrderLog::from_work_order(&work_order, true, result_message);
            let log_result: WorkOrderLog = diesel::insert_into(work_order_log::table)
                .values(&log_entry)
                .get_result(conn)?;

            // Delete work order (cascade deletes targets)
            diesel::delete(work_orders::table.filter(work_orders::id.eq(work_order_id)))
                .execute(conn)?;

            Ok::<WorkOrderLog, diesel::result::Error>(log_result)
        })?;

        // Emit completion event
        self.emit_completion_event(&log_result);

        Ok(log_result)
    }

    /// Emits a work order completion event.
    /// Emits `workorder.completed` on success, `workorder.failed` on failure.
    fn emit_completion_event(&self, log: &WorkOrderLog) {
        let event_data = serde_json::json!({
            "work_order_log_id": log.id,
            "work_type": log.work_type,
            "success": log.success,
            "result_message": log.result_message,
            "agent_id": log.claimed_by,
            "completed_at": log.created_at,
        });

        let event_type = if log.success {
            EVENT_WORKORDER_COMPLETED
        } else {
            EVENT_WORKORDER_FAILED
        };

        event_bus::emit_event(self.dal, &BrokkrEvent::new(event_type, event_data));
    }

    /// Completes a work order with failure.
    ///
    /// If `retryable` is true and retries remain, the work order is marked as RETRY_PENDING.
    /// If `retryable` is false or max retries exceeded, the work order is moved to the log.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order that failed.
    /// * `error_message` - The error message describing the failure.
    /// * `retryable` - Whether this failure type can be retried.
    ///
    /// # Returns
    ///
    /// Returns either:
    /// - `Ok(None)` if the work order was scheduled for retry
    /// - `Ok(Some(WorkOrderLog))` if not retryable or max retries exceeded
    /// - `Err` on database error
    pub fn complete_failure(
        &self,
        work_order_id: Uuid,
        error_message: String,
        retryable: bool,
    ) -> Result<Option<WorkOrderLog>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let result: Option<WorkOrderLog> = conn.transaction(|conn| {
            // Get the work order
            let work_order: WorkOrder = work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .first(conn)?;

            let new_retry_count = work_order.retry_count + 1;

            // Move to log immediately if not retryable, or if max retries exceeded
            if !retryable || new_retry_count > work_order.max_retries {
                // Move to log
                let log_entry =
                    NewWorkOrderLog::from_work_order(&work_order, false, Some(error_message));
                let log_result: WorkOrderLog = diesel::insert_into(work_order_log::table)
                    .values(&log_entry)
                    .get_result(conn)?;

                // Delete work order
                diesel::delete(work_orders::table.filter(work_orders::id.eq(work_order_id)))
                    .execute(conn)?;

                Ok::<Option<WorkOrderLog>, diesel::result::Error>(Some(log_result))
            } else {
                // Schedule retry with exponential backoff
                let backoff_multiplier = 2_i64.pow(new_retry_count as u32);
                let backoff_duration =
                    Duration::seconds(work_order.backoff_seconds as i64 * backoff_multiplier);
                let now = Utc::now();
                let next_retry = now + backoff_duration;

                diesel::update(work_orders::table.filter(work_orders::id.eq(work_order_id)))
                    .set((
                        work_orders::status.eq(WORK_ORDER_STATUS_RETRY_PENDING),
                        work_orders::retry_count.eq(new_retry_count),
                        work_orders::next_retry_after.eq(next_retry),
                        work_orders::claimed_by.eq(None::<Uuid>),
                        work_orders::claimed_at.eq(None::<chrono::DateTime<Utc>>),
                        work_orders::last_error.eq(&error_message),
                        work_orders::last_error_at.eq(now),
                    ))
                    .execute(conn)?;

                Ok(None)
            }
        })?;

        // Emit completion event if work order was moved to log
        if let Some(ref log) = result {
            self.emit_completion_event(log);
        }

        Ok(result)
    }

    // =========================================================================
    // RETRY AND STALE CLAIM OPERATIONS
    // =========================================================================

    /// Resets RETRY_PENDING work orders to PENDING if their backoff period has elapsed.
    ///
    /// This should be called periodically by a background job.
    ///
    /// # Returns
    ///
    /// Returns the number of work orders reset on success, or a diesel::result::Error on failure.
    pub fn process_retry_pending(&self) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let now = Utc::now();

        diesel::update(
            work_orders::table
                .filter(work_orders::status.eq(WORK_ORDER_STATUS_RETRY_PENDING))
                .filter(work_orders::next_retry_after.le(now)),
        )
        .set((
            work_orders::status.eq(WORK_ORDER_STATUS_PENDING),
            work_orders::next_retry_after.eq(None::<chrono::DateTime<Utc>>),
        ))
        .execute(conn)
    }

    /// Resets stale claimed work orders to PENDING.
    ///
    /// A claim is stale if: claimed_at + claim_timeout_seconds < NOW()
    ///
    /// This should be called periodically by a background job.
    ///
    /// # Returns
    ///
    /// Returns the number of work orders reset on success, or a diesel::result::Error on failure.
    pub fn process_stale_claims(&self) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        let now = Utc::now();

        // We need to use raw SQL for this because Diesel doesn't support
        // arithmetic with columns in WHERE clauses easily
        diesel::sql_query(
            "UPDATE work_orders
             SET status = 'PENDING', claimed_by = NULL, claimed_at = NULL, updated_at = NOW()
             WHERE status = 'CLAIMED'
             AND claimed_at + (claim_timeout_seconds || ' seconds')::INTERVAL < $1",
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(conn)
    }

    // =========================================================================
    // WORK ORDER TARGETS OPERATIONS
    // =========================================================================

    /// Adds an agent as a target for a work order.
    ///
    /// # Arguments
    ///
    /// * `new_target` - The new target to add.
    ///
    /// # Returns
    ///
    /// Returns the created WorkOrderTarget on success, or a diesel::result::Error on failure.
    pub fn add_target(
        &self,
        new_target: &NewWorkOrderTarget,
    ) -> Result<WorkOrderTarget, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::insert_into(work_order_targets::table)
            .values(new_target)
            .get_result(conn)
    }

    /// Adds multiple agents as targets for a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `agent_ids` - A slice of agent UUIDs to add as targets.
    ///
    /// # Returns
    ///
    /// Returns the number of targets added on success, or a diesel::result::Error on failure.
    pub fn add_targets(
        &self,
        work_order_id: Uuid,
        agent_ids: &[Uuid],
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let targets: Vec<NewWorkOrderTarget> = agent_ids
            .iter()
            .filter_map(|&agent_id| NewWorkOrderTarget::new(work_order_id, agent_id).ok())
            .collect();

        diesel::insert_into(work_order_targets::table)
            .values(&targets)
            .execute(conn)
    }

    /// Lists all targets for a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    ///
    /// # Returns
    ///
    /// Returns a Vec of WorkOrderTargets on success, or a diesel::result::Error on failure.
    pub fn list_targets(
        &self,
        work_order_id: Uuid,
    ) -> Result<Vec<WorkOrderTarget>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_order_targets::table
            .filter(work_order_targets::work_order_id.eq(work_order_id))
            .load::<WorkOrderTarget>(conn)
    }

    /// Removes a target from a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `agent_id` - The UUID of the agent to remove.
    ///
    /// # Returns
    ///
    /// Returns the number of targets removed on success, or a diesel::result::Error on failure.
    pub fn remove_target(
        &self,
        work_order_id: Uuid,
        agent_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(
            work_order_targets::table
                .filter(work_order_targets::work_order_id.eq(work_order_id))
                .filter(work_order_targets::agent_id.eq(agent_id)),
        )
        .execute(conn)
    }

    // =========================================================================
    // WORK ORDER LOG OPERATIONS
    // =========================================================================

    /// Retrieves a work order log entry by its UUID.
    ///
    /// # Arguments
    ///
    /// * `log_id` - The UUID of the log entry to retrieve.
    ///
    /// # Returns
    ///
    /// Returns an Option<WorkOrderLog> if found, or a diesel::result::Error on failure.
    pub fn get_log(&self, log_id: Uuid) -> Result<Option<WorkOrderLog>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_order_log::table
            .filter(work_order_log::id.eq(log_id))
            .first(conn)
            .optional()
    }

    /// Lists work order log entries with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `work_type` - Optional work type filter.
    /// * `success` - Optional success status filter.
    /// * `agent_id` - Optional agent ID filter.
    /// * `limit` - Optional limit on number of results.
    ///
    /// # Returns
    ///
    /// Returns a Vec of WorkOrderLog entries on success, or a diesel::result::Error on failure.
    pub fn list_log(
        &self,
        work_type: Option<&str>,
        success: Option<bool>,
        agent_id: Option<Uuid>,
        limit: Option<i64>,
    ) -> Result<Vec<WorkOrderLog>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let mut query = work_order_log::table.into_boxed();

        if let Some(wt) = work_type {
            query = query.filter(work_order_log::work_type.eq(wt));
        }

        if let Some(s) = success {
            query = query.filter(work_order_log::success.eq(s));
        }

        if let Some(aid) = agent_id {
            query = query.filter(work_order_log::claimed_by.eq(aid));
        }

        query = query.order(work_order_log::completed_at.desc());

        if let Some(l) = limit {
            query = query.limit(l);
        }

        query.load::<WorkOrderLog>(conn)
    }

    /// Lists the log entries of the work orders that a generator owned, with
    /// the filters of [`Self::list_log`]. The owner is part of the query
    /// (BROKKR-A-0013).
    pub fn list_log_owned(
        &self,
        owner_generator_id: Uuid,
        work_type: Option<&str>,
        success: Option<bool>,
        agent_id: Option<Uuid>,
        limit: Option<i64>,
    ) -> Result<Vec<WorkOrderLog>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let mut query = work_order_log::table
            .filter(work_order_log::owner_generator_id.eq(owner_generator_id))
            .into_boxed();

        if let Some(wt) = work_type {
            query = query.filter(work_order_log::work_type.eq(wt));
        }

        if let Some(s) = success {
            query = query.filter(work_order_log::success.eq(s));
        }

        if let Some(aid) = agent_id {
            query = query.filter(work_order_log::claimed_by.eq(aid));
        }

        query = query.order(work_order_log::completed_at.desc());

        if let Some(l) = limit {
            query = query.limit(l);
        }

        query.load::<WorkOrderLog>(conn)
    }

    // =========================================================================
    // WORK ORDER LABELS OPERATIONS
    // =========================================================================

    /// Adds a label to a work order.
    ///
    /// # Arguments
    ///
    /// * `new_label` - The new label to add.
    ///
    /// # Returns
    ///
    /// Returns the created WorkOrderLabel on success, or a diesel::result::Error on failure.
    pub fn add_label(
        &self,
        new_label: &NewWorkOrderLabel,
    ) -> Result<WorkOrderLabel, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::insert_into(work_order_labels::table)
            .values(new_label)
            .get_result(conn)
    }

    /// Adds multiple labels to a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `labels` - A slice of label strings to add.
    ///
    /// # Returns
    ///
    /// Returns the number of labels added on success, or a diesel::result::Error on failure.
    pub fn add_labels(
        &self,
        work_order_id: Uuid,
        labels: &[String],
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let new_labels: Vec<NewWorkOrderLabel> = labels
            .iter()
            .filter_map(|label| NewWorkOrderLabel::new(work_order_id, label.clone()).ok())
            .collect();

        diesel::insert_into(work_order_labels::table)
            .values(&new_labels)
            .execute(conn)
    }

    /// Lists all labels for a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    ///
    /// # Returns
    ///
    /// Returns a Vec of WorkOrderLabels on success, or a diesel::result::Error on failure.
    pub fn list_labels(
        &self,
        work_order_id: Uuid,
    ) -> Result<Vec<WorkOrderLabel>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_order_labels::table
            .filter(work_order_labels::work_order_id.eq(work_order_id))
            .load::<WorkOrderLabel>(conn)
    }

    /// Removes a label from a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `label` - The label to remove.
    ///
    /// # Returns
    ///
    /// Returns the number of labels removed on success, or a diesel::result::Error on failure.
    pub fn remove_label(
        &self,
        work_order_id: Uuid,
        label: &str,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(
            work_order_labels::table
                .filter(work_order_labels::work_order_id.eq(work_order_id))
                .filter(work_order_labels::label.eq(label)),
        )
        .execute(conn)
    }

    // =========================================================================
    // WORK ORDER ANNOTATIONS OPERATIONS
    // =========================================================================

    /// Adds an annotation to a work order.
    ///
    /// # Arguments
    ///
    /// * `new_annotation` - The new annotation to add.
    ///
    /// # Returns
    ///
    /// Returns the created WorkOrderAnnotation on success, or a diesel::result::Error on failure.
    pub fn add_annotation(
        &self,
        new_annotation: &NewWorkOrderAnnotation,
    ) -> Result<WorkOrderAnnotation, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::insert_into(work_order_annotations::table)
            .values(new_annotation)
            .get_result(conn)
    }

    /// Adds multiple annotations to a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `annotations` - A HashMap of key-value pairs to add.
    ///
    /// # Returns
    ///
    /// Returns the number of annotations added on success, or a diesel::result::Error on failure.
    pub fn add_annotations(
        &self,
        work_order_id: Uuid,
        annotations: &std::collections::HashMap<String, String>,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;

        let new_annotations: Vec<NewWorkOrderAnnotation> = annotations
            .iter()
            .filter_map(|(key, value)| {
                NewWorkOrderAnnotation::new(work_order_id, key.clone(), value.clone()).ok()
            })
            .collect();

        diesel::insert_into(work_order_annotations::table)
            .values(&new_annotations)
            .execute(conn)
    }

    /// Lists all annotations for a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    ///
    /// # Returns
    ///
    /// Returns a Vec of WorkOrderAnnotations on success, or a diesel::result::Error on failure.
    pub fn list_annotations(
        &self,
        work_order_id: Uuid,
    ) -> Result<Vec<WorkOrderAnnotation>, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        work_order_annotations::table
            .filter(work_order_annotations::work_order_id.eq(work_order_id))
            .load::<WorkOrderAnnotation>(conn)
    }

    /// Removes an annotation from a work order.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order.
    /// * `key` - The annotation key.
    /// * `value` - The annotation value.
    ///
    /// # Returns
    ///
    /// Returns the number of annotations removed on success, or a diesel::result::Error on failure.
    pub fn remove_annotation(
        &self,
        work_order_id: Uuid,
        key: &str,
        value: &str,
    ) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.conn()?;
        diesel::delete(
            work_order_annotations::table
                .filter(work_order_annotations::work_order_id.eq(work_order_id))
                .filter(work_order_annotations::key.eq(key))
                .filter(work_order_annotations::value.eq(value)),
        )
        .execute(conn)
    }
}
