/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for WebhookDelivery operations.
//!
//! This module provides functionality to interact with the webhook_deliveries table.
//! It includes methods for creating deliveries, claiming with TTL, processing
//! pending deliveries, recording attempts, and cleaning up old records.
//!
//! ## Delivery Targeting
//!
//! - `target_labels = NULL` or empty: Broker delivers directly
//! - `target_labels = ["env:prod", "region:us"]`: Matching agent delivers
//!
//! ## State Machine
//!
//! - pending: Ready to be claimed
//! - acquired: Claimed by broker/agent, being processed (TTL via acquired_until)
//! - success: Delivered successfully
//! - failed: Attempt failed, will retry (goes back to pending after next_retry_at)
//! - dead: Max retries exceeded

use crate::dal::DAL;
use brokkr_models::models::webhooks::{
    NewWebhookDelivery, WebhookDelivery, DELIVERY_STATUS_ACQUIRED, DELIVERY_STATUS_DEAD,
    DELIVERY_STATUS_FAILED, DELIVERY_STATUS_PENDING, DELIVERY_STATUS_SUCCESS,
};
use brokkr_models::schema::webhook_deliveries;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Default TTL for acquired deliveries (60 seconds).
const DEFAULT_CLAIM_TTL_SECONDS: i64 = 60;

/// Data Access Layer for WebhookDelivery operations.
pub struct WebhookDeliveriesDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl WebhookDeliveriesDAL<'_> {
    /// Creates a new webhook delivery.
    ///
    /// # Arguments
    ///
    /// * `new_delivery` - The delivery to create.
    ///
    /// # Returns
    ///
    /// Returns the created WebhookDelivery record.
    pub fn create(
        &self,
        new_delivery: &NewWebhookDelivery,
    ) -> Result<WebhookDelivery, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(webhook_deliveries::table)
            .values(new_delivery)
            .get_result(conn)
    }

    /// Gets a webhook delivery by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The delivery UUID.
    ///
    /// # Returns
    ///
    /// Returns the delivery if found.
    pub fn get(&self, id: Uuid) -> Result<Option<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        webhook_deliveries::table
            .filter(webhook_deliveries::id.eq(id))
            .first(conn)
            .optional()
    }

    // =========================================================================
    // Claiming Methods
    // =========================================================================

    /// Claims pending deliveries for broker processing (target_labels is NULL or empty).
    ///
    /// This atomically sets acquired_by = NULL (meaning broker) and acquired_until
    /// to current time + TTL.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of deliveries to claim.
    /// * `ttl_seconds` - TTL for the claim in seconds.
    ///
    /// # Returns
    ///
    /// Returns claimed deliveries.
    pub fn claim_for_broker(
        &self,
        limit: i64,
        ttl_seconds: Option<i64>,
    ) -> Result<Vec<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let now = Utc::now();
        let ttl = ttl_seconds.unwrap_or(DEFAULT_CLAIM_TTL_SECONDS);
        let acquired_until = now + Duration::seconds(ttl);

        // First, get IDs of pending deliveries for broker (NULL or empty target_labels)
        // Empty array is treated as equivalent to NULL per spec
        let empty_labels: Vec<Option<String>> = vec![];
        let pending_ids: Vec<Uuid> = webhook_deliveries::table
            .filter(webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING))
            .filter(
                webhook_deliveries::target_labels
                    .is_null()
                    .or(webhook_deliveries::target_labels.eq(&empty_labels)),
            )
            .order(webhook_deliveries::created_at.asc())
            .limit(limit)
            .select(webhook_deliveries::id)
            .load(conn)?;

        if pending_ids.is_empty() {
            return Ok(vec![]);
        }

        // Claim them atomically
        diesel::update(
            webhook_deliveries::table.filter(webhook_deliveries::id.eq_any(&pending_ids)),
        )
        .set((
            webhook_deliveries::status.eq(DELIVERY_STATUS_ACQUIRED),
            webhook_deliveries::acquired_by.eq(None::<Uuid>), // NULL = broker
            webhook_deliveries::acquired_until.eq(acquired_until),
        ))
        .get_results(conn)
    }

    /// Claims pending deliveries for an agent based on label matching.
    ///
    /// The agent can only claim deliveries where it has ALL the required labels.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The claiming agent's UUID.
    /// * `agent_labels` - Labels the agent has.
    /// * `limit` - Maximum number of deliveries to claim.
    /// * `ttl_seconds` - TTL for the claim in seconds.
    ///
    /// # Returns
    ///
    /// Returns claimed deliveries.
    pub fn claim_for_agent(
        &self,
        agent_id: Uuid,
        agent_labels: &[String],
        limit: i64,
        ttl_seconds: Option<i64>,
    ) -> Result<Vec<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let now = Utc::now();
        let ttl = ttl_seconds.unwrap_or(DEFAULT_CLAIM_TTL_SECONDS);
        let acquired_until = now + Duration::seconds(ttl);

        // Get all pending deliveries with target_labels
        let pending: Vec<WebhookDelivery> = webhook_deliveries::table
            .filter(webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING))
            .filter(webhook_deliveries::target_labels.is_not_null())
            .order(webhook_deliveries::created_at.asc())
            .load(conn)?;

        // Filter to deliveries where agent has ALL required labels
        let matching_ids: Vec<Uuid> = pending
            .into_iter()
            .filter(|d| {
                if let Some(ref target_labels) = d.target_labels {
                    // Agent must have all required labels
                    target_labels.iter().all(|required| {
                        if let Some(label) = required {
                            agent_labels.contains(label)
                        } else {
                            true // NULL entries in array don't count
                        }
                    })
                } else {
                    false
                }
            })
            .take(limit as usize)
            .map(|d| d.id)
            .collect();

        if matching_ids.is_empty() {
            return Ok(vec![]);
        }

        // Claim them atomically
        diesel::update(
            webhook_deliveries::table.filter(webhook_deliveries::id.eq_any(&matching_ids)),
        )
        .set((
            webhook_deliveries::status.eq(DELIVERY_STATUS_ACQUIRED),
            webhook_deliveries::acquired_by.eq(agent_id),
            webhook_deliveries::acquired_until.eq(acquired_until),
        ))
        .get_results(conn)
    }

    /// Releases expired acquired deliveries back to pending status.
    ///
    /// This should be called periodically to recover from crashed workers.
    ///
    /// # Returns
    ///
    /// Returns the number of released deliveries.
    pub fn release_expired(&self) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let now = Utc::now();

        diesel::update(
            webhook_deliveries::table
                .filter(webhook_deliveries::status.eq(DELIVERY_STATUS_ACQUIRED))
                .filter(webhook_deliveries::acquired_until.lt(now)),
        )
        .set((
            webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING),
            webhook_deliveries::acquired_by.eq(None::<Uuid>),
            webhook_deliveries::acquired_until.eq(None::<DateTime<Utc>>),
        ))
        .execute(conn)
    }

    /// Moves failed deliveries back to pending when retry time is reached.
    ///
    /// This should be called periodically to process retries.
    ///
    /// # Returns
    ///
    /// Returns the number of deliveries moved to pending.
    pub fn process_retries(&self) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let now = Utc::now();

        diesel::update(
            webhook_deliveries::table
                .filter(webhook_deliveries::status.eq(DELIVERY_STATUS_FAILED))
                .filter(webhook_deliveries::next_retry_at.le(now)),
        )
        .set((
            webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING),
            webhook_deliveries::next_retry_at.eq(None::<DateTime<Utc>>),
        ))
        .execute(conn)
    }

    // =========================================================================
    // Result Recording Methods
    // =========================================================================

    /// Records a successful delivery.
    ///
    /// # Arguments
    ///
    /// * `id` - The delivery UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated delivery.
    pub fn mark_success(&self, id: Uuid) -> Result<WebhookDelivery, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        let now = Utc::now();

        diesel::update(webhook_deliveries::table.filter(webhook_deliveries::id.eq(id)))
            .set((
                webhook_deliveries::status.eq(DELIVERY_STATUS_SUCCESS),
                webhook_deliveries::attempts.eq(webhook_deliveries::attempts + 1),
                webhook_deliveries::last_attempt_at.eq(now),
                webhook_deliveries::acquired_until.eq(None::<DateTime<Utc>>),
                webhook_deliveries::next_retry_at.eq(None::<DateTime<Utc>>),
                webhook_deliveries::completed_at.eq(now),
                webhook_deliveries::last_error.eq(None::<String>),
            ))
            .get_result(conn)
    }

    /// Records a failed delivery attempt and schedules retry if applicable.
    ///
    /// # Arguments
    ///
    /// * `id` - The delivery UUID.
    /// * `error` - The error message.
    /// * `max_retries` - Maximum retry attempts allowed.
    ///
    /// # Returns
    ///
    /// Returns the updated delivery.
    pub fn mark_failed(
        &self,
        id: Uuid,
        error: &str,
        max_retries: i32,
    ) -> Result<WebhookDelivery, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Get current delivery to check attempt count
        let delivery: WebhookDelivery = webhook_deliveries::table
            .filter(webhook_deliveries::id.eq(id))
            .first(conn)?;

        let new_attempts = delivery.attempts + 1;
        let now = Utc::now();

        if new_attempts >= max_retries {
            // Max retries exceeded, mark as dead
            diesel::update(webhook_deliveries::table.filter(webhook_deliveries::id.eq(id)))
                .set((
                    webhook_deliveries::status.eq(DELIVERY_STATUS_DEAD),
                    webhook_deliveries::attempts.eq(new_attempts),
                    webhook_deliveries::last_attempt_at.eq(now),
                    webhook_deliveries::acquired_by.eq(None::<Uuid>),
                    webhook_deliveries::acquired_until.eq(None::<DateTime<Utc>>),
                    webhook_deliveries::next_retry_at.eq(None::<DateTime<Utc>>),
                    webhook_deliveries::completed_at.eq(now),
                    webhook_deliveries::last_error.eq(error),
                ))
                .get_result(conn)
        } else {
            // Schedule retry with exponential backoff: 2^attempts seconds
            let backoff_seconds = 2_i64.pow(new_attempts as u32);
            let next_retry = now + Duration::seconds(backoff_seconds);

            diesel::update(webhook_deliveries::table.filter(webhook_deliveries::id.eq(id)))
                .set((
                    webhook_deliveries::status.eq(DELIVERY_STATUS_FAILED),
                    webhook_deliveries::attempts.eq(new_attempts),
                    webhook_deliveries::last_attempt_at.eq(now),
                    webhook_deliveries::acquired_by.eq(None::<Uuid>),
                    webhook_deliveries::acquired_until.eq(None::<DateTime<Utc>>),
                    webhook_deliveries::next_retry_at.eq(next_retry),
                    webhook_deliveries::last_error.eq(error),
                ))
                .get_result(conn)
        }
    }

    // =========================================================================
    // Query Methods
    // =========================================================================

    /// Lists deliveries for a subscription with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription UUID.
    /// * `status_filter` - Optional status to filter by.
    /// * `limit` - Maximum number of deliveries to return.
    /// * `offset` - Number of deliveries to skip for pagination.
    ///
    /// # Returns
    ///
    /// Returns recent deliveries for the subscription.
    pub fn list_for_subscription(
        &self,
        subscription_id: Uuid,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = webhook_deliveries::table
            .filter(webhook_deliveries::subscription_id.eq(subscription_id))
            .into_boxed();

        if let Some(status) = status_filter {
            query = query.filter(webhook_deliveries::status.eq(status));
        }

        query
            .order(webhook_deliveries::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
    }

    /// Retries a failed or dead delivery.
    ///
    /// # Arguments
    ///
    /// * `id` - The delivery UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated delivery, or None if not found or not retriable.
    pub fn retry(&self, id: Uuid) -> Result<Option<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Only retry failed or dead deliveries
        let result = diesel::update(
            webhook_deliveries::table
                .filter(webhook_deliveries::id.eq(id))
                .filter(
                    webhook_deliveries::status
                        .eq(DELIVERY_STATUS_FAILED)
                        .or(webhook_deliveries::status.eq(DELIVERY_STATUS_DEAD)),
                ),
        )
        .set((
            webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING),
            webhook_deliveries::acquired_by.eq(None::<Uuid>),
            webhook_deliveries::acquired_until.eq(None::<DateTime<Utc>>),
            webhook_deliveries::next_retry_at.eq(None::<DateTime<Utc>>),
            webhook_deliveries::completed_at.eq(None::<DateTime<Utc>>),
        ))
        .get_result(conn)
        .optional()?;

        Ok(result)
    }

    /// Deletes old deliveries based on retention policy.
    ///
    /// # Arguments
    ///
    /// * `retention_days` - Number of days to retain deliveries.
    ///
    /// # Returns
    ///
    /// Returns the number of deleted records.
    pub fn cleanup_old(&self, retention_days: i64) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let cutoff = Utc::now() - Duration::days(retention_days);

        diesel::delete(
            webhook_deliveries::table
                .filter(webhook_deliveries::created_at.lt(cutoff))
                .filter(
                    webhook_deliveries::status
                        .eq(DELIVERY_STATUS_SUCCESS)
                        .or(webhook_deliveries::status.eq(DELIVERY_STATUS_DEAD)),
                ),
        )
        .execute(conn)
    }

    /// Gets delivery statistics for a subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription UUID.
    ///
    /// # Returns
    ///
    /// Returns counts of deliveries by status.
    pub fn get_stats(
        &self,
        subscription_id: Uuid,
    ) -> Result<DeliveryStats, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let deliveries: Vec<WebhookDelivery> = webhook_deliveries::table
            .filter(webhook_deliveries::subscription_id.eq(subscription_id))
            .load(conn)?;

        let mut stats = DeliveryStats::default();
        for d in deliveries {
            match d.status.as_str() {
                DELIVERY_STATUS_PENDING => stats.pending += 1,
                DELIVERY_STATUS_ACQUIRED => stats.acquired += 1,
                DELIVERY_STATUS_SUCCESS => stats.success += 1,
                DELIVERY_STATUS_FAILED => stats.failed += 1,
                DELIVERY_STATUS_DEAD => stats.dead += 1,
                _ => {}
            }
        }

        Ok(stats)
    }
}

/// Statistics about webhook deliveries.
#[derive(Debug, Default, Clone)]
pub struct DeliveryStats {
    /// Number of pending deliveries.
    pub pending: i64,
    /// Number of acquired deliveries (in progress).
    pub acquired: i64,
    /// Number of successful deliveries.
    pub success: i64,
    /// Number of failed deliveries (retrying).
    pub failed: i64,
    /// Number of dead deliveries (max retries exceeded).
    pub dead: i64,
}
