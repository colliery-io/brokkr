/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::fixtures::TestFixture;
use serde_json::{json, Value};

#[tokio::test]
async fn test_create_diagnostic_request() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create agent and deployment object
    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Diag Test Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Diag Test Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let request_body = json!({
        "agent_id": agent.id,
        "requested_by": "admin@test.com",
        "retention_minutes": 60
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/deployment-objects/{}/diagnostics",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let diagnostic_request: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(diagnostic_request["agent_id"], agent.id.to_string());
    assert_eq!(
        diagnostic_request["deployment_object_id"],
        deployment_object.id.to_string()
    );
    assert_eq!(diagnostic_request["status"], "pending");
    assert_eq!(diagnostic_request["requested_by"], "admin@test.com");
}

#[tokio::test]
async fn test_create_diagnostic_request_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Unauth Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Unauth Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let request_body = json!({
        "agent_id": agent.id
    });

    // Try with agent PAK (should fail - requires admin)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/deployment-objects/{}/diagnostics",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_pending_diagnostics() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Pending Diag Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Pending Diag Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create a diagnostic request using admin
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Get pending as the agent
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/diagnostics/pending", agent.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let pending: Vec<Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["status"], "pending");
}

#[tokio::test]
async fn test_get_pending_diagnostics_unauthorized() {
    let fixture = TestFixture::new();

    let (agent1, _) = fixture.create_test_agent_with_pak(
        "Agent 1".to_string(),
        "Cluster 1".to_string(),
    );
    let (_, agent2_pak) = fixture.create_test_agent_with_pak(
        "Agent 2".to_string(),
        "Cluster 2".to_string(),
    );

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Agent 2 trying to get Agent 1's pending diagnostics
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/diagnostics/pending", agent1.id))
                .header("Authorization", format!("Bearer {}", agent2_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_claim_diagnostic() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Claim Diag Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Claim Diag Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create a diagnostic request
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Claim the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/diagnostics/{}/claim", request.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let claimed: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(claimed["status"], "claimed");
    assert!(!claimed["claimed_at"].is_null());
}

#[tokio::test]
async fn test_claim_already_claimed() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Already Claimed Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Already Claimed Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create and claim a diagnostic request
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    fixture.dal.diagnostic_requests().claim(request.id).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Try to claim again
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/diagnostics/{}/claim", request.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_submit_diagnostic_result() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Submit Result Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Submit Result Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create and claim a diagnostic request
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    fixture.dal.diagnostic_requests().claim(request.id).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let result_body = json!({
        "pod_statuses": r#"[{"name": "pod-1", "phase": "Running"}]"#,
        "events": r#"[{"type": "Normal", "reason": "Started"}]"#,
        "log_tails": r#"{"pod-1/container": ["line 1", "line 2"]}"#,
        "collected_at": "2025-01-15T10:30:00Z"
    });

    // Submit result
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/diagnostics/{}/result", request.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(result_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["request_id"], request.id.to_string());
    assert!(result["pod_statuses"].as_str().unwrap().contains("pod-1"));

    // Verify the request is now completed
    let updated_request = fixture
        .dal
        .diagnostic_requests()
        .get(request.id)
        .unwrap()
        .unwrap();
    assert_eq!(updated_request.status, "completed");
}

#[tokio::test]
async fn test_submit_result_not_claimed() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Not Claimed Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Not Claimed Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create request but don't claim it
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let result_body = json!({
        "pod_statuses": "[]",
        "events": "[]",
        "collected_at": "2025-01-15T10:30:00Z"
    });

    // Try to submit without claiming first
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/diagnostics/{}/result", request.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .header("Content-Type", "application/json")
                .body(Body::from(result_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_diagnostic_with_result() {
    let fixture = TestFixture::new();

    let (agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Get With Result Agent".to_string(),
        "Test Cluster".to_string(),
    );
    let stack = fixture.create_test_stack(
        "Get With Result Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create, claim, and complete a diagnostic request with result
    use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
    use brokkr_models::models::diagnostic_results::NewDiagnosticResult;
    use chrono::Utc;

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    fixture.dal.diagnostic_requests().claim(request.id).unwrap();

    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[{"name": "test-pod", "phase": "Running"}]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();
    fixture.dal.diagnostic_results().create(&new_result).unwrap();
    fixture.dal.diagnostic_requests().complete(request.id).unwrap();

    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Get the diagnostic (admin can view any)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/diagnostics/{}", request.id))
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let diagnostic: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(diagnostic["request"]["status"], "completed");
    assert!(!diagnostic["result"].is_null());
    assert!(diagnostic["result"]["pod_statuses"]
        .as_str()
        .unwrap()
        .contains("test-pod"));
}

#[tokio::test]
async fn test_get_diagnostic_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/diagnostics/{}", fake_id))
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
