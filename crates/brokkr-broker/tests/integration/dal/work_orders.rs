/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::work_orders::{
    NewWorkOrder, NewWorkOrderTarget, WORK_ORDER_STATUS_CLAIMED, WORK_ORDER_STATUS_PENDING,
    WORK_ORDER_STATUS_RETRY_PENDING, WORK_TYPE_BUILD,
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
    let mut new_wo = NewWorkOrder::new(
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
        .complete_failure(work_order.id, "Build failed".to_string())
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
        .complete_failure(work_order.id, "Build failed".to_string())
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
        Some(3),  // max_retries
        Some(1),  // 1 second backoff
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
        .complete_failure(work_order.id, "Failed".to_string())
        .expect("Failed to complete work order");

    // Wait for backoff to elapse
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Process retry pending
    let count = fixture
        .dal
        .work_orders()
        .process_retry_pending()
        .expect("Failed to process retry pending");

    assert_eq!(count, 1);

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
        .complete_failure(wo2.id, "error".to_string())
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
