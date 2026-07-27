/*
 * Copyright (c) 2025-2026 Dylan Storey
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
///
/// A filter narrows an event-type match further, using field values carried in
/// the event payload. Evaluated by [`WebhookFilters::matches`] at emission time
/// (see `brokkr_broker::utils::event_bus::emit_event`).
///
/// Semantics (BROKKR-T-0288):
///
/// * Every filter field that is set must match: the fields are ANDed.
/// * **An event that does not carry a filtered field does not match.** A
///   subscription filtering on `stack_id` therefore receives nothing for event
///   types whose payload has no `stack_id` (`agent.*`, `workorder.*`, and
///   `deployment.applied`/`deployment.failed`, which carry only
///   `deployment_object_id`). This is deliberate: a filter is a narrowing
///   statement, and widening it to "…or the event didn't say" would deliver
///   precisely the events the operator asked to exclude.
/// * A JSON `null` counts as absent, not as a value. Several payloads emit
///   `"stack_id": null` / `"agent_id": null` when the source column is NULL
///   (e.g. `deployment.deleted` for an already-purged object,
///   `workorder.completed` for an unclaimed order).
///
/// # Read/write asymmetry
///
/// This type is the **read** path: deserialization deliberately ignores unknown
/// keys, so rows written before a field was removed still load — notably the
/// dropped `labels` filter, which was never evaluated and is superseded by
/// `target_labels` delivery routing. A legacy row must keep delivering exactly
/// as it does today; a broker that refused to load it would silently stop the
/// subscription instead. **Do not add `deny_unknown_fields` here.**
///
/// The **write** path is strict, and intentionally not symmetric: `POST` and
/// `PUT /webhooks` reject a body carrying `filters.labels` with a 422 that names
/// `target_labels` as the replacement (see
/// `brokkr_broker::api::v1::webhooks::reject_removed_write_fields`). Accepting
/// the key and dropping it would leave the caller believing their deliveries
/// were scoped when they were not — the failure mode that motivated
/// BROKKR-T-0288 in the first place. Rejection is enforced on the raw request
/// body, above this type, precisely so the tolerant deserialization below is
/// preserved.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct WebhookFilters {
    /// Filter by specific agent ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    /// Filter by specific stack ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_id: Option<Uuid>,
}

impl WebhookFilters {
    /// Returns `true` when no filter field is set, i.e. the filter matches every
    /// event of a subscribed type.
    ///
    /// A legacy row carrying only the removed `labels` key deserializes to this.
    pub fn is_empty(&self) -> bool {
        self.agent_id.is_none() && self.stack_id.is_none()
    }

    /// Evaluates this filter against an event.
    ///
    /// Returns `true` when every set field is present in `event.data` and equal
    /// to the filter's value. See the type-level docs for the absent-field rule.
    pub fn matches(&self, event: &BrokkrEvent) -> bool {
        if let Some(agent_id) = self.agent_id
            && !payload_uuid_eq(&event.data, "agent_id", agent_id)
        {
            return false;
        }
        if let Some(stack_id) = self.stack_id
            && !payload_uuid_eq(&event.data, "stack_id", stack_id)
        {
            return false;
        }
        true
    }
}

/// Returns `true` only when `data` is an object carrying `field` as a UUID
/// string equal to `expected`.
///
/// A missing key, a JSON `null`, a non-string value, or an unparseable string
/// all return `false` — the absent-field rule in [`WebhookFilters`].
fn payload_uuid_eq(data: &serde_json::Value, field: &str, expected: Uuid) -> bool {
    data.get(field)
        .and_then(|value| value.as_str())
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .is_some_and(|actual| actual == expected)
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
        assert!(
            result
                .unwrap_err()
                .contains("Subscription ID cannot be nil")
        );
    }

    #[test]
    fn test_webhook_filters_serialization() {
        let filters = WebhookFilters {
            agent_id: Some(Uuid::new_v4()),
            stack_id: Some(Uuid::new_v4()),
        };

        let json = serde_json::to_string(&filters).unwrap();
        let parsed: WebhookFilters = serde_json::from_str(&json).unwrap();

        assert_eq!(filters, parsed);
    }

    #[test]
    fn test_webhook_filters_unset_fields_are_omitted() {
        let json = serde_json::to_string(&WebhookFilters::default()).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_webhook_filters_legacy_labels_key_is_ignored() {
        // Read path only. Rows written before `labels` was dropped
        // (BROKKR-T-0288) must still deserialize; the removed key is ignored and
        // the row filters nothing. The *write* path rejects the same key with a
        // 422 — see `reject_removed_write_fields` in the broker.
        let legacy = r#"{"labels":{"env":"prod"}}"#;
        let parsed: WebhookFilters = serde_json::from_str(legacy).unwrap();

        assert_eq!(parsed, WebhookFilters::default());
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_webhook_filters_legacy_labels_alongside_agent_id() {
        let agent_id = Uuid::new_v4();
        let legacy = format!(r#"{{"agent_id":"{agent_id}","labels":{{"env":"prod"}}}}"#);
        let parsed: WebhookFilters = serde_json::from_str(&legacy).unwrap();

        assert_eq!(parsed.agent_id, Some(agent_id));
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_webhook_filters_empty_matches_everything() {
        let filters = WebhookFilters::default();

        assert!(filters.is_empty());
        assert!(filters.matches(&BrokkrEvent::new(
            EVENT_WORKORDER_CREATED,
            serde_json::json!({"work_order_id": Uuid::new_v4()}),
        )));
    }

    #[test]
    fn test_webhook_filters_agent_id_matches_and_rejects() {
        let agent_id = Uuid::new_v4();
        let filters = WebhookFilters {
            agent_id: Some(agent_id),
            stack_id: None,
        };

        let matching = BrokkrEvent::new(
            EVENT_AGENT_REGISTERED,
            serde_json::json!({"agent_id": agent_id, "name": "a"}),
        );
        let other = BrokkrEvent::new(
            EVENT_AGENT_REGISTERED,
            serde_json::json!({"agent_id": Uuid::new_v4(), "name": "b"}),
        );

        assert!(filters.matches(&matching));
        assert!(!filters.matches(&other));
    }

    #[test]
    fn test_webhook_filters_absent_field_does_not_match() {
        // The decided rule: an event that carries no such field never matches.
        // `stack.*` payloads have no agent_id; `deployment.applied` has no
        // stack_id (only deployment_object_id).
        let agent_filter = WebhookFilters {
            agent_id: Some(Uuid::new_v4()),
            stack_id: None,
        };
        let stack_event = BrokkrEvent::new(
            EVENT_STACK_CREATED,
            serde_json::json!({"stack_id": Uuid::new_v4(), "name": "s"}),
        );
        assert!(!agent_filter.matches(&stack_event));

        let stack_filter = WebhookFilters {
            agent_id: None,
            stack_id: Some(Uuid::new_v4()),
        };
        let applied_event = BrokkrEvent::new(
            EVENT_DEPLOYMENT_APPLIED,
            serde_json::json!({
                "agent_id": Uuid::new_v4(),
                "deployment_object_id": Uuid::new_v4(),
            }),
        );
        assert!(!stack_filter.matches(&applied_event));
    }

    #[test]
    fn test_webhook_filters_null_field_does_not_match() {
        // `deployment.deleted` emits `"stack_id": null` when the object row is
        // already gone; null is absent, not a wildcard.
        let filters = WebhookFilters {
            agent_id: None,
            stack_id: Some(Uuid::new_v4()),
        };
        let event = BrokkrEvent::new(
            EVENT_DEPLOYMENT_DELETED,
            serde_json::json!({"deployment_object_id": Uuid::new_v4(), "stack_id": null}),
        );

        assert!(!filters.matches(&event));
    }

    #[test]
    fn test_webhook_filters_all_set_fields_must_match() {
        let agent_id = Uuid::new_v4();
        let stack_id = Uuid::new_v4();
        let filters = WebhookFilters {
            agent_id: Some(agent_id),
            stack_id: Some(stack_id),
        };

        // Only one of the two fields agrees, so the conjunction fails.
        let event = BrokkrEvent::new(
            EVENT_DEPLOYMENT_CREATED,
            serde_json::json!({"agent_id": agent_id, "stack_id": Uuid::new_v4()}),
        );
        assert!(!filters.matches(&event));

        let both = BrokkrEvent::new(
            EVENT_DEPLOYMENT_CREATED,
            serde_json::json!({"agent_id": agent_id, "stack_id": stack_id}),
        );
        assert!(filters.matches(&both));
    }

    #[test]
    fn test_webhook_filters_non_object_payload_does_not_match() {
        let filters = WebhookFilters {
            agent_id: Some(Uuid::new_v4()),
            stack_id: None,
        };
        let event = BrokkrEvent::new(EVENT_AGENT_REGISTERED, serde_json::json!("not-an-object"));

        assert!(!filters.matches(&event));
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
