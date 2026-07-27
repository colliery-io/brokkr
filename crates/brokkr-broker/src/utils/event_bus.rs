/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Event emission utilities for webhook notifications.
//!
//! This module provides a database-centric approach to event emission.
//! Events are directly inserted into the webhook_deliveries table for
//! matching subscriptions. No in-memory event bus is used.

use crate::dal::DAL;
use brokkr_models::models::webhooks::{
    BrokkrEvent, NewWebhookDelivery, WebhookFilters, WebhookSubscription,
};
use tracing::{debug, error};

/// Decides whether a subscription's stored `filters` admit this event.
///
/// The subscription and the event payload are both in hand here, which is the
/// only place in the delivery chain where that is true: `get_matching_subscriptions`
/// sees only the event *type*, and delivery workers see only the serialized
/// payload. So filter evaluation lives here (BROKKR-T-0288).
///
/// Rules, in order:
/// * No `filters` column, or an empty/`{}` filter, admits everything. An empty
///   string is included because `update_webhook` falls back to `""` if filter
///   serialization ever fails.
/// * A parsed filter delegates to [`WebhookFilters::matches`], whose contract is
///   that **an event lacking the filtered field does not match**.
/// * Unparseable filter JSON admits *nothing*. Nothing in the API can write such
///   a row (both create and update serialize a `WebhookFilters`), so this means
///   the column was hand-edited or written by an incompatible build. A filter is
///   a narrowing statement, usually a scoping one; failing open would deliver an
///   endpoint exactly the events an operator tried to exclude, and that leak
///   cannot be undone. A stalled subscription is recoverable and this logs at
///   error level naming the subscription.
fn subscription_admits_event(subscription: &WebhookSubscription, event: &BrokkrEvent) -> bool {
    let Some(raw) = subscription.filters.as_deref() else {
        return true;
    };
    if raw.trim().is_empty() {
        return true;
    }

    match serde_json::from_str::<WebhookFilters>(raw) {
        Ok(filters) => filters.is_empty() || filters.matches(event),
        Err(e) => {
            error!(
                "Subscription {} has unparseable filters; refusing to deliver event {} (id: {}): {}",
                subscription.id, event.event_type, event.id, e
            );
            false
        }
    }
}

/// Emits an event by creating webhook deliveries for all matching subscriptions.
///
/// This function:
/// 1. Finds all enabled subscriptions matching the event type
/// 2. Discards those whose `filters` exclude this event's payload
/// 3. Creates a webhook_delivery record for each remaining subscription
/// 4. Copies target_labels from subscription to delivery for routing
///
/// # Arguments
/// * `dal` - The Data Access Layer instance.
/// * `event` - The event to emit.
///
/// # Returns
/// The number of deliveries created.
pub fn emit_event(dal: &DAL, event: &BrokkrEvent) -> usize {
    // Find all subscriptions matching this event type
    let subscriptions = match dal
        .webhook_subscriptions()
        .get_matching_subscriptions(&event.event_type)
    {
        Ok(subs) => subs,
        Err(e) => {
            error!(
                "Failed to get matching subscriptions for event {}: {:?}",
                event.event_type, e
            );
            return 0;
        }
    };

    if subscriptions.is_empty() {
        debug!(
            "No subscriptions match event {} (id: {})",
            event.event_type, event.id
        );
        return 0;
    }

    debug!(
        "Emitting event {} (id: {}) to {} subscription(s)",
        event.event_type,
        event.id,
        subscriptions.len()
    );

    let mut created_count = 0;

    // Create a delivery for each matching subscription
    for subscription in subscriptions {
        // Payload filtering (agent_id / stack_id). Note the deliberate
        // consequence: a subscription filtering on a field this event type never
        // carries receives nothing at all. `deployment.applied`/`.failed` carry
        // `agent_id` but only a `deployment_object_id`, so a `stack_id` filter
        // excludes them; resolving the owning stack would mean a DAL round trip
        // per subscription on the write path, and the fix belongs at the
        // emission site (put `stack_id` in the payload), not here.
        if !subscription_admits_event(&subscription, event) {
            debug!(
                "Subscription {} filtered out event {} (id: {})",
                subscription.id, event.event_type, event.id
            );
            continue;
        }

        // Copy target_labels from subscription to delivery
        let target_labels = subscription.target_labels.clone();

        match NewWebhookDelivery::new(subscription.id, event, target_labels) {
            Ok(new_delivery) => match dal.webhook_deliveries().create(&new_delivery) {
                Ok(delivery) => {
                    let target = if delivery.target_labels.is_some() {
                        "agent"
                    } else {
                        "broker"
                    };
                    debug!(
                        "Created delivery {} for subscription {} (event: {}, target: {})",
                        delivery.id, subscription.id, event.event_type, target
                    );
                    created_count += 1;
                }
                Err(e) => {
                    error!(
                        "Failed to create delivery for subscription {}: {:?}",
                        subscription.id, e
                    );
                }
            },
            Err(e) => {
                error!(
                    "Failed to create NewWebhookDelivery for subscription {}: {}",
                    subscription.id, e
                );
            }
        }
    }

    created_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_models::models::webhooks::{EVENT_AGENT_REGISTERED, EVENT_STACK_CREATED};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    /// Builds an in-memory subscription carrying `filters` verbatim, so the
    /// stored-JSON handling can be exercised without a database.
    fn subscription_with_filters(filters: Option<&str>) -> WebhookSubscription {
        let now = Utc::now();
        WebhookSubscription {
            id: Uuid::new_v4(),
            name: "Filter Sub".to_string(),
            url_encrypted: b"ciphertext".to_vec(),
            auth_header_encrypted: None,
            event_types: vec![Some("*".to_string())],
            filters: filters.map(str::to_string),
            target_labels: None,
            enabled: true,
            max_retries: 5,
            timeout_seconds: 30,
            created_at: now,
            updated_at: now,
            created_by: Some("test".to_string()),
        }
    }

    #[test]
    fn test_subscription_admits_event_without_filters() {
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, json!({"agent_id": Uuid::new_v4()}));

        assert!(subscription_admits_event(
            &subscription_with_filters(None),
            &event
        ));
        assert!(subscription_admits_event(
            &subscription_with_filters(Some("{}")),
            &event
        ));
        // `update_webhook` writes "" if filter serialization ever fails.
        assert!(subscription_admits_event(
            &subscription_with_filters(Some("")),
            &event
        ));
    }

    #[test]
    fn test_subscription_admits_event_agent_filter() {
        let agent_id = Uuid::new_v4();
        let subscription =
            subscription_with_filters(Some(&format!(r#"{{"agent_id":"{agent_id}"}}"#)));

        let matching = BrokkrEvent::new(EVENT_AGENT_REGISTERED, json!({"agent_id": agent_id}));
        let other = BrokkrEvent::new(EVENT_AGENT_REGISTERED, json!({"agent_id": Uuid::new_v4()}));
        let unrelated = BrokkrEvent::new(EVENT_STACK_CREATED, json!({"stack_id": Uuid::new_v4()}));

        assert!(subscription_admits_event(&subscription, &matching));
        assert!(!subscription_admits_event(&subscription, &other));
        // Event type carries no agent_id at all: excluded, by design.
        assert!(!subscription_admits_event(&subscription, &unrelated));
    }

    #[test]
    fn test_subscription_admits_event_legacy_labels_only_filter() {
        // A row written before `labels` was dropped must not stall delivery.
        let subscription = subscription_with_filters(Some(r#"{"labels":{"env":"prod"}}"#));
        let event = BrokkrEvent::new(EVENT_STACK_CREATED, json!({"stack_id": Uuid::new_v4()}));

        assert!(subscription_admits_event(&subscription, &event));
    }

    #[test]
    fn test_subscription_admits_nothing_on_unparseable_filters() {
        let subscription = subscription_with_filters(Some("{not json"));
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, json!({"agent_id": Uuid::new_v4()}));

        assert!(!subscription_admits_event(&subscription, &event));
    }

    #[test]
    fn test_brokkr_event_creation() {
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, json!({"agent_id": "test-123"}));

        assert_eq!(event.event_type, EVENT_AGENT_REGISTERED);
        assert!(!event.id.is_nil());
        assert!(event.timestamp.timestamp() > 0);
    }

    #[test]
    fn test_brokkr_event_unique_ids() {
        let event1 = BrokkrEvent::new("test.event", json!({}));
        let event2 = BrokkrEvent::new("test.event", json!({}));

        assert_ne!(event1.id, event2.id);
    }
}
