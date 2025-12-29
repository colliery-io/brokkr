/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for WebhookSubscription operations.
//!
//! This module provides functionality to interact with the webhook_subscriptions table.
//! It includes methods for creating, updating, deleting, and querying webhook subscriptions.

use crate::dal::DAL;
use brokkr_models::models::webhooks::{
    NewWebhookSubscription, UpdateWebhookSubscription, WebhookSubscription,
};
use brokkr_models::schema::webhook_subscriptions;
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for WebhookSubscription operations.
pub struct WebhookSubscriptionsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl WebhookSubscriptionsDAL<'_> {
    /// Creates a new webhook subscription.
    ///
    /// # Arguments
    ///
    /// * `new_subscription` - The subscription to create.
    ///
    /// # Returns
    ///
    /// Returns the created WebhookSubscription record.
    pub fn create(
        &self,
        new_subscription: &NewWebhookSubscription,
    ) -> Result<WebhookSubscription, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(webhook_subscriptions::table)
            .values(new_subscription)
            .get_result(conn)
    }

    /// Gets a webhook subscription by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription UUID.
    ///
    /// # Returns
    ///
    /// Returns the subscription if found.
    pub fn get(&self, id: Uuid) -> Result<Option<WebhookSubscription>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        webhook_subscriptions::table
            .filter(webhook_subscriptions::id.eq(id))
            .first(conn)
            .optional()
    }

    /// Lists all webhook subscriptions.
    ///
    /// # Arguments
    ///
    /// * `enabled_only` - If true, only return enabled subscriptions.
    ///
    /// # Returns
    ///
    /// Returns a list of webhook subscriptions.
    pub fn list(
        &self,
        enabled_only: bool,
    ) -> Result<Vec<WebhookSubscription>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = webhook_subscriptions::table.into_boxed();

        if enabled_only {
            query = query.filter(webhook_subscriptions::enabled.eq(true));
        }

        query
            .order(webhook_subscriptions::created_at.desc())
            .load(conn)
    }

    /// Gets all enabled subscriptions that match a given event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event type to match (e.g., "health.degraded").
    ///
    /// # Returns
    ///
    /// Returns subscriptions that should receive this event type.
    pub fn get_matching_subscriptions(
        &self,
        event_type: &str,
    ) -> Result<Vec<WebhookSubscription>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Get all enabled subscriptions
        let subscriptions: Vec<WebhookSubscription> = webhook_subscriptions::table
            .filter(webhook_subscriptions::enabled.eq(true))
            .load(conn)?;

        // Filter by event type pattern matching
        let matching = subscriptions
            .into_iter()
            .filter(|sub| {
                sub.event_types.iter().any(|pattern| {
                    if let Some(p) = pattern {
                        matches_event_pattern(p, event_type)
                    } else {
                        false
                    }
                })
            })
            .collect();

        Ok(matching)
    }

    /// Updates a webhook subscription.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription UUID.
    /// * `update` - The fields to update.
    ///
    /// # Returns
    ///
    /// Returns the updated subscription.
    pub fn update(
        &self,
        id: Uuid,
        update: &UpdateWebhookSubscription,
    ) -> Result<WebhookSubscription, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(webhook_subscriptions::table.filter(webhook_subscriptions::id.eq(id)))
            .set(update)
            .get_result(conn)
    }

    /// Deletes a webhook subscription.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription UUID.
    ///
    /// # Returns
    ///
    /// Returns the number of deleted rows.
    pub fn delete(&self, id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::delete(webhook_subscriptions::table.filter(webhook_subscriptions::id.eq(id)))
            .execute(conn)
    }

    /// Enables or disables a subscription.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription UUID.
    /// * `enabled` - Whether to enable or disable.
    ///
    /// # Returns
    ///
    /// Returns the updated subscription.
    pub fn set_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<WebhookSubscription, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(webhook_subscriptions::table.filter(webhook_subscriptions::id.eq(id)))
            .set(webhook_subscriptions::enabled.eq(enabled))
            .get_result(conn)
    }
}

/// Matches an event type against a pattern.
///
/// Patterns support:
/// - Exact match: "health.degraded" matches "health.degraded"
/// - Wildcard suffix: "health.*" matches "health.degraded", "health.failing", etc.
/// - Full wildcard: "*" matches everything
fn matches_event_pattern(pattern: &str, event_type: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.ends_with(".*") {
        let prefix = &pattern[..pattern.len() - 2];
        return event_type.starts_with(prefix) && event_type[prefix.len()..].starts_with('.');
    }

    pattern == event_type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_event_pattern_exact() {
        assert!(matches_event_pattern("health.degraded", "health.degraded"));
        assert!(!matches_event_pattern("health.degraded", "health.failing"));
    }

    #[test]
    fn test_matches_event_pattern_wildcard_suffix() {
        assert!(matches_event_pattern("health.*", "health.degraded"));
        assert!(matches_event_pattern("health.*", "health.failing"));
        assert!(matches_event_pattern("health.*", "health.recovered"));
        assert!(!matches_event_pattern("health.*", "agent.offline"));
        assert!(!matches_event_pattern("health.*", "healthcheck")); // No dot after prefix
    }

    #[test]
    fn test_matches_event_pattern_full_wildcard() {
        assert!(matches_event_pattern("*", "health.degraded"));
        assert!(matches_event_pattern("*", "agent.offline"));
        assert!(matches_event_pattern("*", "anything"));
    }
}
