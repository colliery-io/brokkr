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
use brokkr_models::models::stacks::NewStack;
use brokkr_models::models::deployment_objects::DeploymentObject;
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
async fn test_get_applicable_deployment_objects() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    
    // Create an agent
    let (agent, pak) = fixture.create_test_agent_with_pak("Test Agent".to_string(), "Test Cluster".to_string());
    // Create a stack
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create 4 deployment objects
    let do1 = fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let do2 = fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);
    let do3 = fixture.create_test_deployment_object(stack.id, "yaml_content: object3".to_string(), false);
    let do4 = fixture.create_test_deployment_object(stack.id, "yaml_content: object4".to_string(), false);

    // Create 3 agent events: 1 success, 1 failure, 1 success
    fixture.create_test_agent_event(&agent, &do1, "DEPLOY", "SUCCESS", None);
    fixture.create_test_agent_event(&agent, &do2, "DEPLOY", "FAILURE", None);
    fixture.create_test_agent_event(&agent, &do3, "DEPLOY", "SUCCESS", None);

    // Get applicable deployment objects
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/applicable-deployment-objects", agent.id))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();

    // It should match the last deployment object (do4)
    assert_eq!(fetched_objects.len(), 1);
    assert!(fetched_objects.iter().any(|obj| obj.id == do4.id));
}

#[tokio::test]
async fn test_get_agent_by_name_and_cluster_name() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create a test agent
    let _test_agent = fixture.create_test_agent("test-agent".to_string(), "test-cluster".to_string());

 

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