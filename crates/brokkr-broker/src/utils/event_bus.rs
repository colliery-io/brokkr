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
use brokkr_utils::logging::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Default channel buffer size for events.
const DEFAULT_CHANNEL_SIZE: usize = 1000;

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

    #[test]
    fn test_default_channel_size() {
        assert_eq!(DEFAULT_CHANNEL_SIZE, 1000);
    }
}
