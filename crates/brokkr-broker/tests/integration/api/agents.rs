use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use brokkr_models::models::agent_annotations::NewAgentAnnotation;
use brokkr_models::models::agent_events::NewAgentEvent;
use brokkr_models::models::agent_labels::NewAgentLabel;
use brokkr_models::models::agent_targets::NewAgentTarget;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::agents::NewAgent;
use brokkr_models::models::stacks::{NewStack, Stack};
use serde_json::Value;
use std::ops::Not;
use tower::ServiceExt;
use uuid::Uuid;

async fn make_unauthorized_request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<String>,
) -> StatusCode {
    let mut request = Request::builder().method(method).uri(uri);

    if let Some(ref _b) = body {
        request = request.header("Content-Type", "application/json");
    }

    let response = app
        .oneshot(request.body(Body::from(body.unwrap_or_default())).unwrap())
        .await
        .unwrap();

    response.status()
}

#[tokio::test]
async fn test_create_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let new_agent = NewAgent::new("Test Agent".to_string(), "Test Cluster".to_string())
        .expect("Failed to create NewAgent");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agents")
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&new_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["agent"]["name"], "Test Agent");
    assert_eq!(json["agent"]["cluster_name"], "Test Cluster");
    assert!(json["initial_pak"].is_string());
}

#[tokio::test]
async fn test_get_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "Test Agent");
    assert_eq!(json["cluster_name"], "Test Cluster");
}

#[tokio::test]
async fn test_update_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create a test agent
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Prepare update payload
    let update_payload = serde_json::json!({
        "name": "Updated Agent",
        "cluster_name": "Updated Cluster",
        "status": "ACTIVE"
    });

    // Send update request
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();

    // Verify updated fields
    assert_eq!(updated_agent.name, "Updated Agent");
    assert_eq!(updated_agent.cluster_name, "Updated Cluster");
    assert_eq!(updated_agent.status, "ACTIVE");

    // Verify unchanged fields
    assert_eq!(updated_agent.id, test_agent.id);
    assert!(updated_agent.updated_at > test_agent.created_at);
    assert_eq!(updated_agent.deleted_at, None);

    // Verify that pak_hash is not returned in the response
    assert!(serde_json::to_string(&updated_agent)
        .unwrap()
        .contains("pak_hash")
        .not());

    // Fetch the agent from the database to verify the update
    let db_agent = fixture.dal.agents().get(test_agent.id).unwrap().unwrap();
    assert_eq!(db_agent, updated_agent);
}

#[tokio::test]
async fn test_delete_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let test_stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);

    let test_do = fixture.create_test_deployment_object(
        test_stack.id,
        "Test Deployment Object".to_string(),
        false,
    );

    // Create a test event
    let new_event = NewAgentEvent::new(
        test_agent.id,
        test_do.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    )
    .expect("Failed to create NewAgentEvent");

    fixture
        .dal
        .agent_events()
        .create(&new_event)
        .expect("Failed to create agent event");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/events", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let test_stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let test_do = fixture.create_test_deployment_object(
        test_stack.id,
        "Test Deployment Object".to_string(),
        false,
    );

    let new_event = NewAgentEvent::new(
        test_agent.id,
        test_do.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    )
    .expect("Failed to create NewAgentEvent");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/events", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["event_type"], "TEST_EVENT");
    assert_eq!(json["status"], "SUCCESS");
}

#[tokio::test]
async fn test_list_agent_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test label
    let new_label = NewAgentLabel::new(test_agent.id, "test_label".to_string())
        .expect("Failed to create NewAgentLabel");
    fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/labels", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json.as_array().unwrap().is_empty());
    assert_eq!(json[0]["label"], "test_label");
}

#[tokio::test]
async fn test_add_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_label = NewAgentLabel::new(test_agent.id, "new_label".to_string())
        .expect("Failed to create NewAgentLabel");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/labels", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&new_label).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["label"], "new_label");
}

#[tokio::test]
async fn test_remove_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test label
    let new_label = NewAgentLabel::new(test_agent.id, "test_label".to_string())
        .expect("Failed to create NewAgentLabel");
    fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/labels/test_label",
                    test_agent.id
                ))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_agent_annotations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test annotation
    let new_annotation = NewAgentAnnotation::new(
        test_agent.id,
        "test_key".to_string(),
        "test_value".to_string(),
    )
    .expect("Failed to create NewAgentAnnotation");
    fixture
        .dal
        .agent_annotations()
        .create(&new_annotation)
        .expect("Failed to create agent annotation");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/annotations", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json.as_array().unwrap().is_empty());
    assert_eq!(json[0]["key"], "test_key");
    assert_eq!(json[0]["value"], "test_value");
}

#[tokio::test]
async fn test_add_agent_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_annotation = NewAgentAnnotation::new(
        test_agent.id,
        "new_key".to_string(),
        "new_value".to_string(),
    )
    .expect("Failed to create NewAgentAnnotation");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/annotations", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&new_annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["key"], "new_key");
    assert_eq!(json["value"], "new_value");
}

#[tokio::test]
async fn test_remove_agent_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test annotation
    let new_annotation = NewAgentAnnotation::new(
        test_agent.id,
        "test_key".to_string(),
        "test_value".to_string(),
    )
    .expect("Failed to create NewAgentAnnotation");
    fixture
        .dal
        .agent_annotations()
        .create(&new_annotation)
        .expect("Failed to create agent annotation");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/annotations/test_key",
                    test_agent.id
                ))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_agent_targets() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, fixture.admin_generator.id)
        .expect("Failed to create NewStack");
    let stack = fixture
        .dal
        .stacks()
        .create(&new_stack)
        .expect("Failed to create stack");

    // Add a test target
    let new_target =
        NewAgentTarget::new(test_agent.id, stack.id).expect("Failed to create NewAgentTarget");
    fixture
        .dal
        .agent_targets()
        .create(&new_target)
        .expect("Failed to create agent target");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/targets", test_agent.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json.as_array().unwrap().is_empty());
    assert_eq!(json[0]["stack_id"], stack.id.to_string());
}

#[tokio::test]
async fn test_add_agent_target() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, fixture.admin_generator.id)
        .expect("Failed to create NewStack");
    let stack = fixture
        .dal
        .stacks()
        .create(&new_stack)
        .expect("Failed to create stack");

    let new_target =
        NewAgentTarget::new(test_agent.id, stack.id).expect("Failed to create NewAgentTarget");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/targets", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&new_target).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["stack_id"], stack.id.to_string());
}

#[tokio::test]
async fn test_remove_agent_target() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, fixture.admin_generator.id)
        .expect("Failed to create NewStack");
    let stack = fixture
        .dal
        .stacks()
        .create(&new_stack)
        .expect("Failed to create stack");

    // Add a test target
    let new_target =
        NewAgentTarget::new(test_agent.id, stack.id).expect("Failed to create NewAgentTarget");
    fixture
        .dal
        .agent_targets()
        .create(&new_target)
        .expect("Failed to create agent target");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/agents/{}/targets/{}",
                    test_agent.id, stack.id
                ))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_unauthorized_list_agent_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let status = make_unauthorized_request(
        app.clone(),
        "GET",
        &format!("/api/v1/agents/{}/events", test_agent.id),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_create_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_event = NewAgentEvent::new(
        test_agent.id,
        Uuid::new_v4(),
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    )
    .expect("Failed to create NewAgentEvent");

    let status = make_unauthorized_request(
        app.clone(),
        "POST",
        &format!("/api/v1/agents/{}/events", test_agent.id),
        Some(serde_json::to_string(&new_event).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_list_agent_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let status = make_unauthorized_request(
        app.clone(),
        "GET",
        &format!("/api/v1/agents/{}/labels", test_agent.id),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_add_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_label = NewAgentLabel::new(test_agent.id, "new_label".to_string())
        .expect("Failed to create NewAgentLabel");

    let status = make_unauthorized_request(
        app.clone(),
        "POST",
        &format!("/api/v1/agents/{}/labels", test_agent.id),
        Some(serde_json::to_string(&new_label).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_create_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let new_agent = NewAgent::new("Test Agent".to_string(), "Test Cluster".to_string())
        .expect("Failed to create NewAgent");

    let status = make_unauthorized_request(
        app.clone(),
        "POST",
        "/api/v1/agents",
        Some(serde_json::to_string(&new_agent).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_get_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let status = make_unauthorized_request(
        app.clone(),
        "GET",
        &format!("/api/v1/agents/{}", test_agent.id),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_update_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let updated_agent = serde_json::json!({
        "name": "Updated Agent",
        "cluster_name": "Updated Cluster"
    });

    let status = make_unauthorized_request(
        app.clone(),
        "PUT",
        &format!("/api/v1/agents/{}", test_agent.id),
        Some(serde_json::to_string(&updated_agent).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_delete_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let status = make_unauthorized_request(
        app.clone(),
        "DELETE",
        &format!("/api/v1/agents/{}", test_agent.id),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_agent_with_mismatched_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (_agent1, agent1_pak) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Cluster 1".to_string());
    let (agent2, _) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster 2".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}", agent2.id))
                .header("Authorization", format!("Bearer {}", agent1_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_agent_with_mismatched_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (_agent1, agent1_pak) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Cluster 1".to_string());
    let (agent2, _) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster 2".to_string());

    let mut updated_agent = agent2.clone();
    updated_agent.name = "Updated Agent 2".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/agents/{}", agent2.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", agent1_pak))
                .body(Body::from(serde_json::to_string(&updated_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_agent_event_with_mismatched_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (_agent1, agent1_pak) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Cluster 1".to_string());
    let (agent2, _) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster 2".to_string());

    let new_event = NewAgentEvent::new(
        agent2.id,
        Uuid::new_v4(),
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    )
    .expect("Failed to create NewAgentEvent");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/events", agent2.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", agent1_pak))
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_agent_labels_with_mismatched_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (_agent1, agent1_pak) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Cluster 1".to_string());
    let (agent2, _) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Cluster 2".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/labels", agent2.id))
                .header("Authorization", format!("Bearer {}", agent1_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_record_heartbeat() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (agent, pak) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Cluster 1".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/heartbeat", agent.id))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify that the last_heartbeat_at field has been updated
    let updated_agent = fixture.dal.agents().get(agent.id).unwrap().unwrap();
    assert!(updated_agent.last_heartbeat.is_some());
    assert!(updated_agent.last_heartbeat.unwrap() > agent.created_at);
}

#[tokio::test]
async fn test_get_target_state_incremental() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create an agent
    let (agent, agent_pak) = fixture
        .create_test_agent_with_pak("Agent Target State".to_string(), "Test Cluster".to_string());

    // Create a stack and associate the agent with it
    let stack = fixture.create_test_stack(
        "Stack Target State".to_string(),
        None,
        fixture.admin_generator.id,
    );
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Test incremental mode (default)
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/agents/{}/target-state",
                    agent.id.to_string()
                ))
                .method("GET")
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    // Check response in incremental mode - should only contain object2
    let objects = body_json.as_array().unwrap();
    assert_eq!(
        objects.len(),
        1,
        "Should only contain the undeployed object"
    );
    assert_eq!(
        objects[0]["id"].as_str().unwrap(),
        object2.id.to_string(),
        "The undeployed object should be object2"
    );
}

#[tokio::test]
async fn test_get_target_state_full() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create an agent
    let (agent, agent_pak) = fixture
        .create_test_agent_with_pak("Agent Target State".to_string(), "Test Cluster".to_string());

    // Create a stack and associate the agent with it
    let stack = fixture.create_test_stack(
        "Stack Target State".to_string(),
        None,
        fixture.admin_generator.id,
    );
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Test full mode
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/agents/{}/target-state?mode=full",
                    agent.id.to_string()
                ))
                .method("GET")
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();


    // Check response in full mode - should contain only the latest object
    let objects = body_json.as_array().unwrap();
    assert_eq!(
        objects.len(),
        1,
        "Should contain only the latest object in full mode"
    );


    // Collect IDs for easier verification
    let ids: Vec<String> = objects
        .iter()
        .map(|obj| obj["id"].as_str().unwrap().to_string())
        .collect();

    // Verify that only the latest object is included in the response
    assert!(
        !ids.contains(&object1.id.to_string()),
        "object1 should not be included in full mode as it's not the latest"
    );
    assert!(
        ids.contains(&object2.id.to_string()),
        "object2 (the latest) should be included in full mode"

    );
}

#[tokio::test]
async fn test_get_target_state_with_invalid_mode() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create an agent
    let (agent, agent_pak) = fixture
        .create_test_agent_with_pak("Agent Invalid Mode".to_string(), "Test Cluster".to_string());

    // Create a stack and associate the agent with it
    let stack = fixture.create_test_stack(
        "Stack Invalid Mode".to_string(),
        None,
        fixture.admin_generator.id,
    );
    fixture.create_test_agent_target(agent.id, stack.id);

    // Test with invalid mode parameter
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/agents/{}/target-state?mode=invalid",
                    agent.id.to_string()
                ))
                .method("GET")
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // The API should handle invalid mode gracefully - we expect it to
    // either return a 400 Bad Request or fall back to the default mode (incremental)
    // Let's check both possibilities


    if resp.status() == StatusCode::BAD_REQUEST {
        // If API returns 400, the test passes as this is a valid response for invalid parameters
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    } else {
        // If API falls back to default mode, verify we got a 200 OK
        assert_eq!(resp.status(), StatusCode::OK);


        // Check that the response contains the expected format
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body).unwrap();

        // Verify the response contains an array (we don't check the exact content
        // since we're just verifying the API handled the invalid parameter gracefully)
        assert!(body_json.is_array(), "Response should be a JSON array");
    }
}

#[tokio::test]
async fn test_get_agent_by_name_and_cluster_name() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create a test agent
    let _test_agent =
        fixture.create_test_agent("test-agent".to_string(), "test-cluster".to_string());

    // Retrieve the agent by name and cluster name
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agents/?name=test-agent&cluster_name=test-cluster")
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let agent: Agent = serde_json::from_slice(&body).unwrap();

    assert_eq!(agent.name, "test-agent");
    assert_eq!(agent.cluster_name, "test-cluster");
}

#[tokio::test]
async fn test_get_agent_stacks() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create test agents
    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("test-agent".to_string(), "test-cluster".to_string());
    let (_other_agent, other_agent_pak) =
        fixture.create_test_agent_with_pak("other-agent".to_string(), "other-cluster".to_string());

    // Create test stacks
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);
    let stack3 = fixture.create_test_stack("Stack 3".to_string(), None, generator.id);
    let stack4 = fixture.create_test_stack("Stack 4".to_string(), None, generator.id);

    // Create associations:
    // 1. Direct target
    let new_target = NewAgentTarget::new(agent.id, stack1.id).unwrap();
    fixture.dal.agent_targets().create(&new_target).unwrap();

    // 2. Label match
    fixture.create_test_agent_label(agent.id, "env=prod".to_string());
    fixture.create_test_stack_label(stack2.id, "env=prod".to_string());

    // 3. Annotation match
    fixture.create_test_agent_annotation(agent.id, "region".to_string(), "us-west".to_string());
    fixture.create_test_stack_annotation(stack3.id, "region", "us-west");

    // Test with admin PAK
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/stacks", agent.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let stacks: Vec<Stack> = serde_json::from_slice(&body).unwrap();

    // Should return stacks 1, 2, and 3 (not 4)
    assert_eq!(stacks.len(), 3);
    assert!(stacks.iter().any(|s| s.id == stack1.id)); // Direct target
    assert!(stacks.iter().any(|s| s.id == stack2.id)); // Label match
    assert!(stacks.iter().any(|s| s.id == stack3.id)); // Annotation match
    assert!(!stacks.iter().any(|s| s.id == stack4.id)); // No association

    // Test with agent's own PAK
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/stacks", agent.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("Error response body: {}", String::from_utf8_lossy(&body));
    }

    assert_eq!(status, StatusCode::OK);

    // Test with other agent's PAK (should be forbidden)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/stacks", agent.id))
                .header("Authorization", format!("Bearer {}", other_agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Test with non-existent agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/stacks", Uuid::new_v4()))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test unauthorized access
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/stacks", agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rotate_agent_pak_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (agent, _) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());
    let original_pak_hash = agent.pak_hash.clone();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/rotate-pak", agent.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["agent"].is_object());
    assert!(json["pak"].is_string());

    // Verify agent fields
    assert_eq!(json["agent"]["id"], agent.id.to_string());
    assert_eq!(json["agent"]["name"], "Test Agent");
    assert_eq!(json["agent"]["cluster_name"], "Test Cluster");

    // Verify PAK hash has changed
    let updated_agent = fixture.dal.agents().get(agent.id).unwrap().unwrap();
    assert_ne!(updated_agent.pak_hash, original_pak_hash);
}

#[tokio::test]
async fn test_rotate_agent_pak_self_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create agent with PAK
    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());
    let original_pak_hash = agent.pak_hash.clone();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/rotate-pak", agent.id))
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["agent"].is_object());
    assert!(json["pak"].is_string());

    // Verify PAK hash has changed
    let updated_agent = fixture.dal.agents().get(agent.id).unwrap().unwrap();
    assert_ne!(updated_agent.pak_hash, original_pak_hash);
}

#[tokio::test]
async fn test_rotate_agent_pak_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/rotate-pak", test_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rotate_agent_pak_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create two agents
    let (agent1, _) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Test Cluster".to_string());
    let (_agent2, agent2_pak) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Test Cluster".to_string());

    // Try to rotate agent1's PAK using agent2's PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/rotate-pak", agent1.id))
                .header("Authorization", format!("Bearer {}", agent2_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_target_state_with_mismatched_auth() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create two agents
    let (agent1, _) =
        fixture.create_test_agent_with_pak("Agent 1".to_string(), "Test Cluster".to_string());

    let (_, agent2_pak) =
        fixture.create_test_agent_with_pak("Agent 2".to_string(), "Test Cluster".to_string());

    // Create a stack and associate agent1 with it
    let stack = fixture.create_test_stack(
        "Stack Auth Test".to_string(),
        None,
        fixture.admin_generator.id,
    );
    fixture.create_test_agent_target(agent1.id, stack.id);

    // Test with agent2's auth credentials when requesting agent1's target state
    // This should be forbidden
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/agents/{}/target-state?mode=full",
                    agent1.id.to_string()
                ))
                .method("GET")
                .header("Authorization", format!("Bearer {}", agent2_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify we get a forbidden status
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
