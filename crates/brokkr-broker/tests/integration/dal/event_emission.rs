/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for webhook event emission.
//!
//! These tests verify that various actions in the system correctly emit
//! webhook events and create corresponding delivery records.

use crate::fixtures::TestFixture;
use brokkr_models::models::webhooks::{NewWebhookSubscription, EVENT_WORKORDER_COMPLETED};
use brokkr_models::models::work_orders::NewWorkOrder;

fn create_subscription_for_event(name: &str, event_type: &str) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some(event_type.to_string())],
        filters: None,
        target_labels: None, // Broker delivery
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    }
}

fn create_disabled_subscription(name: &str, event_type: &str) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some(event_type.to_string())],
        filters: None,
        target_labels: None,
        enabled: false, // DISABLED
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    }
}

fn create_subscription_with_target_labels(
    name: &str,
    event_type: &str,
    labels: Vec<String>,
) -> NewWebhookSubscription {
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some(event_type.to_string())],
        filters: None,
        target_labels: Some(labels.into_iter().map(Some).collect()),
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    }
}

fn create_subscription_with_agent_filter(
    name: &str,
    event_type: &str,
    agent_id: uuid::Uuid,
) -> NewWebhookSubscription {
    let filters = serde_json::json!({ "agent_id": agent_id.to_string() });
    NewWebhookSubscription {
        name: name.to_string(),
        url_encrypted: b"https://example.com/webhook".to_vec(),
        auth_header_encrypted: None,
        event_types: vec![Some(event_type.to_string())],
        filters: Some(filters.to_string()),
        target_labels: None,
        enabled: true,
        max_retries: 5,
        timeout_seconds: 30,
        created_by: Some("test".to_string()),
    }
}

// =============================================================================
// Work Order Completion Event Tests
// =============================================================================

#[test]
fn test_work_order_completion_emits_event() {
    let fixture = TestFixture::new();

    // Create subscription for workorder.completed events
    let sub = create_subscription_for_event("WO Completion Sub", EVENT_WORKORDER_COMPLETED);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Create an agent and work order
    let (agent, _) = fixture.create_test_agent_with_pak(
        format!("Agent-emit-{}", uuid::Uuid::new_v4()),
        format!("cluster-emit-{}", uuid::Uuid::new_v4()),
    );

    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    // Add target for the agent
    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    // Claim the work order
    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    // Complete the work order - this should emit an event
    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Completed successfully".to_string()))
        .unwrap();

    // Verify a webhook delivery was created
    let deliveries = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries.len(), 1, "Should have created one delivery for workorder.completed");
    assert_eq!(deliveries[0].event_type, EVENT_WORKORDER_COMPLETED);
    assert_eq!(deliveries[0].status, "pending");

    // Verify payload contains work order info
    let payload: serde_json::Value = serde_json::from_str(&deliveries[0].payload).unwrap();
    assert!(payload["data"]["work_order_log_id"].is_string(), "Expected work_order_log_id in payload: {:?}", payload);
    assert!(payload["data"]["success"].as_bool().unwrap());
}

#[test]
fn test_wildcard_subscription_matches_events() {
    let fixture = TestFixture::new();

    // Create agent and work order first (these emit workorder.created and workorder.claimed)
    let (agent, _) = fixture.create_test_agent_with_pak(
        "Agent-wildcard".to_string(),
        "cluster-wildcard".to_string(),
    );

    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    // NOW create subscription with wildcard pattern - after create/claim events
    let sub = create_subscription_for_event("WO Wildcard Sub", "workorder.*");
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Complete the work order - this emits workorder.completed which matches workorder.*
    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Done".to_string()))
        .unwrap();

    // Verify wildcard matched the completion event
    let deliveries = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries.len(), 1, "Wildcard should match workorder.completed");
    assert_eq!(deliveries[0].event_type, EVENT_WORKORDER_COMPLETED);
}

#[test]
fn test_disabled_subscription_receives_no_deliveries() {
    let fixture = TestFixture::new();

    // Create DISABLED subscription
    let sub = create_disabled_subscription("Disabled Sub", EVENT_WORKORDER_COMPLETED);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Complete a work order
    let (agent, _) = fixture.create_test_agent_with_pak(
        format!("Agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Done".to_string()))
        .unwrap();

    // Verify no delivery was created for disabled subscription
    let deliveries = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries.len(), 0, "Disabled subscription should receive no deliveries");
}

#[test]
fn test_delivery_inherits_target_labels_from_subscription() {
    let fixture = TestFixture::new();

    // Create subscription with target labels
    let sub = create_subscription_with_target_labels(
        "Agent Delivery Sub",
        EVENT_WORKORDER_COMPLETED,
        vec!["env:prod".to_string(), "region:us".to_string()],
    );
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Complete a work order
    let (agent, _) = fixture.create_test_agent_with_pak(
        format!("Agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Done".to_string()))
        .unwrap();

    // Verify delivery has target_labels from subscription
    let deliveries = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries.len(), 1);

    let target_labels = deliveries[0].target_labels.as_ref()
        .expect("Delivery should have target_labels");

    assert_eq!(target_labels.len(), 2);
    assert!(target_labels.contains(&Some("env:prod".to_string())));
    assert!(target_labels.contains(&Some("region:us".to_string())));
}

#[test]
fn test_no_delivery_when_no_matching_subscription() {
    let fixture = TestFixture::new();

    // Create agent FIRST so we can filter by it
    let (agent, _) = fixture.create_test_agent_with_pak(
        format!("Agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    // Create subscription for a DIFFERENT event type (agent.registered, not workorder.completed)
    // Also add agent filter for test isolation
    let sub = create_subscription_with_agent_filter("Agent Events Sub", "agent.registered", agent.id);
    let subscription = fixture.dal.webhook_subscriptions().create(&sub).unwrap();

    // Complete a work order (emits workorder.completed, not agent.registered)
    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Done".to_string()))
        .unwrap();

    // Verify no delivery was created (event type doesn't match)
    let deliveries = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries.len(), 0, "Non-matching subscription should receive no deliveries");
}

#[test]
fn test_multiple_subscriptions_receive_same_event() {
    let fixture = TestFixture::new();

    // Create two subscriptions for the same event
    let sub1 = create_subscription_for_event("Sub 1", EVENT_WORKORDER_COMPLETED);
    let subscription1 = fixture.dal.webhook_subscriptions().create(&sub1).unwrap();

    let sub2 = create_subscription_for_event("Sub 2", EVENT_WORKORDER_COMPLETED);
    let subscription2 = fixture.dal.webhook_subscriptions().create(&sub2).unwrap();

    // Complete a work order
    let (agent, _) = fixture.create_test_agent_with_pak(
        format!("Agent-{}", uuid::Uuid::new_v4()),
        format!("cluster-{}", uuid::Uuid::new_v4()),
    );

    let wo = NewWorkOrder::new(
        "custom".to_string(),
        "test: yaml".to_string(),
        None,
        None,
        None,
    ).unwrap();
    let work_order = fixture.dal.work_orders().create(&wo).unwrap();

    fixture.dal.work_orders()
        .add_targets(work_order.id, &[agent.id])
        .unwrap();

    fixture.dal.work_orders()
        .claim(work_order.id, agent.id)
        .unwrap();

    fixture.dal.work_orders()
        .complete_success(work_order.id, Some("Done".to_string()))
        .unwrap();

    // Verify both subscriptions received a delivery
    let deliveries1 = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription1.id, None, 100, 0)
        .unwrap();

    let deliveries2 = fixture.dal.webhook_deliveries()
        .list_for_subscription(subscription2.id, None, 100, 0)
        .unwrap();

    assert_eq!(deliveries1.len(), 1, "Subscription 1 should receive delivery");
    assert_eq!(deliveries2.len(), 1, "Subscription 2 should receive delivery");

    // Deliveries should have the same event_id (same event, different subscriptions)
    let event_id1 = &deliveries1[0].event_id;
    let event_id2 = &deliveries2[0].event_id;
    assert_eq!(event_id1, event_id2, "Both deliveries should have same event_id");
}
