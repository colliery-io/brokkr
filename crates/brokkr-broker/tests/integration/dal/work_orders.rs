/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::work_orders::{
    NewWorkOrder, WORK_ORDER_STATUS_CLAIMED, WORK_ORDER_STATUS_PENDING,
    WORK_ORDER_STATUS_RETRY_PENDING, WORK_TYPE_BUILD, WorkOrder,
};
use uuid::Uuid;

// =========================================================================
// WORK ORDER CRUD TESTS
// =========================================================================

#[test]
fn test_create_work_order() {
    let fixture = TestFixture::new();

    let new_work_order = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "apiVersion: v1\nkind: ConfigMap".to_string(),
        None,
        None,
        None,
    )
    .expect("Failed to create NewWorkOrder");

    let created = fixture
        .dal
        .work_orders()
        .create(&new_work_order)
        .expect("Failed to create work order");

    assert_eq!(created.work_type, WORK_TYPE_BUILD);
    assert_eq!(created.yaml_content, "apiVersion: v1\nkind: ConfigMap");
    assert_eq!(created.status, WORK_ORDER_STATUS_PENDING);
    assert_eq!(created.retry_count, 0);
    assert!(created.claimed_by.is_none());
    assert!(created.claimed_at.is_none());
}

#[test]
fn test_get_work_order() {
    let fixture = TestFixture::new();

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "test: yaml");

    let retrieved = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to get work order")
        .expect("Work order not found");

    assert_eq!(retrieved.id, work_order.id);
    assert_eq!(retrieved.work_type, work_order.work_type);
}

#[test]
fn test_get_nonexistent_work_order() {
    let fixture = TestFixture::new();

    let result = fixture
        .dal
        .work_orders()
        .get(Uuid::new_v4())
        .expect("Failed to query work order");

    assert!(result.is_none());
}

#[test]
fn test_list_work_orders() {
    let fixture = TestFixture::new();

    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");
    fixture.create_test_work_order("other_type", "yaml3");

    let all_orders = fixture
        .dal
        .work_orders()
        .list()
        .expect("Failed to list work orders");

    assert_eq!(all_orders.len(), 3);
}

#[test]
fn test_list_filtered_by_status() {
    let fixture = TestFixture::new();

    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");

    // Claim the first work order to change its status
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    fixture.create_test_work_order_target(wo1.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(wo1.id, agent.id)
        .expect("Failed to claim work order");

    // Filter by PENDING status
    let pending_orders = fixture
        .dal
        .work_orders()
        .list_filtered(Some(WORK_ORDER_STATUS_PENDING), None)
        .expect("Failed to list filtered work orders");

    assert_eq!(pending_orders.len(), 1);

    // Filter by CLAIMED status
    let claimed_orders = fixture
        .dal
        .work_orders()
        .list_filtered(Some(WORK_ORDER_STATUS_CLAIMED), None)
        .expect("Failed to list filtered work orders");

    assert_eq!(claimed_orders.len(), 1);
    assert_eq!(claimed_orders[0].id, wo1.id);
}

#[test]
fn test_list_filtered_by_work_type() {
    let fixture = TestFixture::new();

    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");
    fixture.create_test_work_order("deploy", "yaml3");

    let build_orders = fixture
        .dal
        .work_orders()
        .list_filtered(None, Some(WORK_TYPE_BUILD))
        .expect("Failed to list filtered work orders");

    assert_eq!(build_orders.len(), 2);
}

#[test]
fn test_delete_work_order() {
    let fixture = TestFixture::new();

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "test: yaml");

    let affected = fixture
        .dal
        .work_orders()
        .delete(work_order.id)
        .expect("Failed to delete work order");

    assert_eq!(affected, 1);

    let result = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to query work order");

    assert!(result.is_none());
}

// =========================================================================
// CLAIM TESTS
// =========================================================================

#[test]
fn test_list_pending_for_agent() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());

    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    let wo2 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");
    let wo3 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml3");

    // Target wo1 and wo2 to agent1, wo3 to agent2
    fixture.create_test_work_order_target(wo1.id, agent1.id);
    fixture.create_test_work_order_target(wo2.id, agent1.id);
    fixture.create_test_work_order_target(wo3.id, agent2.id);

    // Agent1 should see wo1 and wo2
    let agent1_orders = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent1.id, None)
        .expect("Failed to list pending for agent");

    assert_eq!(agent1_orders.len(), 2);

    // Agent2 should see wo3
    let agent2_orders = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent2.id, None)
        .expect("Failed to list pending for agent");

    assert_eq!(agent2_orders.len(), 1);
    assert_eq!(agent2_orders[0].id, wo3.id);
}

#[test]
fn test_list_pending_for_agent_with_work_type_filter() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    let wo2 = fixture.create_test_work_order("deploy", "yaml2");

    fixture.create_test_work_order_target(wo1.id, agent.id);
    fixture.create_test_work_order_target(wo2.id, agent.id);

    // Filter by build type
    let build_orders = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, Some(WORK_TYPE_BUILD))
        .expect("Failed to list pending for agent");

    assert_eq!(build_orders.len(), 1);
    assert_eq!(build_orders[0].id, wo1.id);
}

#[test]
fn test_claim_work_order() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    let claimed = fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    assert_eq!(claimed.status, WORK_ORDER_STATUS_CLAIMED);
    assert_eq!(claimed.claimed_by, Some(agent.id));
    assert!(claimed.claimed_at.is_some());
}

#[test]
fn test_claim_work_order_not_targeted() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    // Note: NOT adding target

    let result = fixture.dal.work_orders().claim(work_order.id, agent.id);

    assert!(result.is_err());
}

#[test]
fn test_claim_already_claimed_work_order() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    fixture.create_test_work_order_target(work_order.id, agent1.id);
    fixture.create_test_work_order_target(work_order.id, agent2.id);

    // Agent1 claims first
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent1.id)
        .expect("Failed to claim work order");

    // Agent2 tries to claim - should fail because status is no longer PENDING
    let result = fixture.dal.work_orders().claim(work_order.id, agent2.id);

    assert!(result.is_err());
}

#[test]
fn test_release_work_order() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim then release
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    let released = fixture
        .dal
        .work_orders()
        .release(work_order.id, agent.id)
        .expect("Failed to release work order");

    assert_eq!(released.status, WORK_ORDER_STATUS_PENDING);
    assert!(released.claimed_by.is_none());
    assert!(released.claimed_at.is_none());
}

#[test]
fn test_release_work_order_wrong_agent() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    fixture.create_test_work_order_target(work_order.id, agent1.id);

    // Agent1 claims
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent1.id)
        .expect("Failed to claim work order");

    // Agent2 tries to release - should fail
    let result = fixture.dal.work_orders().release(work_order.id, agent2.id);

    assert!(result.is_err());
}

// =========================================================================
// COMPLETION TESTS
// =========================================================================

#[test]
fn test_complete_success() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim then complete
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    let log_entry = fixture
        .dal
        .work_orders()
        .complete_success(work_order.id, Some("sha256:abc123".to_string()))
        .expect("Failed to complete work order");

    assert_eq!(log_entry.id, work_order.id);
    assert!(log_entry.success);
    assert_eq!(log_entry.result_message, Some("sha256:abc123".to_string()));
    assert_eq!(log_entry.claimed_by, Some(agent.id));

    // Work order should be deleted
    let result = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to query work order");
    assert!(result.is_none());
}

#[test]
fn test_complete_failure_with_retries() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create work order with retries
    let new_wo = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(3), // max_retries
        None,
        None,
    )
    .expect("Failed to create NewWorkOrder");

    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create work order");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim then fail
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    let result = fixture
        .dal
        .work_orders()
        .complete_failure(work_order.id, "Build failed".to_string(), true)
        .expect("Failed to complete work order");

    // Should return None (scheduled for retry, not moved to log)
    assert!(result.is_none());

    // Work order should still exist with RETRY_PENDING status
    let updated = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to get work order")
        .expect("Work order not found");

    assert_eq!(updated.status, WORK_ORDER_STATUS_RETRY_PENDING);
    assert_eq!(updated.retry_count, 1);
    assert!(updated.next_retry_after.is_some());
}

#[test]
fn test_complete_failure_max_retries_exceeded() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create work order with no retries
    let new_wo = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(0), // max_retries
        None,
        None,
    )
    .expect("Failed to create NewWorkOrder");

    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create work order");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim then fail
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    let result = fixture
        .dal
        .work_orders()
        .complete_failure(work_order.id, "Build failed".to_string(), true)
        .expect("Failed to complete work order");

    // Should return Some (moved to log)
    assert!(result.is_some());
    let log_entry = result.unwrap();

    assert_eq!(log_entry.id, work_order.id);
    assert!(!log_entry.success);
    assert_eq!(log_entry.result_message, Some("Build failed".to_string()));

    // Work order should be deleted
    let deleted = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to query work order");
    assert!(deleted.is_none());
}

#[test]
fn test_complete_failure_non_retryable() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create work order with retries available
    let new_wo = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(3), // max_retries - would normally allow retry
        None,
        None,
    )
    .expect("Failed to create NewWorkOrder");

    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create work order");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim the work order
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    // Complete with non-retryable error - should immediately move to log
    let result = fixture
        .dal
        .work_orders()
        .complete_failure(
            work_order.id,
            "NotFound: resource does not exist".to_string(),
            false,
        )
        .expect("Failed to complete work order");

    // Should return Some (moved to log immediately, no retry)
    assert!(result.is_some());
    let log_entry = result.unwrap();

    assert_eq!(log_entry.id, work_order.id);
    assert!(!log_entry.success);
    assert_eq!(
        log_entry.result_message,
        Some("NotFound: resource does not exist".to_string())
    );
    // Should have 0 retries since it failed immediately
    assert_eq!(log_entry.retries_attempted, 0);

    // Work order should be deleted
    let deleted = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to query work order");
    assert!(deleted.is_none());
}

// =========================================================================
// RETRY AND STALE CLAIM TESTS
// =========================================================================

#[test]
fn test_process_retry_pending() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create work order with very short backoff for testing
    let new_wo = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(3), // max_retries
        Some(1), // 1 second backoff
        None,
    )
    .expect("Failed to create NewWorkOrder");

    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create work order");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim and fail to put in RETRY_PENDING state
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    fixture
        .dal
        .work_orders()
        .complete_failure(work_order.id, "Failed".to_string(), true)
        .expect("Failed to complete work order");

    // Wait for backoff to elapse, then poll. Backoff is 2s; we allow up to
    // 15s so a busy CI runner doesn't flake the assertion. Polling beats
    // a single fixed sleep — the same pattern is used in
    // `webhook_deliveries::test_process_retries`.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut count: usize = 0;
    while std::time::Instant::now() < deadline {
        count = fixture
            .dal
            .work_orders()
            .process_retry_pending()
            .expect("Failed to process retry pending");
        if count >= 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    assert!(
        count >= 1,
        "Should process at least 1 retry within 15s of next_retry_at"
    );

    // Work order should be back to PENDING
    let updated = fixture
        .dal
        .work_orders()
        .get(work_order.id)
        .expect("Failed to get work order")
        .expect("Work order not found");

    assert_eq!(updated.status, WORK_ORDER_STATUS_PENDING);
}

// =========================================================================
// TARGET TESTS
// =========================================================================

#[test]
fn test_add_target() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    let target = fixture.create_test_work_order_target(work_order.id, agent.id);

    assert_eq!(target.work_order_id, work_order.id);
    assert_eq!(target.agent_id, agent.id);
}

#[test]
fn test_add_targets_batch() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster".to_string());
    let agent3 = fixture.create_test_agent("Agent 3".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    let count = fixture
        .dal
        .work_orders()
        .add_targets(work_order.id, &[agent1.id, agent2.id, agent3.id])
        .expect("Failed to add targets");

    assert_eq!(count, 3);

    let targets = fixture
        .dal
        .work_orders()
        .list_targets(work_order.id)
        .expect("Failed to list targets");

    assert_eq!(targets.len(), 3);
}

#[test]
fn test_list_targets() {
    let fixture = TestFixture::new();

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    fixture.create_test_work_order_target(work_order.id, agent1.id);
    fixture.create_test_work_order_target(work_order.id, agent2.id);

    let targets = fixture
        .dal
        .work_orders()
        .list_targets(work_order.id)
        .expect("Failed to list targets");

    assert_eq!(targets.len(), 2);
}

#[test]
fn test_remove_target() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    let count = fixture
        .dal
        .work_orders()
        .remove_target(work_order.id, agent.id)
        .expect("Failed to remove target");

    assert_eq!(count, 1);

    let targets = fixture
        .dal
        .work_orders()
        .list_targets(work_order.id)
        .expect("Failed to list targets");

    assert_eq!(targets.len(), 0);
}

// =========================================================================
// LOG TESTS
// =========================================================================

#[test]
fn test_get_log() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim work order");

    fixture
        .dal
        .work_orders()
        .complete_success(work_order.id, Some("result".to_string()))
        .expect("Failed to complete work order");

    let log_entry = fixture
        .dal
        .work_orders()
        .get_log(work_order.id)
        .expect("Failed to get log")
        .expect("Log entry not found");

    assert_eq!(log_entry.id, work_order.id);
    assert!(log_entry.success);
}

#[test]
fn test_list_log() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create and complete two work orders
    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order_target(wo1.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(wo1.id, agent.id)
        .expect("Failed to claim");
    fixture
        .dal
        .work_orders()
        .complete_success(wo1.id, None)
        .expect("Failed to complete");

    let wo2 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");
    fixture.create_test_work_order_target(wo2.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(wo2.id, agent.id)
        .expect("Failed to claim");
    fixture
        .dal
        .work_orders()
        .complete_success(wo2.id, None)
        .expect("Failed to complete");

    let logs = fixture
        .dal
        .work_orders()
        .list_log(None, None, None, None)
        .expect("Failed to list log");

    assert_eq!(logs.len(), 2);
}

#[test]
fn test_list_log_filtered() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create and complete with success
    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order_target(wo1.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(wo1.id, agent.id)
        .expect("Failed to claim");
    fixture
        .dal
        .work_orders()
        .complete_success(wo1.id, None)
        .expect("Failed to complete");

    // Create and complete with failure (no retries)
    let new_wo = NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml2".to_string(),
        Some(0), // max_retries
        None,
        None,
    )
    .expect("Failed to create");
    let wo2 = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create");
    fixture.create_test_work_order_target(wo2.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(wo2.id, agent.id)
        .expect("Failed to claim");
    fixture
        .dal
        .work_orders()
        .complete_failure(wo2.id, "error".to_string(), true)
        .expect("Failed to complete");

    // Filter by success
    let successful = fixture
        .dal
        .work_orders()
        .list_log(None, Some(true), None, None)
        .expect("Failed to list log");

    assert_eq!(successful.len(), 1);
    assert!(successful[0].success);

    // Filter by failure
    let failed = fixture
        .dal
        .work_orders()
        .list_log(None, Some(false), None, None)
        .expect("Failed to list log");

    assert_eq!(failed.len(), 1);
    assert!(!failed[0].success);
}

#[test]
fn test_list_log_with_limit() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create and complete three work orders
    for i in 1..=3 {
        let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, &format!("yaml{}", i));
        fixture.create_test_work_order_target(wo.id, agent.id);
        fixture
            .dal
            .work_orders()
            .claim(wo.id, agent.id)
            .expect("Failed to claim");
        fixture
            .dal
            .work_orders()
            .complete_success(wo.id, None)
            .expect("Failed to complete");
    }

    let logs = fixture
        .dal
        .work_orders()
        .list_log(None, None, None, Some(2))
        .expect("Failed to list log");

    assert_eq!(logs.len(), 2);
}

// =========================================================================
// LABEL AND ANNOTATION TARGETING TESTS
// =========================================================================

#[test]
fn test_add_label() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    let label = fixture.create_test_work_order_label(wo.id, "production");

    assert_eq!(label.work_order_id, wo.id);
    assert_eq!(label.label, "production");
}

#[test]
fn test_add_multiple_labels() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    let labels = vec![
        "production".to_string(),
        "gpu".to_string(),
        "us-east".to_string(),
    ];

    let count = fixture
        .dal
        .work_orders()
        .add_labels(wo.id, &labels)
        .expect("Failed to add labels");

    assert_eq!(count, 3);

    let retrieved = fixture
        .dal
        .work_orders()
        .list_labels(wo.id)
        .expect("Failed to list labels");

    assert_eq!(retrieved.len(), 3);
}

#[test]
fn test_remove_label() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "production");

    let removed = fixture
        .dal
        .work_orders()
        .remove_label(wo.id, "production")
        .expect("Failed to remove label");

    assert_eq!(removed, 1);

    let remaining = fixture
        .dal
        .work_orders()
        .list_labels(wo.id)
        .expect("Failed to list labels");

    assert_eq!(remaining.len(), 0);
}

#[test]
fn test_add_annotation() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    let annotation = fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    assert_eq!(annotation.work_order_id, wo.id);
    assert_eq!(annotation.key, "region");
    assert_eq!(annotation.value, "us-east");
}

#[test]
fn test_add_multiple_annotations() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    let mut annotations = std::collections::HashMap::new();
    annotations.insert("region".to_string(), "us-east".to_string());
    annotations.insert("tier".to_string(), "production".to_string());

    let count = fixture
        .dal
        .work_orders()
        .add_annotations(wo.id, &annotations)
        .expect("Failed to add annotations");

    assert_eq!(count, 2);

    let retrieved = fixture
        .dal
        .work_orders()
        .list_annotations(wo.id)
        .expect("Failed to list annotations");

    assert_eq!(retrieved.len(), 2);
}

#[test]
fn test_remove_annotation() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    let removed = fixture
        .dal
        .work_orders()
        .remove_annotation(wo.id, "region", "us-east")
        .expect("Failed to remove annotation");

    assert_eq!(removed, 1);

    let remaining = fixture
        .dal
        .work_orders()
        .list_annotations(wo.id)
        .expect("Failed to list annotations");

    assert_eq!(remaining.len(), 0);
}

#[test]
fn test_list_pending_for_agent_with_label_match() {
    let fixture = TestFixture::new();

    // Create an agent with a label
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "gpu".to_string());

    // Create a work order that targets agents with the "gpu" label
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");

    // Agent should see the work order (label match)
    let pending = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, None)
        .expect("Failed to list pending");

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, wo.id);
}

#[test]
fn test_list_pending_for_agent_with_annotation_match() {
    let fixture = TestFixture::new();

    // Create an agent with an annotation
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-east".to_string());

    // Create a work order that targets agents with the same annotation
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    // Agent should see the work order (annotation match)
    let pending = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, None)
        .expect("Failed to list pending");

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, wo.id);
}

#[test]
fn test_list_pending_for_agent_no_match() {
    let fixture = TestFixture::new();

    // Create an agent with a label
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "cpu".to_string());

    // Create a work order that targets agents with a different label
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");

    // Agent should NOT see the work order (no match)
    let pending = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, None)
        .expect("Failed to list pending");

    assert_eq!(pending.len(), 0);
}

#[test]
fn test_list_pending_for_agent_or_logic() {
    let fixture = TestFixture::new();

    // Create an agent with one label
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "production".to_string());

    // Create a work order that targets agents with multiple labels (OR logic)
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "staging");
    fixture.create_test_work_order_label(wo.id, "production");

    // Agent should see the work order (matches "production")
    let pending = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, None)
        .expect("Failed to list pending");

    assert_eq!(pending.len(), 1);
}

#[test]
fn test_list_pending_for_agent_combined_targeting() {
    let fixture = TestFixture::new();

    // Create agents
    let agent1 = fixture.create_test_agent("Agent1".to_string(), "Cluster".to_string());
    let agent2 = fixture.create_test_agent("Agent2".to_string(), "Cluster".to_string());
    let agent3 = fixture.create_test_agent("Agent3".to_string(), "Cluster".to_string());

    // Agent1 has a label that matches
    fixture.create_test_agent_label(agent1.id, "gpu".to_string());
    // Agent2 has an annotation that matches
    fixture.create_test_agent_annotation(agent2.id, "region".to_string(), "us-east".to_string());
    // Agent3 is a hard target

    // Create a work order with all three targeting methods
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");
    fixture.create_test_work_order_target(wo.id, agent3.id);

    // All three agents should see the work order
    for agent in &[agent1, agent2, agent3] {
        let pending = fixture
            .dal
            .work_orders()
            .list_pending_for_agent(agent.id, None)
            .expect("Failed to list pending");

        assert_eq!(
            pending.len(),
            1,
            "Agent {} should see the work order",
            agent.name
        );
        assert_eq!(pending[0].id, wo.id);
    }
}

#[test]
fn test_claim_with_label_match() {
    let fixture = TestFixture::new();

    // Create an agent with a label
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "gpu".to_string());

    // Create a work order that targets agents with the "gpu" label
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");

    // Agent should be able to claim the work order
    let claimed = fixture
        .dal
        .work_orders()
        .claim(wo.id, agent.id)
        .expect("Failed to claim work order");

    assert_eq!(claimed.status, WORK_ORDER_STATUS_CLAIMED);
    assert_eq!(claimed.claimed_by, Some(agent.id));
}

#[test]
fn test_claim_with_annotation_match() {
    let fixture = TestFixture::new();

    // Create an agent with an annotation
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-east".to_string());

    // Create a work order that targets agents with the same annotation
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    // Agent should be able to claim the work order
    let claimed = fixture
        .dal
        .work_orders()
        .claim(wo.id, agent.id)
        .expect("Failed to claim work order");

    assert_eq!(claimed.status, WORK_ORDER_STATUS_CLAIMED);
    assert_eq!(claimed.claimed_by, Some(agent.id));
}

#[test]
fn test_claim_without_authorization() {
    let fixture = TestFixture::new();

    // Create an agent with no matching attributes
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_label(agent.id, "cpu".to_string());

    // Create a work order that targets agents with a different label
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");

    // Agent should NOT be able to claim the work order
    let result = fixture.dal.work_orders().claim(wo.id, agent.id);

    assert!(result.is_err());
    match result {
        Err(diesel::result::Error::NotFound) => {} // Expected
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_annotation_key_value_must_both_match() {
    let fixture = TestFixture::new();

    // Create an agent with an annotation
    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-west".to_string());

    // Create a work order that targets a different value for the same key
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    // Agent should NOT see the work order (key matches but value doesn't)
    let pending = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent.id, None)
        .expect("Failed to list pending");

    assert_eq!(pending.len(), 0);
}

#[test]
fn test_labels_deleted_on_work_order_delete() {
    let fixture = TestFixture::new();

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "production");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

    // Verify labels and annotations exist
    let labels = fixture
        .dal
        .work_orders()
        .list_labels(wo.id)
        .expect("Failed to list labels");
    assert_eq!(labels.len(), 1);

    let annotations = fixture
        .dal
        .work_orders()
        .list_annotations(wo.id)
        .expect("Failed to list annotations");
    assert_eq!(annotations.len(), 1);

    // Delete the work order
    fixture
        .dal
        .work_orders()
        .delete(wo.id)
        .expect("Failed to delete work order");

    // Labels and annotations should be deleted via CASCADE
    // (we can't easily test this directly since the FK no longer exists,
    // but we can verify the work order is gone)
    let result = fixture
        .dal
        .work_orders()
        .get(wo.id)
        .expect("Failed to query work order");
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// BROKKR-T-0402: a work order has an owner (BROKKR-A-0013).
// ---------------------------------------------------------------------------

/// A pending work order owned by `owner` (a generator, or `None` for an admin).
fn owned_work_order(fixture: &TestFixture, owner: Option<Uuid>) -> WorkOrder {
    let new_work_order =
        NewWorkOrder::new("custom".to_string(), "a: 1".to_string(), None, None, None)
            .expect("NewWorkOrder")
            .owned_by(owner);
    fixture
        .dal
        .work_orders()
        .create(&new_work_order)
        .expect("Failed to create work order")
}

#[test]
fn test_a_work_order_keeps_the_owner_of_its_creator() {
    let fixture = TestFixture::new();
    let (generator, _pak) = fixture.create_test_generator_with_pak("t0402-owner".to_string(), None);

    let owned = owned_work_order(&fixture, Some(generator.id));
    assert_eq!(owned.owner_generator_id, Some(generator.id));
    let read = fixture.dal.work_orders().get(owned.id).unwrap().unwrap();
    assert_eq!(read.owner_generator_id, Some(generator.id));

    // A work order with no owner is an admin's: the shape of each row from
    // before the migration.
    let admin_owned = owned_work_order(&fixture, None);
    assert_eq!(admin_owned.owner_generator_id, None);
    let plain = fixture.create_test_work_order("custom", "a: 2");
    assert_eq!(plain.owner_generator_id, None);
}

#[test]
fn test_the_log_entry_keeps_the_owner() {
    let fixture = TestFixture::new();
    let (generator, _pak) = fixture.create_test_generator_with_pak("t0402-log".to_string(), None);
    let agent = fixture.create_test_agent("t0402-log-agent".to_string(), "c".to_string());
    // The agent must be registered with the owner to claim (BROKKR-T-0403).
    fixture.register_agent_with_generator(agent.id, generator.id);

    // One ends well, one fails with no retry: both paths write the log.
    for success in [true, false] {
        let work_order = owned_work_order(&fixture, Some(generator.id));
        fixture.create_test_work_order_target(work_order.id, agent.id);
        fixture
            .dal
            .work_orders()
            .claim(work_order.id, agent.id)
            .expect("claim failed");
        if success {
            fixture
                .dal
                .work_orders()
                .complete_success(work_order.id, Some("ok".to_string()))
                .expect("complete_success failed");
        } else {
            fixture
                .dal
                .work_orders()
                .complete_failure(work_order.id, "no".to_string(), false)
                .expect("complete_failure failed");
        }

        assert!(
            fixture
                .dal
                .work_orders()
                .get(work_order.id)
                .unwrap()
                .is_none()
        );
        let log = fixture
            .dal
            .work_orders()
            .get_log(work_order.id)
            .unwrap()
            .expect("the work order is in the log");
        assert_eq!(log.success, success);
        assert_eq!(log.owner_generator_id, Some(generator.id));
    }
}

#[test]
fn test_a_deleted_generator_takes_its_waiting_work_orders() {
    let fixture = TestFixture::new();
    let (generator, _pak) =
        fixture.create_test_generator_with_pak("t0402-delete".to_string(), None);
    let agent = fixture.create_test_agent("t0402-delete-agent".to_string(), "c".to_string());
    // The agent must be registered with the owner to claim (BROKKR-T-0403).
    fixture.register_agent_with_generator(agent.id, generator.id);

    let waiting = owned_work_order(&fixture, Some(generator.id));
    let claimed = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_target(claimed.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(claimed.id, agent.id)
        .expect("claim failed");
    let ended = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_target(ended.id, agent.id);
    fixture
        .dal
        .work_orders()
        .claim(ended.id, agent.id)
        .expect("claim failed");
    fixture
        .dal
        .work_orders()
        .complete_success(ended.id, None)
        .expect("complete_success failed");
    let of_an_admin = owned_work_order(&fixture, None);

    fixture
        .dal
        .generators()
        .soft_delete(generator.id)
        .expect("soft_delete failed");

    let get = |id| fixture.dal.work_orders().get(id).expect("get failed");
    assert!(get(waiting.id).is_none(), "a waiting work order is deleted");
    assert!(
        get(claimed.id).is_some(),
        "a claimed work order is left to end"
    );
    assert!(get(of_an_admin.id).is_some(), "an admin's work order stays");
    assert!(
        fixture
            .dal
            .work_orders()
            .get_log(ended.id)
            .unwrap()
            .is_some(),
        "the log entry stays"
    );
}

// ---------------------------------------------------------------------------
// BROKKR-T-0403: a generator's work order reaches only the agents registered
// with that generator (BROKKR-A-0013).
// ---------------------------------------------------------------------------

/// The ids of the pending work orders an agent can have, sorted.
fn pending_ids(fixture: &TestFixture, agent: Uuid) -> Vec<Uuid> {
    let mut ids: Vec<Uuid> = fixture
        .dal
        .work_orders()
        .list_pending_for_agent(agent, None)
        .expect("list_pending_for_agent failed")
        .into_iter()
        .map(|w| w.id)
        .collect();
    ids.sort();
    ids
}

/// The fleet's pending count for one agent.
fn pending_count(fixture: &TestFixture, agent: Uuid) -> i64 {
    fixture
        .dal
        .work_orders()
        .pending_counts_by_agent()
        .expect("pending_counts_by_agent failed")
        .into_iter()
        .find(|(a, _)| *a == agent)
        .map(|(_, n)| n)
        .unwrap_or(0)
}

#[test]
fn test_a_generator_owned_work_order_reaches_registered_agents_only() {
    let fixture = TestFixture::new();
    let (generator, _pak) = fixture.create_test_generator_with_pak("t0403-owner".to_string(), None);

    // Both agents carry the same label and annotation. Only `inside` is
    // registered with the generator.
    let inside = fixture.create_test_agent("t0403-inside".to_string(), "c".to_string());
    let outside = fixture.create_test_agent("t0403-outside".to_string(), "c".to_string());
    fixture.register_agent_with_generator(inside.id, generator.id);
    for agent in [&inside, &outside] {
        fixture.create_test_agent_label(agent.id, "t0403-team".to_string());
        fixture.create_test_agent_annotation(
            agent.id,
            "t0403-tier".to_string(),
            "gold".to_string(),
        );
    }

    // Three work orders of the generator: by id, by label, by annotation. The
    // target rows name both agents: the DAL writes them with no check, as a
    // row from before a deregistration would be.
    let by_id = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_target(by_id.id, inside.id);
    fixture.create_test_work_order_target(by_id.id, outside.id);
    let by_label = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_label(by_label.id, "t0403-team");
    let by_annotation = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_annotation(by_annotation.id, "t0403-tier", "gold");

    // An admin's work order by the same label reaches both, as before.
    let of_an_admin = owned_work_order(&fixture, None);
    fixture.create_test_work_order_label(of_an_admin.id, "t0403-team");

    let mut all = vec![by_id.id, by_label.id, by_annotation.id, of_an_admin.id];
    all.sort();
    assert_eq!(
        pending_ids(&fixture, inside.id),
        all,
        "the registered agent has each work order"
    );
    assert_eq!(
        pending_ids(&fixture, outside.id),
        vec![of_an_admin.id],
        "the agent that is not registered has only the admin's work order"
    );

    // The fleet's count is the length of the pending list, for each agent.
    for agent in [&inside, &outside] {
        assert_eq!(
            pending_count(&fixture, agent.id),
            pending_ids(&fixture, agent.id).len() as i64,
            "pending count mismatch for agent {}",
            agent.name
        );
    }

    // The claim follows the same rule.
    for work_order in [&by_id, &by_label, &by_annotation] {
        assert!(
            matches!(
                fixture.dal.work_orders().claim(work_order.id, outside.id),
                Err(diesel::result::Error::NotFound)
            ),
            "the agent that is not registered cannot claim"
        );
        let claimed = fixture
            .dal
            .work_orders()
            .claim(work_order.id, inside.id)
            .expect("the registered agent claims");
        assert_eq!(claimed.claimed_by, Some(inside.id));
    }
    fixture
        .dal
        .work_orders()
        .claim(of_an_admin.id, outside.id)
        .expect("an admin's work order is claimed as before");
}

#[test]
fn test_a_deregistered_agent_loses_the_generator_work_orders() {
    let fixture = TestFixture::new();
    let (generator, _pak) = fixture.create_test_generator_with_pak("t0403-dereg".to_string(), None);
    let agent = fixture.create_test_agent("t0403-dereg-agent".to_string(), "c".to_string());
    fixture.register_agent_with_generator(agent.id, generator.id);
    let work_order = owned_work_order(&fixture, Some(generator.id));
    fixture.create_test_work_order_target(work_order.id, agent.id);
    assert_eq!(pending_ids(&fixture, agent.id), vec![work_order.id]);

    fixture
        .dal
        .agent_generator_registrations()
        .delete(agent.id, generator.id)
        .expect("deregister failed");

    assert!(
        pending_ids(&fixture, agent.id).is_empty(),
        "the target row is still there, but the agent is outside the registrations"
    );
    assert_eq!(pending_count(&fixture, agent.id), 0);
    assert!(
        fixture
            .dal
            .work_orders()
            .claim(work_order.id, agent.id)
            .is_err()
    );
}

#[test]
fn test_a_target_outside_the_registrations_is_refused_before_the_insert() {
    use brokkr_broker::api::v1::work_orders::refuse_unregistered_targets;

    let fixture = TestFixture::new();
    let (generator, _pak) =
        fixture.create_test_generator_with_pak("t0403-create".to_string(), None);
    let inside = fixture.create_test_agent("t0403-create-in".to_string(), "c".to_string());
    let outside = fixture.create_test_agent("t0403-create-out".to_string(), "c".to_string());
    fixture.register_agent_with_generator(inside.id, generator.id);

    assert_eq!(
        fixture
            .dal
            .work_orders()
            .unregistered_targets(generator.id, &[inside.id, outside.id])
            .expect("unregistered_targets failed"),
        vec![outside.id]
    );

    // The check of the create route: it runs before the insert, so a refusal
    // leaves the queue as it was.
    let queued = |fixture: &TestFixture| fixture.dal.work_orders().list().expect("list").len();
    let before = queued(&fixture);
    let refusal = refuse_unregistered_targets(&fixture.dal, generator.id, &[inside.id, outside.id])
        .expect_err("a target outside the registrations is refused");
    assert_eq!(refusal.status, axum::http::StatusCode::FORBIDDEN);
    assert_eq!(refusal.code, "agent_not_registered");
    assert!(refusal.message.contains(&outside.id.to_string()));
    assert_eq!(queued(&fixture), before, "no work order row is left");

    assert!(refuse_unregistered_targets(&fixture.dal, generator.id, &[inside.id]).is_ok());
    assert!(refuse_unregistered_targets(&fixture.dal, generator.id, &[]).is_ok());
}
