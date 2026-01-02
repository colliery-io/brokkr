/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Event emission utilities for webhook notifications.
//!
//! This module provides a database-centric approach to event emission.
//! Events are directly inserted into the webhook_deliveries table for
//! matching subscriptions. No in-memory event bus is used.

use crate::dal::DAL;
use brokkr_models::models::webhooks::{BrokkrEvent, NewWebhookDelivery};
use tracing::{debug, error};

/// Emits an event by creating webhook deliveries for all matching subscriptions.
///
/// This function:
/// 1. Finds all enabled subscriptions matching the event type
/// 2. Creates a webhook_delivery record for each matching subscription
/// 3. Copies target_labels from subscription to delivery for routing
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
        // Copy target_labels from subscription to delivery
        let target_labels = subscription.target_labels.clone();

        match NewWebhookDelivery::new(subscription.id, event, target_labels) {
            Ok(new_delivery) => {
                match dal.webhook_deliveries().create(&new_delivery) {
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
                }
            }
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
    use serde_json::json;
    use brokkr_models::models::webhooks::EVENT_AGENT_REGISTERED;

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
