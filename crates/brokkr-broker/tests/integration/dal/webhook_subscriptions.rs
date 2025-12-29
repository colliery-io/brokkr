/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::webhooks::{NewWebhookSubscription, UpdateWebhookSubscription};

fn create_test_subscription(name: &str, event_types: Vec<&str>) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: Some(b"Bearer test-token".to_vec()),
        event_types: event_types.iter().map(|s| Some(s.to_string())).collect(),
        filters: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test-user".to_string()),
    }
}

#[test]
fn test_create_subscription() {
    let fixture = TestFixture::new();
    let new_sub = create_test_subscription("Test Webhook", vec!["health.degraded", "health.failing"]);

    let created = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .expect("Failed to create subscription");

    assert_eq!(created.name, "Test Webhook");
    assert!(created.enabled);
    assert_eq!(created.max_retries, 5);
    assert_eq!(created.event_types.len(), 2);
}

#[test]
fn test_get_subscription() {
    let fixture = TestFixture::new();
    let new_sub = create_test_subscription("Get Test", vec!["workorder.completed"]);

    let created = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .expect("Failed to create subscription");

    let retrieved = fixture
        .dal
        .webhook_subscriptions()
        .get(created.id)
        .expect("Failed to get subscription")
        .expect("Subscription not found");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, "Get Test");
}

#[test]
fn test_list_subscriptions() {
    let fixture = TestFixture::new();

    // Create multiple subscriptions
    let sub1 = create_test_subscription("Sub 1", vec!["health.degraded"]);
    let sub2 = create_test_subscription("Sub 2", vec!["health.failing"]);

    fixture.dal.webhook_subscriptions().create(&sub1).expect("Failed to create sub1");
    fixture.dal.webhook_subscriptions().create(&sub2).expect("Failed to create sub2");

    let all_subs = fixture
        .dal
        .webhook_subscriptions()
        .list(false)
        .expect("Failed to list subscriptions");

    assert_eq!(all_subs.len(), 2);
}

#[test]
fn test_list_enabled_only() {
    let fixture = TestFixture::new();

    let enabled_sub = create_test_subscription("Enabled Sub", vec!["health.degraded"]);
    let mut disabled_sub = create_test_subscription("Disabled Sub", vec!["health.failing"]);
    disabled_sub.enabled = false;

    fixture.dal.webhook_subscriptions().create(&enabled_sub).expect("Failed to create enabled sub");
    fixture.dal.webhook_subscriptions().create(&disabled_sub).expect("Failed to create disabled sub");

    let enabled_only = fixture
        .dal
        .webhook_subscriptions()
        .list(true)
        .expect("Failed to list enabled subscriptions");

    assert_eq!(enabled_only.len(), 1);
    assert_eq!(enabled_only[0].name, "Enabled Sub");
}

#[test]
fn test_update_subscription() {
    let fixture = TestFixture::new();
    let new_sub = create_test_subscription("Original Name", vec!["health.degraded"]);

    let created = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .expect("Failed to create subscription");

    let update_changeset = UpdateWebhookSubscription {
        name: Some("Updated Name".to_string()),
        url_encrypted: None,
        auth_header_encrypted: None,
        event_types: None,
        filters: None,
        enabled: Some(false),
        max_retries: None,
        timeout_seconds: None,
    };

    let result = fixture
        .dal
        .webhook_subscriptions()
        .update(created.id, &update_changeset)
        .expect("Failed to update subscription");

    assert_eq!(result.name, "Updated Name");
    assert!(!result.enabled);
}

#[test]
fn test_delete_subscription() {
    let fixture = TestFixture::new();
    let new_sub = create_test_subscription("To Delete", vec!["health.degraded"]);

    let created = fixture
        .dal
        .webhook_subscriptions()
        .create(&new_sub)
        .expect("Failed to create subscription");

    let deleted_count = fixture
        .dal
        .webhook_subscriptions()
        .delete(created.id)
        .expect("Failed to delete subscription");

    assert_eq!(deleted_count, 1);

    let retrieved = fixture
        .dal
        .webhook_subscriptions()
        .get(created.id)
        .expect("Failed to query subscription");

    assert!(retrieved.is_none());
}

#[test]
fn test_get_matching_subscriptions_exact() {
    let fixture = TestFixture::new();

    let sub1 = create_test_subscription("Health Sub", vec!["health.degraded", "health.failing"]);
    let sub2 = create_test_subscription("Work Order Sub", vec!["workorder.completed"]);

    fixture.dal.webhook_subscriptions().create(&sub1).expect("Failed to create sub1");
    fixture.dal.webhook_subscriptions().create(&sub2).expect("Failed to create sub2");

    let health_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("health.degraded")
        .expect("Failed to get matching subscriptions");

    assert_eq!(health_matches.len(), 1);
    assert_eq!(health_matches[0].name, "Health Sub");

    let work_order_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("workorder.completed")
        .expect("Failed to get matching subscriptions");

    assert_eq!(work_order_matches.len(), 1);
    assert_eq!(work_order_matches[0].name, "Work Order Sub");
}

#[test]
fn test_get_matching_subscriptions_wildcard() {
    let fixture = TestFixture::new();

    // Subscribe to all health events using wildcard
    let wildcard_sub = create_test_subscription("All Health Events", vec!["health.*"]);
    let specific_sub = create_test_subscription("Only Degraded", vec!["health.degraded"]);

    fixture.dal.webhook_subscriptions().create(&wildcard_sub).expect("Failed to create wildcard sub");
    fixture.dal.webhook_subscriptions().create(&specific_sub).expect("Failed to create specific sub");

    // health.degraded should match both
    let degraded_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("health.degraded")
        .expect("Failed to get matching subscriptions");

    assert_eq!(degraded_matches.len(), 2);

    // health.failing should only match wildcard
    let failing_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("health.failing")
        .expect("Failed to get matching subscriptions");

    assert_eq!(failing_matches.len(), 1);
    assert_eq!(failing_matches[0].name, "All Health Events");
}

#[test]
fn test_get_matching_subscriptions_star_wildcard() {
    let fixture = TestFixture::new();

    // Subscribe to all events
    let all_events_sub = create_test_subscription("All Events", vec!["*"]);

    fixture.dal.webhook_subscriptions().create(&all_events_sub).expect("Failed to create all events sub");

    // Any event should match
    let health_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("health.degraded")
        .expect("Failed to get matching subscriptions");

    assert_eq!(health_matches.len(), 1);

    let work_order_matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("workorder.completed")
        .expect("Failed to get matching subscriptions");

    assert_eq!(work_order_matches.len(), 1);
}

#[test]
fn test_disabled_subscriptions_not_matched() {
    let fixture = TestFixture::new();

    let mut disabled_sub = create_test_subscription("Disabled Sub", vec!["health.degraded"]);
    disabled_sub.enabled = false;

    fixture.dal.webhook_subscriptions().create(&disabled_sub).expect("Failed to create disabled sub");

    let matches = fixture
        .dal
        .webhook_subscriptions()
        .get_matching_subscriptions("health.degraded")
        .expect("Failed to get matching subscriptions");

    // Disabled subscriptions should not be matched
    assert_eq!(matches.len(), 0);
}
