/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for WebhookDelivery operations.
//!
//! This module provides functionality to interact with the webhook_deliveries table.
//! It includes methods for creating deliveries, processing pending deliveries,
//! recording attempts, and cleaning up old records.

use crate::dal::DAL;
use brokkr_models::models::webhooks::{
    NewWebhookDelivery, WebhookDelivery, DELIVERY_STATUS_DEAD, DELIVERY_STATUS_FAILED,
    DELIVERY_STATUS_PENDING, DELIVERY_STATUS_SUCCESS,
};
use brokkr_models::schema::webhook_deliveries;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

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

    /// Gets pending deliveries that are ready to be attempted.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of deliveries to return.
    ///
    /// # Returns
    ///
    /// Returns deliveries ready for attempt, ordered by next_attempt_at.
    pub fn get_pending(&self, limit: i64) -> Result<Vec<WebhookDelivery>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        webhook_deliveries::table
            .filter(webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING))
            .filter(webhook_deliveries::next_attempt_at.le(Utc::now()))
            .order(webhook_deliveries::next_attempt_at.asc())
            .limit(limit)
            .load(conn)
    }

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

    /// Records a successful delivery attempt.
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
                webhook_deliveries::next_attempt_at.eq(None::<DateTime<Utc>>),
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
                    webhook_deliveries::next_attempt_at.eq(None::<DateTime<Utc>>),
                    webhook_deliveries::completed_at.eq(now),
                    webhook_deliveries::last_error.eq(error),
                ))
                .get_result(conn)
        } else {
            // Schedule retry with exponential backoff
            let backoff_seconds = 2_i64.pow(new_attempts as u32);
            let next_attempt = now + Duration::seconds(backoff_seconds);

            diesel::update(webhook_deliveries::table.filter(webhook_deliveries::id.eq(id)))
                .set((
                    webhook_deliveries::status.eq(DELIVERY_STATUS_PENDING),
                    webhook_deliveries::attempts.eq(new_attempts),
                    webhook_deliveries::last_attempt_at.eq(now),
                    webhook_deliveries::next_attempt_at.eq(next_attempt),
                    webhook_deliveries::last_error.eq(error),
                ))
                .get_result(conn)
        }
    }

    /// Retries a failed or dead delivery.
    ///
    /// # Arguments
    ///
    /// * `id` - The delivery UUID.
    ///
    /// # Returns
    ///
    /// Returns the updated delivery, or None if not found.
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
            webhook_deliveries::next_attempt_at.eq(Utc::now()),
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
    /// Number of successful deliveries.
    pub success: i64,
    /// Number of failed deliveries (retrying).
    pub failed: i64,
    /// Number of dead deliveries (max retries exceeded).
    pub dead: i64,
}
