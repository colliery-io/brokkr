/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use brokkr_models::models::work_orders::{WorkOrder, WorkOrderLog, WORK_TYPE_BUILD};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

async fn make_request(
    app: Router,
    method: &str,
    uri: &str,
    auth: Option<&str>,
    body: Option<String>,
) -> (StatusCode, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(uri);

    if let Some(auth_token) = auth {
        request = request.header("Authorization", format!("Bearer {}", auth_token));
    }

    if body.is_some() {
        request = request.header("Content-Type", "application/json");
    }

    let response = app
        .oneshot(request.body(Body::from(body.unwrap_or_default())).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();

    (status, body_bytes)
}

// =============================================================================
// WORK ORDER MANAGEMENT (ADMIN) TESTS
// =============================================================================

#[tokio::test]
async fn test_create_work_order() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "target_agent_ids": [agent.id]
    });

    let (status, body) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&admin_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["work_type"], WORK_TYPE_BUILD);
    assert_eq!(json["status"], "PENDING");
}

#[tokio::test]
async fn test_create_work_order_no_targets() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "target_agent_ids": []
    });

    let (status, _) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&admin_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_work_order_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "test",
        "target_agent_ids": [agent.id]
    });

    let (status, _) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        None,
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_work_order_forbidden_non_admin() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "test",
        "target_agent_ids": [agent.id]
    });

    let (status, _) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_work_orders() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create some work orders
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");

    let (status, body) = make_request(app, "GET", "/api/v1/work-orders", Some(&admin_pak), None).await;

    assert_eq!(status, StatusCode::OK);

    let json: Vec<WorkOrder> = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.len(), 2);
}

#[tokio::test]
async fn test_list_work_orders_filtered() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    fixture.create_test_work_order("deploy", "yaml2");

    let (status, body) = make_request(
        app,
        "GET",
        &format!("/api/v1/work-orders?work_type={}", WORK_TYPE_BUILD),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: Vec<WorkOrder> = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.len(), 1);
    assert_eq!(json[0].work_type, WORK_TYPE_BUILD);
}

#[tokio::test]
async fn test_get_work_order() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "test yaml");

    let (status, body) = make_request(
        app,
        "GET",
        &format!("/api/v1/work-orders/{}", work_order.id),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: WorkOrder = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, work_order.id);
}

#[tokio::test]
async fn test_get_work_order_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (status, _) = make_request(
        app,
        "GET",
        &format!("/api/v1/work-orders/{}", Uuid::new_v4()),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_work_order() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "test yaml");

    let (status, _) = make_request(
        app,
        "DELETE",
        &format!("/api/v1/work-orders/{}", work_order.id),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's deleted
    let result = fixture.dal.work_orders().get(work_order.id).unwrap();
    assert!(result.is_none());
}

// =============================================================================
// AGENT OPERATIONS TESTS
// =============================================================================

#[tokio::test]
async fn test_list_pending_for_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let wo1 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml1");
    let wo2 = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml2");
    fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml3"); // Not targeted

    fixture.create_test_work_order_target(wo1.id, agent.id);
    fixture.create_test_work_order_target(wo2.id, agent.id);

    let (status, body) = make_request(
        app,
        "GET",
        &format!("/api/v1/agents/{}/work-orders/pending", agent.id),
        Some(&agent_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: Vec<WorkOrder> = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.len(), 2);
}

#[tokio::test]
async fn test_list_pending_for_agent_admin() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(wo.id, agent.id);

    let (status, _) = make_request(
        app,
        "GET",
        &format!("/api/v1/agents/{}/work-orders/pending", agent.id),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_list_pending_for_other_agent_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let (_, agent2_pak) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster".to_string());

    let (status, _) = make_request(
        app,
        "GET",
        &format!("/api/v1/agents/{}/work-orders/pending", agent1.id),
        Some(&agent2_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_claim_work_order() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    let request_body = serde_json::json!({
        "agent_id": agent.id
    });

    let (status, body) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/claim", work_order.id),
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: WorkOrder = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.status, "CLAIMED");
    assert_eq!(json.claimed_by, Some(agent.id));
}

#[tokio::test]
async fn test_claim_work_order_not_targeted() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    // Not adding target

    let request_body = serde_json::json!({
        "agent_id": agent.id
    });

    let (status, _) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/claim", work_order.id),
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_complete_work_order_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim first
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim");

    let request_body = serde_json::json!({
        "success": true,
        "message": "sha256:abc123"
    });

    let (status, body) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/complete", work_order.id),
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["result_message"], "sha256:abc123");
}

#[tokio::test]
async fn test_complete_work_order_failure_with_retry() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Create work order with retries
    let new_wo = brokkr_models::models::work_orders::NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(3), // max_retries
        None,
        None,
    )
    .expect("Failed to create");
    let work_order = fixture.dal.work_orders().create(&new_wo).expect("Failed to create");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim first
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim");

    let request_body = serde_json::json!({
        "success": false,
        "message": "Build failed"
    });

    let (status, body) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/complete", work_order.id),
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    // Should return 202 Accepted (scheduled for retry)
    assert_eq!(status, StatusCode::ACCEPTED);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "retry_scheduled");
}

#[tokio::test]
async fn test_complete_work_order_failure_max_retries() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Create work order with no retries
    let new_wo = brokkr_models::models::work_orders::NewWorkOrder::new(
        WORK_TYPE_BUILD.to_string(),
        "yaml".to_string(),
        Some(0), // max_retries
        None,
        None,
    )
    .expect("Failed to create");
    let work_order = fixture.dal.work_orders().create(&new_wo).expect("Failed to create");

    fixture.create_test_work_order_target(work_order.id, agent.id);

    // Claim first
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent.id)
        .expect("Failed to claim");

    let request_body = serde_json::json!({
        "success": false,
        "message": "Build failed"
    });

    let (status, body) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/complete", work_order.id),
        Some(&agent_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    // Should return 200 OK (moved to log)
    assert_eq!(status, StatusCode::OK);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(!json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_complete_work_order_wrong_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster".to_string());
    let (_, agent2_pak) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster".to_string());

    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(work_order.id, agent1.id);

    // Agent1 claims
    fixture
        .dal
        .work_orders()
        .claim(work_order.id, agent1.id)
        .expect("Failed to claim");

    // Agent2 tries to complete
    let request_body = serde_json::json!({
        "success": true,
        "message": "done"
    });

    let (status, _) = make_request(
        app,
        "POST",
        &format!("/api/v1/work-orders/{}/complete", work_order.id),
        Some(&agent2_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// =============================================================================
// WORK ORDER LOG TESTS
// =============================================================================

#[tokio::test]
async fn test_list_work_order_log() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    // Create and complete a work order
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(wo.id, agent.id);
    fixture.dal.work_orders().claim(wo.id, agent.id).unwrap();
    fixture
        .dal
        .work_orders()
        .complete_success(wo.id, Some("result".to_string()))
        .unwrap();

    let (status, body) = make_request(
        app,
        "GET",
        "/api/v1/work-order-log",
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: Vec<WorkOrderLog> = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.len(), 1);
    assert!(json[0].success);
}

#[tokio::test]
async fn test_get_work_order_log() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Agent".to_string(), "Cluster".to_string());

    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_target(wo.id, agent.id);
    fixture.dal.work_orders().claim(wo.id, agent.id).unwrap();
    fixture
        .dal
        .work_orders()
        .complete_success(wo.id, None)
        .unwrap();

    let (status, body) = make_request(
        app,
        "GET",
        &format!("/api/v1/work-order-log/{}", wo.id),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let json: WorkOrderLog = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, wo.id);
}

#[tokio::test]
async fn test_get_work_order_log_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (status, _) = make_request(
        app,
        "GET",
        &format!("/api/v1/work-order-log/{}", Uuid::new_v4()),
        Some(&admin_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_work_order_log_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (_, agent_pak) =
        fixture.create_test_agent_with_pak("Agent".to_string(), "Cluster".to_string());

    let (status, _) = make_request(
        app,
        "GET",
        "/api/v1/work-order-log",
        Some(&agent_pak),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}
