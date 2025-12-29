/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::webhooks::{BrokkrEvent, NewWebhookDelivery, NewWebhookSubscription};
use uuid::Uuid;

fn create_test_subscription(name: &str) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: Some(b"Bearer test-token".to_vec()),
        event_types: vec![Some("health.degraded".to_string()), Some("health.failing".to_string())],
        filters: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test-user".to_string()),
    }
}

fn create_test_event() -> BrokkrEvent {
    BrokkrEvent::new(
        "health.degraded",
        serde_json::json!({
            "agent_id": Uuid::new_v4(),
            "status": "degraded"
        }),
    )
}

#[test]
fn test_create_delivery() {
    let fixture = TestFixture::new();

    // Create a subscription first
    let sub = create_test_subscription("Test Sub");
    let subscription = fixture
        .dal
        .webhook_subscriptions()
        .create(&sub)
        .expect("Failed to create subscription");

    // Create a delivery
    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event)
        .expect("Failed to create NewWebhookDelivery");

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .create(&new_delivery)
        .expect("Failed to create delivery");

    assert_eq!(delivery.subscription_id, subscription.id);
    assert_eq!(delivery.event_type, "health.degraded");
    assert_eq!(delivery.status, "pending");
    assert_eq!(delivery.attempts, 0);
}

#[test]
fn test_get_delivery() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
    let created = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    let retrieved = fixture
        .dal
        .webhook_deliveries()
        .get(created.id)
        .expect("Failed to get delivery")
        .expect("Delivery not found");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.event_id, created.event_id);
}

#[test]
fn test_get_pending_deliveries() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create multiple deliveries
    for _ in 0..3 {
        let event = create_test_event();
        let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
        fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();
    }

    let pending = fixture
        .dal
        .webhook_deliveries()
        .get_pending(10)
        .expect("Failed to get pending deliveries");

    assert_eq!(pending.len(), 3);
    for delivery in pending {
        assert_eq!(delivery.status, "pending");
    }
}

#[test]
fn test_mark_success() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_success(delivery.id)
        .expect("Failed to mark as success");

    assert_eq!(updated.status, "success");
    assert_eq!(updated.attempts, 1);
    assert!(updated.last_attempt_at.is_some());
}

#[test]
fn test_mark_failed_with_retry() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // First failure should schedule retry
    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_failed(delivery.id, "Connection refused", 5)
        .expect("Failed to mark as failed");

    assert_eq!(updated.status, "pending"); // Still pending for retry
    assert_eq!(updated.attempts, 1);
    assert_eq!(updated.last_error, Some("Connection refused".to_string()));
    assert!(updated.next_attempt_at > delivery.next_attempt_at); // Backoff applied
}

#[test]
fn test_mark_failed_max_retries_exceeded() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Fail with max_retries = 0 to immediately mark as dead
    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_failed(delivery.id, "Connection refused", 0)
        .expect("Failed to mark as failed");

    assert_eq!(updated.status, "dead");
    assert_eq!(updated.attempts, 1);
}

#[test]
fn test_list_for_subscription() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create deliveries with different statuses
    let event1 = create_test_event();
    let new_delivery1 = NewWebhookDelivery::new(subscription.id, &event1).unwrap();
    let delivery1 = fixture.dal.webhook_deliveries().create(&new_delivery1).unwrap();
    fixture.dal.webhook_deliveries().mark_success(delivery1.id).unwrap();

    let event2 = create_test_event();
    let new_delivery2 = NewWebhookDelivery::new(subscription.id, &event2).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery2).unwrap(); // Still pending

    // List all deliveries for subscription
    let all_deliveries = fixture
        .dal
        .webhook_deliveries()
        .list_for_subscription(subscription.id, None, 10, 0)
        .expect("Failed to list deliveries");

    assert_eq!(all_deliveries.len(), 2);

    // List only pending
    let pending_only = fixture
        .dal
        .webhook_deliveries()
        .list_for_subscription(subscription.id, Some("pending"), 10, 0)
        .expect("Failed to list pending deliveries");

    assert_eq!(pending_only.len(), 1);

    // List only success
    let success_only = fixture
        .dal
        .webhook_deliveries()
        .list_for_subscription(subscription.id, Some("success"), 10, 0)
        .expect("Failed to list success deliveries");

    assert_eq!(success_only.len(), 1);
}

#[test]
fn test_cleanup_old_deliveries() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create and complete a delivery
    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();
    fixture.dal.webhook_deliveries().mark_success(delivery.id).unwrap();

    // Cleanup with 0 days retention (should delete everything)
    let deleted = fixture
        .dal
        .webhook_deliveries()
        .cleanup_old(0)
        .expect("Failed to cleanup");

    assert_eq!(deleted, 1);

    // Verify deleted
    let retrieved = fixture.dal.webhook_deliveries().get(delivery.id).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_pending_deliveries_pagination() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create 5 deliveries
    for _ in 0..5 {
        let event = create_test_event();
        let new_delivery = NewWebhookDelivery::new(subscription.id, &event).unwrap();
        fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();
    }

    // Get first 2
    let first_batch = fixture
        .dal
        .webhook_deliveries()
        .get_pending(2)
        .expect("Failed to get pending deliveries");

    assert_eq!(first_batch.len(), 2);

    // Get more than available
    let all = fixture
        .dal
        .webhook_deliveries()
        .get_pending(100)
        .expect("Failed to get pending deliveries");

    assert_eq!(all.len(), 5);
}
