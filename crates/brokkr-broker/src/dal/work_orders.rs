/*
 * Copyright (c) 2025 Dylan Storey
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
use brokkr_models::models::webhooks::{BrokkrEvent, EVENT_WORKORDER_COMPLETED};
use brokkr_models::models::work_order_annotations::{NewWorkOrderAnnotation, WorkOrderAnnotation};
use brokkr_models::models::work_order_labels::{NewWorkOrderLabel, WorkOrderLabel};
use brokkr_models::models::work_orders::{
    NewWorkOrder, NewWorkOrderLog, NewWorkOrderTarget, WorkOrder, WorkOrderLog, WorkOrderTarget,
    WORK_ORDER_STATUS_CLAIMED, WORK_ORDER_STATUS_PENDING, WORK_ORDER_STATUS_RETRY_PENDING,
};
use brokkr_models::schema::{
    agent_annotations, agent_labels, work_order_annotations, work_order_labels, work_order_log,
    work_order_targets, work_orders,
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
    pub fn create(&self, new_work_order: &NewWorkOrder) -> Result<WorkOrder, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(work_orders::table)
            .values(new_work_order)
            .get_result(conn)
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(work_orders::table.filter(work_orders::id.eq(work_order_id))).execute(conn)
    }

    // =========================================================================
    // CLAIM OPERATIONS
    // =========================================================================

    /// Lists pending work orders that are claimable by a specific agent.
    ///
    /// A work order is claimable if:
    /// - Status is PENDING
    /// - At least one of the following conditions is met (OR logic):
    ///   - Agent is in the work_order_targets (hard targets)
    ///   - Agent has a label matching any of the work order's labels
    ///   - Agent has an annotation matching any of the work order's annotations
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Get agent's labels
        let agent_label_list: Vec<String> = agent_labels::table
            .filter(agent_labels::agent_id.eq(agent_id))
            .select(agent_labels::label)
            .load::<String>(conn)?;

        // Get agent's annotations
        let agent_annotation_list: Vec<(String, String)> = agent_annotations::table
            .filter(agent_annotations::agent_id.eq(agent_id))
            .select((agent_annotations::key, agent_annotations::value))
            .load::<(String, String)>(conn)?;

        let mut matching_work_order_ids: HashSet<Uuid> = HashSet::new();

        // 1. Get work orders where agent is a hard target
        let hard_target_ids: Vec<Uuid> = work_order_targets::table
            .inner_join(work_orders::table)
            .filter(work_order_targets::agent_id.eq(agent_id))
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .select(work_orders::id)
            .load::<Uuid>(conn)?;

        matching_work_order_ids.extend(hard_target_ids);

        // 2. Get work orders with labels matching agent's labels (OR logic)
        if !agent_label_list.is_empty() {
            let label_matched_ids: Vec<Uuid> = work_order_labels::table
                .inner_join(work_orders::table)
                .filter(work_order_labels::label.eq_any(&agent_label_list))
                .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
                .select(work_orders::id)
                .load::<Uuid>(conn)?;

            matching_work_order_ids.extend(label_matched_ids);
        }

        // 3. Get work orders with annotations matching agent's annotations (OR logic)
        if !agent_annotation_list.is_empty() {
            for (key, value) in &agent_annotation_list {
                let annotation_matched_ids: Vec<Uuid> = work_order_annotations::table
                    .inner_join(work_orders::table)
                    .filter(work_order_annotations::key.eq(key))
                    .filter(work_order_annotations::value.eq(value))
                    .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
                    .select(work_orders::id)
                    .load::<Uuid>(conn)?;

                matching_work_order_ids.extend(annotation_matched_ids);
            }
        }

        if matching_work_order_ids.is_empty() {
            return Ok(vec![]);
        }

        // Load and filter the matching work orders
        let ids: Vec<Uuid> = matching_work_order_ids.into_iter().collect();

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

    /// Atomically claims a work order for an agent.
    ///
    /// This operation will only succeed if:
    /// - The work order exists
    /// - The work order status is PENDING
    /// - At least one of the following conditions is met (OR logic):
    ///   - Agent is in the work_order_targets (hard targets)
    ///   - Agent has a label matching any of the work order's labels
    ///   - Agent has an annotation matching any of the work order's annotations
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Check if agent is authorized via any targeting mechanism (OR logic)
        let is_authorized = self.is_agent_authorized_for_work_order(conn, work_order_id, agent_id)?;

        if !is_authorized {
            return Err(diesel::result::Error::NotFound);
        }

        // Atomically update the work order status to CLAIMED
        let now = Utc::now();
        diesel::update(
            work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING)),
        )
        .set((
            work_orders::status.eq(WORK_ORDER_STATUS_CLAIMED),
            work_orders::claimed_by.eq(agent_id),
            work_orders::claimed_at.eq(now),
        ))
        .get_result(conn)
    }

    /// Checks if an agent is authorized to claim a work order using any targeting mechanism.
    ///
    /// Returns true if the agent matches via hard targets, labels, or annotations.
    fn is_agent_authorized_for_work_order(
        &self,
        conn: &mut diesel::pg::PgConnection,
        work_order_id: Uuid,
        agent_id: Uuid,
    ) -> Result<bool, diesel::result::Error> {
        // 1. Check hard targets
        let target_exists: bool = work_order_targets::table
            .filter(work_order_targets::work_order_id.eq(work_order_id))
            .filter(work_order_targets::agent_id.eq(agent_id))
            .select(diesel::dsl::count_star().gt(0))
            .first(conn)?;

        if target_exists {
            return Ok(true);
        }

        // 2. Check label matching
        // Get the work order's labels
        let work_order_label_list: Vec<String> = work_order_labels::table
            .filter(work_order_labels::work_order_id.eq(work_order_id))
            .select(work_order_labels::label)
            .load::<String>(conn)?;

        if !work_order_label_list.is_empty() {
            // Check if agent has any of these labels
            let agent_has_label: i64 = agent_labels::table
                .filter(agent_labels::agent_id.eq(agent_id))
                .filter(agent_labels::label.eq_any(&work_order_label_list))
                .select(diesel::dsl::count_star())
                .first(conn)?;

            if agent_has_label > 0 {
                return Ok(true);
            }
        }

        // 3. Check annotation matching
        // Get the work order's annotations
        let work_order_annotation_list: Vec<(String, String)> = work_order_annotations::table
            .filter(work_order_annotations::work_order_id.eq(work_order_id))
            .select((work_order_annotations::key, work_order_annotations::value))
            .load::<(String, String)>(conn)?;

        if !work_order_annotation_list.is_empty() {
            // Check if agent has any of these key-value pairs
            for (key, value) in &work_order_annotation_list {
                let agent_has_annotation: i64 = agent_annotations::table
                    .filter(agent_annotations::agent_id.eq(agent_id))
                    .filter(agent_annotations::key.eq(key))
                    .filter(agent_annotations::value.eq(value))
                    .select(diesel::dsl::count_star())
                    .first(conn)?;

                if agent_has_annotation > 0 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
    fn emit_completion_event(&self, log: &WorkOrderLog) {
        let event_data = serde_json::json!({
            "work_order_log_id": log.id,
            "work_type": log.work_type,
            "success": log.success,
            "result_message": log.result_message,
            "agent_id": log.claimed_by,
            "completed_at": log.created_at,
        });

        event_bus::emit_event(self.dal, &BrokkrEvent::new(EVENT_WORKORDER_COMPLETED, event_data));
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
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
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(
            work_order_annotations::table
                .filter(work_order_annotations::work_order_id.eq(work_order_id))
                .filter(work_order_annotations::key.eq(key))
                .filter(work_order_annotations::value.eq(value)),
        )
        .execute(conn)
    }
}
