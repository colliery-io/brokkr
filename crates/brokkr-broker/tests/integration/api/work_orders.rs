/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use brokkr_models::models::work_orders::{WORK_TYPE_BUILD, WorkOrder, WorkOrderLog};
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
async fn test_create_work_order_empty_targets() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Empty target_agent_ids with no other targeting = bad request
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

    let (status, body) =
        make_request(app, "GET", "/api/v1/work-orders", Some(&admin_pak), None).await;

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
    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create");

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
    let work_order = fixture
        .dal
        .work_orders()
        .create(&new_wo)
        .expect("Failed to create");

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

    let (status, body) =
        make_request(app, "GET", "/api/v1/work-order-log", Some(&admin_pak), None).await;

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

    let (status, _) =
        make_request(app, "GET", "/api/v1/work-order-log", Some(&agent_pak), None).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// =============================================================================
// LABEL/ANNOTATION TARGETING API TESTS
// =============================================================================

#[tokio::test]
async fn test_create_work_order_with_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "targeting": {
            "labels": ["gpu", "production"]
        }
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
    let work_order_id: Uuid = json["id"].as_str().unwrap().parse().unwrap();

    // Verify labels were added
    let labels = fixture
        .dal
        .work_orders()
        .list_labels(work_order_id)
        .expect("Failed to list labels");

    assert_eq!(labels.len(), 2);
    let label_values: Vec<&str> = labels.iter().map(|l| l.label.as_str()).collect();
    assert!(label_values.contains(&"gpu"));
    assert!(label_values.contains(&"production"));
}

#[tokio::test]
async fn test_create_work_order_with_annotations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "targeting": {
            "annotations": {
                "region": "us-east",
                "tier": "production"
            }
        }
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
    let work_order_id: Uuid = json["id"].as_str().unwrap().parse().unwrap();

    // Verify annotations were added
    let annotations = fixture
        .dal
        .work_orders()
        .list_annotations(work_order_id)
        .expect("Failed to list annotations");

    assert_eq!(annotations.len(), 2);
}

#[tokio::test]
async fn test_create_work_order_with_combined_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "targeting": {
            "agent_ids": [agent.id],
            "labels": ["gpu"],
            "annotations": {"region": "us-east"}
        }
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
    let work_order_id: Uuid = json["id"].as_str().unwrap().parse().unwrap();

    // Verify all targeting was added
    let targets = fixture
        .dal
        .work_orders()
        .list_targets(work_order_id)
        .expect("Failed to list targets");
    assert_eq!(targets.len(), 1);

    let labels = fixture
        .dal
        .work_orders()
        .list_labels(work_order_id)
        .expect("Failed to list labels");
    assert_eq!(labels.len(), 1);

    let annotations = fixture
        .dal
        .work_orders()
        .list_annotations(work_order_id)
        .expect("Failed to list annotations");
    assert_eq!(annotations.len(), 1);
}

#[tokio::test]
async fn test_create_work_order_no_targeting_fails() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // No targeting at all
    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap"
    });

    let (status, body) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&admin_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    // v1 ErrorResponse shape (T-A1): { "code": "...", "message": "...", ... }
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"].as_str().unwrap(), "no_targeting_specified");
}

#[tokio::test]
async fn test_create_work_order_empty_targeting_fails() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Empty targeting block
    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "targeting": {}
    });

    let (status, body) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&admin_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    // v1 ErrorResponse shape (T-A1): { "code": "...", "message": "...", ... }
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"].as_str().unwrap(), "no_targeting_specified");
}

#[tokio::test]
async fn test_create_work_order_legacy_target_agent_ids() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Use legacy target_agent_ids field
    let request_body = serde_json::json!({
        "work_type": WORK_TYPE_BUILD,
        "yaml_content": "apiVersion: v1\nkind: ConfigMap",
        "target_agent_ids": [agent.id]
    });

    let (status, _) = make_request(
        app,
        "POST",
        "/api/v1/work-orders",
        Some(&admin_pak),
        Some(serde_json::to_string(&request_body).unwrap()),
    )
    .await;

    // Should still work for backwards compatibility
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_pending_with_label_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Add label to agent
    fixture.create_test_agent_label(agent.id, "gpu".to_string());

    // Create work order targeting "gpu" label
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(wo.id, "gpu");

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
    assert_eq!(json.len(), 1);
    assert_eq!(json[0].id, wo.id);
}

#[tokio::test]
async fn test_list_pending_with_annotation_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Add annotation to agent
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-east".to_string());

    // Create work order targeting "region=us-east" annotation
    let wo = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(wo.id, "region", "us-east");

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
    assert_eq!(json.len(), 1);
    assert_eq!(json[0].id, wo.id);
}

#[tokio::test]
async fn test_claim_with_label_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Add label to agent
    fixture.create_test_agent_label(agent.id, "gpu".to_string());

    // Create work order targeting "gpu" label (no hard target)
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(work_order.id, "gpu");

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
async fn test_claim_with_annotation_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Add annotation to agent
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-east".to_string());

    // Create work order targeting "region=us-east" annotation (no hard target)
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_annotation(work_order.id, "region", "us-east");

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
async fn test_claim_with_no_matching_targeting() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());

    // Agent has "cpu" label
    fixture.create_test_agent_label(agent.id, "cpu".to_string());

    // Create work order targeting "gpu" label (agent doesn't have it)
    let work_order = fixture.create_test_work_order(WORK_TYPE_BUILD, "yaml");
    fixture.create_test_work_order_label(work_order.id, "gpu");

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

    // Should fail - agent doesn't match any targeting
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// =============================================================================
// A GENERATOR'S WORK ORDERS (BROKKR-T-0404, BROKKR-A-0013)
// =============================================================================

/// One request, with the body parsed as JSON (`Null` when it is empty).
async fn call(
    fixture: &TestFixture,
    method: &str,
    uri: &str,
    pak: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (status, bytes) = make_request(
        app,
        method,
        uri,
        Some(pak),
        body.map(|b| serde_json::to_string(&b).unwrap()),
    )
    .await;
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Two generators with their PAKs, and an agent registered with the first.
struct Owners {
    owner: brokkr_models::models::generator::Generator,
    owner_pak: String,
    other_pak: String,
    agent: brokkr_models::models::agents::Agent,
    agent_pak: String,
}

fn owners(fixture: &TestFixture, tag: &str) -> Owners {
    let (owner, owner_pak) = fixture.create_test_generator_with_pak(format!("{tag}-owner"), None);
    let (_other, other_pak) = fixture.create_test_generator_with_pak(format!("{tag}-other"), None);
    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak(format!("{tag}-agent"), "cluster".to_string());
    fixture.register_agent_with_generator(agent.id, owner.id);
    Owners {
        owner,
        owner_pak,
        other_pak,
        agent,
        agent_pak,
    }
}

/// The owner creates a work order for its agent, through the route.
async fn create_as(fixture: &TestFixture, pak: &str, agent: Uuid) -> (StatusCode, Value) {
    call(
        fixture,
        "POST",
        "/api/v1/work-orders",
        pak,
        Some(serde_json::json!({
            "work_type": "custom",
            "yaml_content": "apiVersion: v1\nkind: ConfigMap",
            "targeting": { "agent_ids": [agent] }
        })),
    )
    .await
}

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .expect("a list")
        .iter()
        .map(|w| w["id"].as_str().expect("an id").to_string())
        .collect()
}

#[tokio::test]
async fn a_generator_creates_a_work_order_and_owns_it() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-create");

    let (status, body) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["owner_generator_id"], o.owner.id.to_string());
    assert_eq!(body["status"], "PENDING");

    // An admin's work order has no owner.
    let (status, body) = create_as(&fixture, &fixture.admin_pak, o.agent.id).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["owner_generator_id"], Value::Null);
}

#[tokio::test]
async fn a_generator_cannot_target_an_agent_that_is_not_registered_with_it() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-target");
    let outsider = fixture.create_test_agent("t0404-target-outsider".to_string(), "c".to_string());
    let before = fixture.dal.work_orders().list().unwrap().len();

    let (status, body) = create_as(&fixture, &o.owner_pak, outsider.id).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["code"], "agent_not_registered");
    assert_eq!(
        fixture.dal.work_orders().list().unwrap().len(),
        before,
        "a refused create leaves no work order row"
    );
}

#[tokio::test]
async fn a_generator_lists_its_own_work_orders_only() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-list");

    let (_, mine) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    let mine = mine["id"].as_str().unwrap().to_string();
    // An admin's work order for the same agent: the generator must not see it.
    let (_, admins) = create_as(&fixture, &fixture.admin_pak, o.agent.id).await;
    let admins = admins["id"].as_str().unwrap().to_string();

    let (status, list) = call(&fixture, "GET", "/api/v1/work-orders", &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ids(&list), vec![mine.clone()]);

    let (status, list) = call(&fixture, "GET", "/api/v1/work-orders", &o.other_pak, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(ids(&list).is_empty(), "another generator sees none: {list}");

    let (status, list) = call(
        &fixture,
        "GET",
        "/api/v1/work-orders",
        &fixture.admin_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let all = ids(&list);
    assert!(
        all.contains(&mine) && all.contains(&admins),
        "an admin sees each"
    );

    let (status, body) = call(&fixture, "GET", "/api/v1/work-orders", &o.agent_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn only_the_owner_or_an_admin_reads_and_deletes_a_work_order() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-read");
    let (_, mine) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    let uri = format!("/api/v1/work-orders/{}", mine["id"].as_str().unwrap());
    let (_, admins) = create_as(&fixture, &fixture.admin_pak, o.agent.id).await;
    let admins_uri = format!("/api/v1/work-orders/{}", admins["id"].as_str().unwrap());

    // Read.
    let (status, body) = call(&fixture, "GET", &uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = call(&fixture, "GET", &uri, &fixture.admin_pak, None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(&fixture, "GET", &uri, &o.other_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "work_order_not_owned");
    let (status, _) = call(&fixture, "GET", &uri, &o.agent_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // An admin's work order is an admin's alone, also when it targets the
    // generator's agent.
    let (status, body) = call(&fixture, "GET", &admins_uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "work_order_not_owned");
    let (status, body) = call(&fixture, "DELETE", &admins_uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    // Delete.
    let (status, body) = call(&fixture, "DELETE", &uri, &o.other_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "work_order_not_owned");
    let (status, _) = call(&fixture, "DELETE", &uri, &o.agent_pak, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&fixture, "GET", &uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::OK, "a refused delete deletes nothing");

    let (status, body) = call(&fixture, "DELETE", &uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (status, _) = call(&fixture, "GET", &uri, &o.owner_pak, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // An admin deletes a generator's work order.
    let (_, second) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    let second_uri = format!("/api/v1/work-orders/{}", second["id"].as_str().unwrap());
    let (status, _) = call(&fixture, "DELETE", &second_uri, &fixture.admin_pak, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn the_log_follows_the_owner() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-log");
    let (_, mine) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    let id: Uuid = mine["id"].as_str().unwrap().parse().unwrap();
    let (_, admins) = create_as(&fixture, &fixture.admin_pak, o.agent.id).await;
    let admins_id: Uuid = admins["id"].as_str().unwrap().parse().unwrap();

    // The agent runs both to their end.
    for work_order in [id, admins_id] {
        fixture
            .dal
            .work_orders()
            .claim(work_order, o.agent.id)
            .expect("claim failed");
        fixture
            .dal
            .work_orders()
            .complete_success(work_order, Some("ok".to_string()))
            .expect("complete_success failed");
    }

    // The list of the log.
    let (status, list) = call(
        &fixture,
        "GET",
        "/api/v1/work-order-log",
        &o.owner_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ids(&list), vec![id.to_string()]);
    assert_eq!(list[0]["owner_generator_id"], o.owner.id.to_string());
    let (status, list) = call(
        &fixture,
        "GET",
        "/api/v1/work-order-log",
        &o.other_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(ids(&list).is_empty());
    let (status, list) = call(
        &fixture,
        "GET",
        "/api/v1/work-order-log",
        &fixture.admin_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let all = ids(&list);
    assert!(all.contains(&id.to_string()) && all.contains(&admins_id.to_string()));
    let (status, _) = call(
        &fixture,
        "GET",
        "/api/v1/work-order-log",
        &o.agent_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // One entry of the log, and the work order route that answers from it.
    for uri in [
        format!("/api/v1/work-order-log/{id}"),
        format!("/api/v1/work-orders/{id}"),
    ] {
        let (status, body) = call(&fixture, "GET", &uri, &o.owner_pak, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
        assert_eq!(body["owner_generator_id"], o.owner.id.to_string());
        let (status, _) = call(&fixture, "GET", &uri, &fixture.admin_pak, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        let (status, body) = call(&fixture, "GET", &uri, &o.other_pak, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}");
        assert_eq!(body["code"], "work_order_not_owned", "{uri}");
        let (status, _) = call(&fixture, "GET", &uri, &o.agent_pak, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}");
    }

    // An admin's entry is not a generator's to read.
    let (status, body) = call(
        &fixture,
        "GET",
        &format!("/api/v1/work-order-log/{admins_id}"),
        &o.owner_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "work_order_not_owned");
}

#[tokio::test]
async fn a_generator_does_not_get_the_agent_routes() {
    let fixture = TestFixture::new();
    let o = owners(&fixture, "t0404-agent-routes");
    let (_, mine) = create_as(&fixture, &o.owner_pak, o.agent.id).await;
    let id = mine["id"].as_str().unwrap();

    let (status, _) = call(
        &fixture,
        "GET",
        &format!("/api/v1/agents/{}/work-orders/pending", o.agent.id),
        &o.owner_pak,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(
        &fixture,
        "POST",
        &format!("/api/v1/work-orders/{id}/claim"),
        &o.owner_pak,
        Some(serde_json::json!({ "agent_id": o.agent.id })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
