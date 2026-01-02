/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Webhook models for event notifications.
//!
//! This module provides models for webhook subscriptions and deliveries,
//! enabling external systems to receive notifications when events occur in Brokkr.

use crate::schema::{webhook_deliveries, webhook_subscriptions};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Constants
// =============================================================================

/// Valid delivery statuses
pub const DELIVERY_STATUS_PENDING: &str = "pending";
pub const DELIVERY_STATUS_ACQUIRED: &str = "acquired";
pub const DELIVERY_STATUS_SUCCESS: &str = "success";
pub const DELIVERY_STATUS_FAILED: &str = "failed";
pub const DELIVERY_STATUS_DEAD: &str = "dead";

pub const VALID_DELIVERY_STATUSES: &[&str] = &[
    DELIVERY_STATUS_PENDING,
    DELIVERY_STATUS_ACQUIRED,
    DELIVERY_STATUS_SUCCESS,
    DELIVERY_STATUS_FAILED,
    DELIVERY_STATUS_DEAD,
];

// =============================================================================
// Event Type Constants
// =============================================================================

// Agent events
pub const EVENT_AGENT_REGISTERED: &str = "agent.registered";
pub const EVENT_AGENT_DEREGISTERED: &str = "agent.deregistered";

// Stack events
pub const EVENT_STACK_CREATED: &str = "stack.created";
pub const EVENT_STACK_DELETED: &str = "stack.deleted";

// Deployment object events
pub const EVENT_DEPLOYMENT_CREATED: &str = "deployment.created";
pub const EVENT_DEPLOYMENT_APPLIED: &str = "deployment.applied";
pub const EVENT_DEPLOYMENT_FAILED: &str = "deployment.failed";
pub const EVENT_DEPLOYMENT_DELETED: &str = "deployment.deleted";

// Work order events
pub const EVENT_WORKORDER_CREATED: &str = "workorder.created";
pub const EVENT_WORKORDER_CLAIMED: &str = "workorder.claimed";
pub const EVENT_WORKORDER_COMPLETED: &str = "workorder.completed";
pub const EVENT_WORKORDER_FAILED: &str = "workorder.failed";

pub const VALID_EVENT_TYPES: &[&str] = &[
    // Agent
    EVENT_AGENT_REGISTERED,
    EVENT_AGENT_DEREGISTERED,
    // Stack
    EVENT_STACK_CREATED,
    EVENT_STACK_DELETED,
    // Deployment
    EVENT_DEPLOYMENT_CREATED,
    EVENT_DEPLOYMENT_APPLIED,
    EVENT_DEPLOYMENT_FAILED,
    EVENT_DEPLOYMENT_DELETED,
    // Work order
    EVENT_WORKORDER_CREATED,
    EVENT_WORKORDER_CLAIMED,
    EVENT_WORKORDER_COMPLETED,
    EVENT_WORKORDER_FAILED,
];

// =============================================================================
// Event Payload Types
// =============================================================================

/// A Brokkr event that can trigger webhook deliveries.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrokkrEvent {
    /// Unique identifier for this event (idempotency key).
    pub id: Uuid,
    /// Event type (e.g., "deployment.applied").
    pub event_type: String,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Event-specific data.
    pub data: serde_json::Value,
}

impl BrokkrEvent {
    /// Creates a new event.
    pub fn new(event_type: &str, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            timestamp: Utc::now(),
            data,
        }
    }
}

/// Filters for webhook subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct WebhookFilters {
    /// Filter by specific agent ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    /// Filter by specific stack ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_id: Option<Uuid>,
    /// Filter by labels (all must match).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<String, String>>,
}

// =============================================================================
// Webhook Subscription Models
// =============================================================================

/// A webhook subscription record from the database.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = webhook_subscriptions)]
pub struct WebhookSubscription {
    /// Unique identifier for the subscription.
    pub id: Uuid,
    /// Human-readable name for the subscription.
    pub name: String,
    /// Encrypted webhook URL.
    #[serde(skip_serializing)]
    pub url_encrypted: Vec<u8>,
    /// Encrypted Authorization header value.
    #[serde(skip_serializing)]
    pub auth_header_encrypted: Option<Vec<u8>>,
    /// Event types to subscribe to (supports wildcards like "deployment.*").
    pub event_types: Vec<Option<String>>,
    /// JSON-encoded filters.
    pub filters: Option<String>,
    /// Labels for delivery targeting (NULL = broker delivers).
    pub target_labels: Option<Vec<Option<String>>>,
    /// Whether the subscription is active.
    pub enabled: bool,
    /// Maximum delivery retry attempts.
    pub max_retries: i32,
    /// HTTP request timeout in seconds.
    pub timeout_seconds: i32,
    /// When the subscription was created.
    pub created_at: DateTime<Utc>,
    /// When the subscription was last updated.
    pub updated_at: DateTime<Utc>,
    /// Who created the subscription.
    pub created_by: Option<String>,
}

/// A new webhook subscription to be inserted.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = webhook_subscriptions)]
pub struct NewWebhookSubscription {
    /// Human-readable name.
    pub name: String,
    /// Encrypted webhook URL.
    pub url_encrypted: Vec<u8>,
    /// Encrypted Authorization header value.
    pub auth_header_encrypted: Option<Vec<u8>>,
    /// Event types to subscribe to.
    pub event_types: Vec<Option<String>>,
    /// JSON-encoded filters.
    pub filters: Option<String>,
    /// Labels for delivery targeting (NULL = broker delivers).
    pub target_labels: Option<Vec<Option<String>>>,
    /// Whether the subscription is active (defaults to true).
    pub enabled: bool,
    /// Maximum retry attempts (defaults to 5).
    pub max_retries: i32,
    /// HTTP timeout in seconds (defaults to 30).
    pub timeout_seconds: i32,
    /// Who created the subscription.
    pub created_by: Option<String>,
}

impl NewWebhookSubscription {
    /// Creates a new webhook subscription.
    ///
    /// # Arguments
    /// * `name` - Human-readable name for the subscription.
    /// * `url_encrypted` - Pre-encrypted webhook URL.
    /// * `auth_header_encrypted` - Pre-encrypted Authorization header (optional).
    /// * `event_types` - List of event types to subscribe to.
    /// * `filters` - Optional filters as WebhookFilters struct.
    /// * `target_labels` - Optional labels for delivery targeting.
    /// * `created_by` - Who is creating the subscription.
    ///
    /// # Returns
    /// A Result containing the new subscription or an error.
    pub fn new(
        name: String,
        url_encrypted: Vec<u8>,
        auth_header_encrypted: Option<Vec<u8>>,
        event_types: Vec<String>,
        filters: Option<WebhookFilters>,
        target_labels: Option<Vec<String>>,
        created_by: Option<String>,
    ) -> Result<Self, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Name cannot exceed 255 characters".to_string());
        }

        // Validate event types
        if event_types.is_empty() {
            return Err("At least one event type is required".to_string());
        }

        // Serialize filters to JSON if provided
        let filters_json = filters
            .map(|f| serde_json::to_string(&f))
            .transpose()
            .map_err(|e| format!("Failed to serialize filters: {}", e))?;

        Ok(Self {
            name,
            url_encrypted,
            auth_header_encrypted,
            event_types: event_types.into_iter().map(Some).collect(),
            filters: filters_json,
            target_labels: target_labels.map(|labels| labels.into_iter().map(Some).collect()),
            enabled: true,
            max_retries: 5,
            timeout_seconds: 30,
            created_by,
        })
    }
}

/// Changeset for updating a webhook subscription.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = webhook_subscriptions)]
pub struct UpdateWebhookSubscription {
    /// New name.
    pub name: Option<String>,
    /// New encrypted URL.
    pub url_encrypted: Option<Vec<u8>>,
    /// New encrypted auth header.
    pub auth_header_encrypted: Option<Option<Vec<u8>>>,
    /// New event types.
    pub event_types: Option<Vec<Option<String>>>,
    /// New filters.
    pub filters: Option<Option<String>>,
    /// New target labels for delivery.
    pub target_labels: Option<Option<Vec<Option<String>>>>,
    /// Enable/disable.
    pub enabled: Option<bool>,
    /// New max retries.
    pub max_retries: Option<i32>,
    /// New timeout.
    pub timeout_seconds: Option<i32>,
}

// =============================================================================
// Webhook Delivery Models
// =============================================================================

/// A webhook delivery record from the database.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = webhook_deliveries)]
pub struct WebhookDelivery {
    /// Unique identifier for the delivery.
    pub id: Uuid,
    /// The subscription this delivery belongs to.
    pub subscription_id: Uuid,
    /// The event type being delivered.
    pub event_type: String,
    /// The event ID (idempotency key).
    pub event_id: Uuid,
    /// JSON-encoded event payload.
    pub payload: String,
    /// Labels for delivery targeting (copied from subscription).
    pub target_labels: Option<Vec<Option<String>>>,
    /// Delivery status: pending, acquired, success, failed, dead.
    pub status: String,
    /// Agent ID that acquired this delivery (NULL = broker).
    pub acquired_by: Option<Uuid>,
    /// TTL for the acquisition - release if exceeded.
    pub acquired_until: Option<DateTime<Utc>>,
    /// Number of delivery attempts.
    pub attempts: i32,
    /// When the last delivery attempt was made.
    pub last_attempt_at: Option<DateTime<Utc>>,
    /// When to retry after failure.
    pub next_retry_at: Option<DateTime<Utc>>,
    /// Error message from last failed attempt.
    pub last_error: Option<String>,
    /// When the delivery was created.
    pub created_at: DateTime<Utc>,
    /// When the delivery completed (success or dead).
    pub completed_at: Option<DateTime<Utc>>,
}

/// A new webhook delivery to be inserted.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = webhook_deliveries)]
pub struct NewWebhookDelivery {
    /// The subscription to deliver to.
    pub subscription_id: Uuid,
    /// The event type.
    pub event_type: String,
    /// The event ID.
    pub event_id: Uuid,
    /// JSON-encoded payload.
    pub payload: String,
    /// Labels for delivery targeting (copied from subscription).
    pub target_labels: Option<Vec<Option<String>>>,
    /// Initial status (pending).
    pub status: String,
}

impl NewWebhookDelivery {
    /// Creates a new webhook delivery.
    ///
    /// # Arguments
    /// * `subscription_id` - The subscription to deliver to.
    /// * `event` - The event to deliver.
    /// * `target_labels` - Labels for delivery targeting (from subscription).
    ///
    /// # Returns
    /// A Result containing the new delivery or an error.
    pub fn new(
        subscription_id: Uuid,
        event: &BrokkrEvent,
        target_labels: Option<Vec<Option<String>>>,
    ) -> Result<Self, String> {
        if subscription_id.is_nil() {
            return Err("Subscription ID cannot be nil".to_string());
        }

        let payload = serde_json::to_string(event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        Ok(Self {
            subscription_id,
            event_type: event.event_type.clone(),
            event_id: event.id,
            payload,
            target_labels,
            status: DELIVERY_STATUS_PENDING.to_string(),
        })
    }
}

/// Changeset for updating a webhook delivery.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = webhook_deliveries)]
pub struct UpdateWebhookDelivery {
    /// New status.
    pub status: Option<String>,
    /// Agent that acquired this delivery.
    pub acquired_by: Option<Option<Uuid>>,
    /// TTL for the acquisition.
    pub acquired_until: Option<Option<DateTime<Utc>>>,
    /// Increment attempts.
    pub attempts: Option<i32>,
    /// When last attempted.
    pub last_attempt_at: Option<DateTime<Utc>>,
    /// When to retry.
    pub next_retry_at: Option<Option<DateTime<Utc>>>,
    /// Error message.
    pub last_error: Option<Option<String>>,
    /// When completed.
    pub completed_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brokkr_event_new() {
        let data = serde_json::json!({"agent_id": "123"});
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, data.clone());

        assert!(!event.id.is_nil());
        assert_eq!(event.event_type, EVENT_AGENT_REGISTERED);
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_new_webhook_subscription_success() {
        let result = NewWebhookSubscription::new(
            "Test Webhook".to_string(),
            vec![1, 2, 3], // Mock encrypted URL
            None,
            vec!["deployment.*".to_string()],
            None,
            None,
            Some("admin".to_string()),
        );

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.name, "Test Webhook");
        assert!(sub.enabled);
        assert_eq!(sub.max_retries, 5);
        assert!(sub.target_labels.is_none());
    }

    #[test]
    fn test_new_webhook_subscription_with_target_labels() {
        let result = NewWebhookSubscription::new(
            "Test Webhook".to_string(),
            vec![1, 2, 3],
            None,
            vec!["deployment.*".to_string()],
            None,
            Some(vec!["env:prod".to_string(), "region:us-east".to_string()]),
            Some("admin".to_string()),
        );

        assert!(result.is_ok());
        let sub = result.unwrap();
        let labels = sub.target_labels.unwrap();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn test_new_webhook_subscription_empty_name() {
        let result = NewWebhookSubscription::new(
            "".to_string(),
            vec![1, 2, 3],
            None,
            vec!["deployment.*".to_string()],
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Name cannot be empty"));
    }

    #[test]
    fn test_new_webhook_subscription_no_event_types() {
        let result = NewWebhookSubscription::new(
            "Test".to_string(),
            vec![1, 2, 3],
            None,
            vec![],
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("At least one event type"));
    }

    #[test]
    fn test_new_webhook_delivery_success() {
        let event = BrokkrEvent::new(
            EVENT_DEPLOYMENT_APPLIED,
            serde_json::json!({"deployment_object_id": "123"}),
        );
        let result = NewWebhookDelivery::new(Uuid::new_v4(), &event, None);

        assert!(result.is_ok());
        let delivery = result.unwrap();
        assert_eq!(delivery.event_type, EVENT_DEPLOYMENT_APPLIED);
        assert_eq!(delivery.event_id, event.id);
        assert_eq!(delivery.status, DELIVERY_STATUS_PENDING);
        assert!(delivery.target_labels.is_none());
    }

    #[test]
    fn test_new_webhook_delivery_with_target_labels() {
        let event = BrokkrEvent::new(
            EVENT_DEPLOYMENT_APPLIED,
            serde_json::json!({"deployment_object_id": "123"}),
        );
        let labels = Some(vec![Some("env:prod".to_string())]);
        let result = NewWebhookDelivery::new(Uuid::new_v4(), &event, labels);

        assert!(result.is_ok());
        let delivery = result.unwrap();
        assert!(delivery.target_labels.is_some());
    }

    #[test]
    fn test_new_webhook_delivery_nil_subscription() {
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, serde_json::json!({}));
        let result = NewWebhookDelivery::new(Uuid::nil(), &event, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Subscription ID cannot be nil"));
    }

    #[test]
    fn test_webhook_filters_serialization() {
        let filters = WebhookFilters {
            agent_id: Some(Uuid::new_v4()),
            stack_id: None,
            labels: Some(std::collections::HashMap::from([
                ("env".to_string(), "prod".to_string()),
            ])),
        };

        let json = serde_json::to_string(&filters).unwrap();
        let parsed: WebhookFilters = serde_json::from_str(&json).unwrap();

        assert_eq!(filters.agent_id, parsed.agent_id);
        assert_eq!(filters.labels, parsed.labels);
    }

    #[test]
    fn test_valid_event_types() {
        // Ensure all event types are present
        assert!(VALID_EVENT_TYPES.contains(&EVENT_AGENT_REGISTERED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_AGENT_DEREGISTERED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_STACK_CREATED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_STACK_DELETED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_DEPLOYMENT_CREATED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_DEPLOYMENT_APPLIED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_DEPLOYMENT_FAILED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_DEPLOYMENT_DELETED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_WORKORDER_CREATED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_WORKORDER_CLAIMED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_WORKORDER_COMPLETED));
        assert!(VALID_EVENT_TYPES.contains(&EVENT_WORKORDER_FAILED));
    }

    #[test]
    fn test_valid_delivery_statuses() {
        assert!(VALID_DELIVERY_STATUSES.contains(&DELIVERY_STATUS_PENDING));
        assert!(VALID_DELIVERY_STATUSES.contains(&DELIVERY_STATUS_ACQUIRED));
        assert!(VALID_DELIVERY_STATUSES.contains(&DELIVERY_STATUS_SUCCESS));
        assert!(VALID_DELIVERY_STATUSES.contains(&DELIVERY_STATUS_FAILED));
        assert!(VALID_DELIVERY_STATUSES.contains(&DELIVERY_STATUS_DEAD));
    }
}
