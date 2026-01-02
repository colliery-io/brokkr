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
        event_types: vec![Some("deployment.applied".to_string()), Some("deployment.failed".to_string())],
        filters: None,
        target_labels: None, // NULL = broker delivers
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test-user".to_string()),
    }
}

fn create_test_subscription_with_labels(name: &str, labels: Vec<String>) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.*".to_string())],
        filters: None,
        target_labels: Some(labels.into_iter().map(Some).collect()),
        enabled: true,
        max_retries: 3,
        timeout_seconds: 30,
        created_by: Some("test-user".to_string()),
    }
}

fn create_test_event() -> BrokkrEvent {
    BrokkrEvent::new(
        "deployment.applied",
        serde_json::json!({
            "deployment_object_id": Uuid::new_v4(),
            "agent_id": Uuid::new_v4(),
            "status": "applied"
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

    // Create a delivery (no target_labels = broker delivery)
    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None)
        .expect("Failed to create NewWebhookDelivery");

    let delivery = fixture
        .dal
        .webhook_deliveries()
        .create(&new_delivery)
        .expect("Failed to create delivery");

    assert_eq!(delivery.subscription_id, subscription.id);
    assert_eq!(delivery.event_type, "deployment.applied");
    assert_eq!(delivery.status, "pending");
    assert_eq!(delivery.attempts, 0);
    assert!(delivery.target_labels.is_none());
}

#[test]
fn test_create_delivery_with_target_labels() {
    let fixture = TestFixture::new();

    // Create a subscription with target labels
    let sub = create_test_subscription_with_labels("Agent Sub", vec!["env:prod".to_string()]);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create a delivery with target_labels
    let event = create_test_event();
    let target_labels = Some(vec![Some("env:prod".to_string())]);
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, target_labels).unwrap();

    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    assert!(delivery.target_labels.is_some());
    let labels = delivery.target_labels.unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], Some("env:prod".to_string()));
}

#[test]
fn test_get_delivery() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
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
fn test_claim_for_broker() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create multiple broker deliveries (no target_labels)
    for _ in 0..3 {
        let event = create_test_event();
        let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
        fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();
    }

    // Claim deliveries for broker
    let claimed = fixture
        .dal
        .webhook_deliveries()
        .claim_for_broker(10, None)
        .expect("Failed to claim deliveries");

    assert_eq!(claimed.len(), 3);
    for delivery in claimed {
        assert_eq!(delivery.status, "acquired");
        assert!(delivery.acquired_until.is_some());
        assert!(delivery.acquired_by.is_none()); // NULL = broker
    }
}

#[test]
fn test_claim_for_agent_with_matching_labels() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription_with_labels("Agent Sub", vec!["env:prod".to_string()]);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create a delivery with target_labels
    let event = create_test_event();
    let target_labels = Some(vec![Some("env:prod".to_string())]);
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, target_labels).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Create a real agent to claim deliveries
    let (agent, _) = fixture.create_test_agent_with_pak(
        "Test Agent".to_string(),
        "test-cluster".to_string(),
    );
    let agent_id = agent.id;
    let agent_labels = vec!["env:prod".to_string(), "region:us".to_string()];

    let claimed = fixture
        .dal
        .webhook_deliveries()
        .claim_for_agent(agent_id, &agent_labels, 10, None)
        .expect("Failed to claim deliveries");

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].status, "acquired");
    assert_eq!(claimed[0].acquired_by, Some(agent_id));
}

#[test]
fn test_claim_for_agent_without_matching_labels() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription_with_labels("Agent Sub", vec!["env:prod".to_string()]);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create a delivery with target_labels
    let event = create_test_event();
    let target_labels = Some(vec![Some("env:prod".to_string())]);
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, target_labels).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Create a real agent with non-matching labels
    let (agent, _) = fixture.create_test_agent_with_pak(
        "Test Agent".to_string(),
        "test-cluster".to_string(),
    );
    let agent_id = agent.id;
    let agent_labels = vec!["env:staging".to_string()]; // No match

    let claimed = fixture
        .dal
        .webhook_deliveries()
        .claim_for_agent(agent_id, &agent_labels, 10, None)
        .expect("Failed to claim deliveries");

    assert_eq!(claimed.len(), 0); // No match
}

#[test]
fn test_release_expired() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Claim with 0 TTL (immediately expired)
    let claimed = fixture
        .dal
        .webhook_deliveries()
        .claim_for_broker(1, Some(0))
        .unwrap();

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].status, "acquired");

    // Sleep briefly to ensure TTL expires
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Release expired
    let released = fixture
        .dal
        .webhook_deliveries()
        .release_expired()
        .expect("Failed to release expired");

    assert_eq!(released, 1);

    // Verify status is back to pending
    let delivery = fixture.dal.webhook_deliveries().get(claimed[0].id).unwrap().unwrap();
    assert_eq!(delivery.status, "pending");
    assert!(delivery.acquired_by.is_none());
    assert!(delivery.acquired_until.is_none());
}

#[test]
fn test_mark_success() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_success(delivery.id)
        .expect("Failed to mark as success");

    assert_eq!(updated.status, "success");
    assert_eq!(updated.attempts, 1);
    assert!(updated.last_attempt_at.is_some());
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_mark_failed_with_retry() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // First failure should schedule retry
    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_failed(delivery.id, "Connection refused", 5)
        .expect("Failed to mark as failed");

    assert_eq!(updated.status, "failed"); // Now goes to failed status
    assert_eq!(updated.attempts, 1);
    assert_eq!(updated.last_error, Some("Connection refused".to_string()));
    assert!(updated.next_retry_at.is_some()); // Retry scheduled
}

#[test]
fn test_process_retries() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Mark as failed (sets next_retry_at in the past due to low backoff)
    fixture.dal.webhook_deliveries().mark_failed(delivery.id, "Error", 5).unwrap();

    // Wait for retry time
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Process retries should move it back to pending
    let moved = fixture
        .dal
        .webhook_deliveries()
        .process_retries()
        .expect("Failed to process retries");

    assert_eq!(moved, 1);

    // Verify status is back to pending
    let updated = fixture.dal.webhook_deliveries().get(delivery.id).unwrap().unwrap();
    assert_eq!(updated.status, "pending");
}

#[test]
fn test_mark_failed_max_retries_exceeded() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Fail with max_retries = 0 to immediately mark as dead
    let updated = fixture
        .dal
        .webhook_deliveries()
        .mark_failed(delivery.id, "Connection refused", 0)
        .expect("Failed to mark as failed");

    assert_eq!(updated.status, "dead");
    assert_eq!(updated.attempts, 1);
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_list_for_subscription() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create deliveries with different statuses
    let event1 = create_test_event();
    let new_delivery1 = NewWebhookDelivery::new(subscription.id, &event1, None).unwrap();
    let delivery1 = fixture.dal.webhook_deliveries().create(&new_delivery1).unwrap();
    fixture.dal.webhook_deliveries().mark_success(delivery1.id).unwrap();

    let event2 = create_test_event();
    let new_delivery2 = NewWebhookDelivery::new(subscription.id, &event2, None).unwrap();
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
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
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
fn test_claim_pagination() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create 5 deliveries
    for _ in 0..5 {
        let event = create_test_event();
        let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
        fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();
    }

    // Claim only 2
    let first_batch = fixture
        .dal
        .webhook_deliveries()
        .claim_for_broker(2, None)
        .expect("Failed to claim deliveries");

    assert_eq!(first_batch.len(), 2);

    // Remaining 3 are still pending
    let remaining = fixture
        .dal
        .webhook_deliveries()
        .claim_for_broker(100, None)
        .expect("Failed to claim remaining");

    assert_eq!(remaining.len(), 3);
}

#[test]
fn test_retry_failed_delivery() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Mark as dead
    fixture.dal.webhook_deliveries().mark_failed(delivery.id, "Error", 0).unwrap();

    // Retry should reset to pending
    let retried = fixture
        .dal
        .webhook_deliveries()
        .retry(delivery.id)
        .expect("Failed to retry")
        .expect("Delivery not found");

    assert_eq!(retried.status, "pending");
    assert!(retried.completed_at.is_none());
}

#[test]
fn test_get_stats() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create deliveries in different states
    for _ in 0..2 {
        let event = create_test_event();
        let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
        fixture.dal.webhook_deliveries().create(&new_delivery).unwrap(); // pending
    }

    let event_success = create_test_event();
    let delivery_success = NewWebhookDelivery::new(subscription.id, &event_success, None).unwrap();
    let d = fixture.dal.webhook_deliveries().create(&delivery_success).unwrap();
    fixture.dal.webhook_deliveries().mark_success(d.id).unwrap();

    let event_dead = create_test_event();
    let delivery_dead = NewWebhookDelivery::new(subscription.id, &event_dead, None).unwrap();
    let d2 = fixture.dal.webhook_deliveries().create(&delivery_dead).unwrap();
    fixture.dal.webhook_deliveries().mark_failed(d2.id, "Error", 0).unwrap();

    let stats = fixture
        .dal
        .webhook_deliveries()
        .get_stats(subscription.id)
        .expect("Failed to get stats");

    assert_eq!(stats.pending, 2);
    assert_eq!(stats.success, 1);
    assert_eq!(stats.dead, 1);
}

// =============================================================================
// Exponential Backoff Tests
// =============================================================================

#[test]
fn test_exponential_backoff_timing() {
    use chrono::{Duration, Utc};

    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    let delivery = fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // First failure: next_retry_at should be now + 2^1 = 2 seconds
    let before = Utc::now();
    let after_fail1 = fixture.dal.webhook_deliveries()
        .mark_failed(delivery.id, "Error 1", 10)
        .unwrap();

    assert_eq!(after_fail1.attempts, 1);
    let next_retry1 = after_fail1.next_retry_at.expect("next_retry_at should be set");
    let expected_backoff1 = Duration::seconds(2); // 2^1
    assert!(next_retry1 >= before + expected_backoff1);
    assert!(next_retry1 <= before + expected_backoff1 + Duration::seconds(2)); // Allow 2s tolerance

    // Wait for retry time and process
    std::thread::sleep(std::time::Duration::from_secs(3));
    fixture.dal.webhook_deliveries().process_retries().unwrap();

    // Claim it again for second attempt
    let claimed = fixture.dal.webhook_deliveries().claim_for_broker(1, None).unwrap();
    assert_eq!(claimed.len(), 1);

    // Second failure: next_retry_at should be now + 2^2 = 4 seconds
    let before2 = Utc::now();
    let after_fail2 = fixture.dal.webhook_deliveries()
        .mark_failed(delivery.id, "Error 2", 10)
        .unwrap();

    assert_eq!(after_fail2.attempts, 2);
    let next_retry2 = after_fail2.next_retry_at.expect("next_retry_at should be set");
    let expected_backoff2 = Duration::seconds(4); // 2^2
    assert!(next_retry2 >= before2 + expected_backoff2);
    assert!(next_retry2 <= before2 + expected_backoff2 + Duration::seconds(2)); // Allow 2s tolerance
}

// =============================================================================
// Multi-Label Matching Tests
// =============================================================================

#[test]
fn test_claim_requires_all_labels() {
    let fixture = TestFixture::new();

    // Create subscription requiring BOTH labels
    let sub = create_test_subscription_with_labels(
        "Multi-Label Sub",
        vec!["env:prod".to_string(), "region:us".to_string()],
    );
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create delivery with both target labels
    let event = create_test_event();
    let target_labels = Some(vec![
        Some("env:prod".to_string()),
        Some("region:us".to_string()),
    ]);
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, target_labels).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Create agent with only ONE matching label
    let (agent, _) = fixture.create_test_agent_with_pak(
        "Partial Agent".to_string(),
        "test-cluster".to_string(),
    );
    let agent_labels = vec!["env:prod".to_string()]; // Missing region:us

    // Agent should NOT be able to claim (missing region:us)
    let claimed = fixture.dal.webhook_deliveries()
        .claim_for_agent(agent.id, &agent_labels, 10, None)
        .unwrap();

    assert_eq!(claimed.len(), 0, "Agent with partial labels should not claim delivery");

    // Create agent with BOTH labels
    let (agent2, _) = fixture.create_test_agent_with_pak(
        "Full Agent".to_string(),
        "test-cluster-2".to_string(),
    );
    let agent2_labels = vec!["env:prod".to_string(), "region:us".to_string()];

    // This agent SHOULD be able to claim
    let claimed2 = fixture.dal.webhook_deliveries()
        .claim_for_agent(agent2.id, &agent2_labels, 10, None)
        .unwrap();

    assert_eq!(claimed2.len(), 1, "Agent with all labels should claim delivery");
}

#[test]
fn test_empty_target_labels_matches_broker() {
    let fixture = TestFixture::new();

    // Create subscription with empty target_labels array (should behave like NULL = broker)
    let sub = NewWebhookSubscription {
        name: "Empty Labels Sub".to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some("deployment.applied".to_string())],
        filters: None,
        target_labels: Some(vec![]), // Empty array
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    };
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create delivery with empty target_labels
    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, Some(vec![])).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Broker should be able to claim it (empty = broker delivery)
    let claimed = fixture.dal.webhook_deliveries()
        .claim_for_broker(10, None)
        .unwrap();

    assert_eq!(claimed.len(), 1, "Broker should claim delivery with empty target_labels");
}

// =============================================================================
// TTL Expiration Edge Cases
// =============================================================================

#[test]
fn test_valid_acquired_until_stays_acquired() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription("Test Sub");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, None).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // Claim with long TTL (300 seconds)
    let claimed = fixture.dal.webhook_deliveries()
        .claim_for_broker(1, Some(300))
        .unwrap();

    assert_eq!(claimed.len(), 1);
    let delivery_id = claimed[0].id;

    // Try to release expired - should release 0 since TTL not expired
    let released = fixture.dal.webhook_deliveries()
        .release_expired()
        .unwrap();

    assert_eq!(released, 0, "Should not release delivery with valid TTL");

    // Verify still acquired
    let delivery = fixture.dal.webhook_deliveries().get(delivery_id).unwrap().unwrap();
    assert_eq!(delivery.status, "acquired");
}

#[test]
fn test_released_delivery_claimable_by_different_agent() {
    let fixture = TestFixture::new();

    let sub = create_test_subscription_with_labels("Agent Sub", vec!["env:prod".to_string()]);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    let event = create_test_event();
    let target_labels = Some(vec![Some("env:prod".to_string())]);
    let new_delivery = NewWebhookDelivery::new(subscription.id, &event, target_labels).unwrap();
    fixture.dal.webhook_deliveries().create(&new_delivery).unwrap();

    // First agent claims with 0 TTL
    let (agent1, _) = fixture.create_test_agent_with_pak("Agent 1".to_string(), "cluster-1".to_string());
    let agent1_labels = vec!["env:prod".to_string()];
    let claimed1 = fixture.dal.webhook_deliveries()
        .claim_for_agent(agent1.id, &agent1_labels, 1, Some(0))
        .unwrap();
    assert_eq!(claimed1.len(), 1);
    let delivery_id = claimed1[0].id;

    // Wait for TTL to expire
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Release expired
    fixture.dal.webhook_deliveries().release_expired().unwrap();

    // Second agent should be able to claim
    let (agent2, _) = fixture.create_test_agent_with_pak("Agent 2".to_string(), "cluster-2".to_string());
    let agent2_labels = vec!["env:prod".to_string()];
    let claimed2 = fixture.dal.webhook_deliveries()
        .claim_for_agent(agent2.id, &agent2_labels, 1, None)
        .unwrap();

    assert_eq!(claimed2.len(), 1, "Second agent should claim released delivery");
    assert_eq!(claimed2[0].id, delivery_id);
    assert_eq!(claimed2[0].acquired_by, Some(agent2.id));
}
