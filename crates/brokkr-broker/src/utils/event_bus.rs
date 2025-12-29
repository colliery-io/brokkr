/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Event Bus for Brokkr webhook notifications.
//!
//! This module provides an in-process event bus using tokio mpsc channels.
//! Events are emitted from various parts of the system and dispatched to
//! matching webhook subscriptions for delivery.

use crate::dal::DAL;
use brokkr_models::models::webhooks::{BrokkrEvent, NewWebhookDelivery};
use tracing::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Default channel buffer size for events.
const DEFAULT_CHANNEL_SIZE: usize = 1000;

/// Global event bus storage.
static EVENT_BUS: OnceCell<Arc<EventBus>> = OnceCell::new();

/// The event bus for distributing events to webhook subscribers.
#[derive(Clone)]
pub struct EventBus {
    /// Sender for emitting events.
    sender: mpsc::Sender<BrokkrEvent>,
}

impl EventBus {
    /// Creates a new event bus and starts the dispatcher.
    ///
    /// # Arguments
    /// * `dal` - The Data Access Layer for database operations.
    ///
    /// # Returns
    /// An EventBus instance that can be used to emit events.
    pub fn new(dal: DAL) -> Self {
        Self::with_capacity(dal, DEFAULT_CHANNEL_SIZE)
    }

    /// Creates a new event bus with a custom channel capacity.
    ///
    /// # Arguments
    /// * `dal` - The Data Access Layer for database operations.
    /// * `capacity` - The channel buffer size.
    ///
    /// # Returns
    /// An EventBus instance that can be used to emit events.
    pub fn with_capacity(dal: DAL, capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        // Start the dispatcher task
        start_event_dispatcher(dal, receiver);

        info!("Event bus started with capacity {}", capacity);

        Self { sender }
    }

    /// Emits an event to the bus.
    ///
    /// This is a non-blocking operation. If the channel is full, the event
    /// will be dropped and an error will be logged.
    ///
    /// # Arguments
    /// * `event` - The event to emit.
    pub fn emit(&self, event: BrokkrEvent) {
        let sender = self.sender.clone();
        let event_type = event.event_type.clone();
        let event_id = event.id;

        tokio::spawn(async move {
            match sender.send(event).await {
                Ok(_) => {
                    debug!("Event emitted: {} (id: {})", event_type, event_id);
                }
                Err(e) => {
                    error!(
                        "Failed to emit event {} (id: {}): channel full or closed - {}",
                        event_type, event_id, e
                    );
                }
            }
        });
    }

    /// Emits an event synchronously, waiting for it to be accepted.
    ///
    /// # Arguments
    /// * `event` - The event to emit.
    ///
    /// # Returns
    /// Ok if the event was accepted, Err if the channel is closed.
    pub async fn emit_async(&self, event: BrokkrEvent) -> Result<(), mpsc::error::SendError<BrokkrEvent>> {
        let event_type = event.event_type.clone();
        let event_id = event.id;

        self.sender.send(event).await.map_err(|e| {
            error!(
                "Failed to emit event {} (id: {}): {}",
                event_type, event_id, e
            );
            e
        })?;

        debug!("Event emitted (async): {} (id: {})", event_type, event_id);
        Ok(())
    }
}

/// Initializes the global event bus.
///
/// This should be called once during broker startup.
///
/// # Arguments
/// * `dal` - The Data Access Layer for database operations.
///
/// # Returns
/// Ok(()) if initialization succeeded, Err if already initialized.
pub fn init_event_bus(dal: DAL) -> Result<(), String> {
    let bus = EventBus::new(dal);
    EVENT_BUS
        .set(Arc::new(bus))
        .map_err(|_| "Event bus already initialized".to_string())
}

/// Gets the global event bus.
///
/// # Returns
/// The event bus, or None if not initialized.
pub fn get_event_bus() -> Option<Arc<EventBus>> {
    EVENT_BUS.get().cloned()
}

/// Emits an event to the global event bus.
///
/// This is a convenience function for emitting events without
/// needing to get the bus directly.
///
/// # Arguments
/// * `event` - The event to emit.
pub fn emit(event: BrokkrEvent) {
    if let Some(bus) = get_event_bus() {
        bus.emit(event);
    } else {
        warn!("Event bus not initialized, event dropped: {}", event.event_type);
    }
}

/// Starts the event dispatcher background task.
///
/// This task receives events from the channel and creates deliveries
/// for all matching webhook subscriptions.
fn start_event_dispatcher(dal: DAL, mut receiver: mpsc::Receiver<BrokkrEvent>) {
    tokio::spawn(async move {
        info!("Event dispatcher started");

        while let Some(event) = receiver.recv().await {
            dispatch_event(&dal, &event).await;
        }

        warn!("Event dispatcher stopped - channel closed");
    });
}

/// Dispatches a single event to matching subscriptions.
async fn dispatch_event(dal: &DAL, event: &BrokkrEvent) {
    // Find all subscriptions that match this event type
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
            return;
        }
    };

    if subscriptions.is_empty() {
        debug!(
            "No subscriptions match event {} (id: {})",
            event.event_type, event.id
        );
        return;
    }

    debug!(
        "Dispatching event {} (id: {}) to {} subscription(s)",
        event.event_type,
        event.id,
        subscriptions.len()
    );

    // Create a delivery for each matching subscription
    for subscription in subscriptions {
        // TODO: Apply subscription filters here
        // For now, we just check event type matching which is done in get_matching_subscriptions

        match NewWebhookDelivery::new(subscription.id, event) {
            Ok(new_delivery) => {
                match dal.webhook_deliveries().create(&new_delivery) {
                    Ok(delivery) => {
                        debug!(
                            "Created delivery {} for subscription {} (event: {})",
                            delivery.id, subscription.id, event.event_type
                        );
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_models::models::webhooks::BrokkrEvent;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_default_channel_size() {
        assert_eq!(DEFAULT_CHANNEL_SIZE, 1000);
    }

    #[test]
    fn test_get_event_bus_uninitialized() {
        // When event bus is not initialized, get_event_bus returns None
        // Note: We can't easily reset the global state, but we can verify the API
        let bus = get_event_bus();
        // In test context without init, this may or may not be None depending on test order
        // The important thing is it doesn't panic
        let _ = bus;
    }

    #[test]
    fn test_emit_without_bus_does_not_panic() {
        // Create a test event
        let event = BrokkrEvent::new("test.event", json!({"test": "data"}));

        // Calling emit when bus is not initialized should not panic
        // It should just log a warning and drop the event
        emit(event);
    }

    #[test]
    fn test_brokkr_event_creation() {
        let event = BrokkrEvent::new("stack.created", json!({"stack_id": "test-123"}));

        assert_eq!(event.event_type, "stack.created");
        assert!(event.id != Uuid::nil());
        assert!(event.timestamp.timestamp() > 0);
    }

    #[test]
    fn test_brokkr_event_with_minimal_data() {
        let event = BrokkrEvent::new("test.minimal", json!(null));

        assert_eq!(event.event_type, "test.minimal");
        assert_eq!(event.data, json!(null));
    }

    #[test]
    fn test_brokkr_event_data_serialization() {
        let data = json!({
            "agent_id": "agent-1",
            "cluster": "production",
            "metadata": {
                "nested": "value"
            }
        });

        let event = BrokkrEvent::new("agent.heartbeat", data.clone());

        // Verify data is preserved
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_brokkr_event_unique_ids() {
        let event1 = BrokkrEvent::new("test.event", json!({}));
        let event2 = BrokkrEvent::new("test.event", json!({}));

        // Each event should have a unique ID
        assert_ne!(event1.id, event2.id);
    }

    #[test]
    fn test_event_type_patterns() {
        // Test various event type patterns
        let patterns = vec![
            "agent.created",
            "agent.updated",
            "agent.deleted",
            "stack.created",
            "deployment.created",
            "work_order.completed",
        ];

        for pattern in patterns {
            let event = BrokkrEvent::new(pattern, json!({}));
            assert_eq!(event.event_type, pattern);
        }
    }
}
