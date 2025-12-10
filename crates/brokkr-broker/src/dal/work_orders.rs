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
use brokkr_models::models::work_orders::{
    NewWorkOrder, NewWorkOrderLog, NewWorkOrderTarget, WorkOrder, WorkOrderLog, WorkOrderTarget,
    WORK_ORDER_STATUS_CLAIMED, WORK_ORDER_STATUS_PENDING, WORK_ORDER_STATUS_RETRY_PENDING,
};
use brokkr_models::schema::{work_order_log, work_order_targets, work_orders};
use chrono::{Duration, Utc};
use diesel::prelude::*;
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
    /// - Agent is in the work_order_targets for this work order
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

        let mut query = work_orders::table
            .inner_join(work_order_targets::table)
            .filter(work_order_targets::agent_id.eq(agent_id))
            .filter(work_orders::status.eq(WORK_ORDER_STATUS_PENDING))
            .into_boxed();

        if let Some(wt) = work_type {
            query = query.filter(work_orders::work_type.eq(wt));
        }

        query
            .select(work_orders::all_columns)
            .order(work_orders::created_at.asc())
            .load::<WorkOrder>(conn)
    }

    /// Atomically claims a work order for an agent.
    ///
    /// This operation will only succeed if:
    /// - The work order exists
    /// - The work order status is PENDING
    /// - The agent is in the work_order_targets for this work order
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

        // First verify the agent is a valid target for this work order
        let target_exists: bool = work_order_targets::table
            .filter(work_order_targets::work_order_id.eq(work_order_id))
            .filter(work_order_targets::agent_id.eq(agent_id))
            .select(diesel::dsl::count_star().gt(0))
            .first(conn)?;

        if !target_exists {
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

        conn.transaction(|conn| {
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

            Ok(log_result)
        })
    }

    /// Completes a work order with failure.
    ///
    /// If retries remain, the work order is marked as RETRY_PENDING with exponential backoff.
    /// If max retries exceeded, the work order is moved to the log.
    ///
    /// # Arguments
    ///
    /// * `work_order_id` - The UUID of the work order that failed.
    /// * `error_message` - The error message describing the failure.
    ///
    /// # Returns
    ///
    /// Returns either:
    /// - `Ok(None)` if the work order was scheduled for retry
    /// - `Ok(Some(WorkOrderLog))` if max retries exceeded and moved to log
    /// - `Err` on database error
    pub fn complete_failure(
        &self,
        work_order_id: Uuid,
        error_message: String,
    ) -> Result<Option<WorkOrderLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        conn.transaction(|conn| {
            // Get the work order
            let work_order: WorkOrder = work_orders::table
                .filter(work_orders::id.eq(work_order_id))
                .first(conn)?;

            let new_retry_count = work_order.retry_count + 1;

            if new_retry_count > work_order.max_retries {
                // Max retries exceeded - move to log
                let log_entry =
                    NewWorkOrderLog::from_work_order(&work_order, false, Some(error_message));
                let log_result: WorkOrderLog = diesel::insert_into(work_order_log::table)
                    .values(&log_entry)
                    .get_result(conn)?;

                // Delete work order
                diesel::delete(work_orders::table.filter(work_orders::id.eq(work_order_id)))
                    .execute(conn)?;

                Ok(Some(log_result))
            } else {
                // Schedule retry with exponential backoff
                let backoff_multiplier = 2_i64.pow(new_retry_count as u32);
                let backoff_duration =
                    Duration::seconds(work_order.backoff_seconds as i64 * backoff_multiplier);
                let next_retry = Utc::now() + backoff_duration;

                diesel::update(work_orders::table.filter(work_orders::id.eq(work_order_id)))
                    .set((
                        work_orders::status.eq(WORK_ORDER_STATUS_RETRY_PENDING),
                        work_orders::retry_count.eq(new_retry_count),
                        work_orders::next_retry_after.eq(next_retry),
                        work_orders::claimed_by.eq(None::<Uuid>),
                        work_orders::claimed_at.eq(None::<chrono::DateTime<Utc>>),
                    ))
                    .execute(conn)?;

                Ok(None)
            }
        })
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
}
