use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use crate::fixtures::TestFixture;
use brokkr_models::models::agents::NewAgent;
use brokkr_models::models::agent_events::NewAgentEvent;
use brokkr_models::models::agent_labels::NewAgentLabel;
use brokkr_models::models::agent_annotations::NewAgentAnnotation;
use brokkr_models::models::agent_targets::NewAgentTarget;
use brokkr_models::models::stacks::NewStack;
use serde_json;
use uuid::Uuid;

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let mut updated_agent = test_agent.clone();
    updated_agent.name = "Updated Agent".to_string();
    updated_agent.cluster_name = "Updated Cluster".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/agents/{}", test_agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", admin_pak)
                .body(Body::from(serde_json::to_string(&updated_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "Updated Agent");
    assert_eq!(json["cluster_name"], "Updated Cluster");
}

#[tokio::test]
async fn test_delete_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test event
    let new_event = NewAgentEvent::new(
        test_agent.id,
        Uuid::new_v4(), // Assuming a random UUID for deployment_object_id
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");

    fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

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

    assert!(json.as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_event = NewAgentEvent::new(
        test_agent.id,
        Uuid::new_v4(), // Assuming a random UUID for deployment_object_id
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test label
    let new_label = NewAgentLabel::new(test_agent.id, "test_label".to_string())
        .expect("Failed to create NewAgentLabel");
    fixture.dal.agent_labels().create(&new_label).expect("Failed to create agent label");

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

    assert!(json.as_array().unwrap().len() > 0);
    assert_eq!(json[0]["label"], "test_label");
}

#[tokio::test]
async fn test_add_agent_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test label
    let new_label = NewAgentLabel::new(test_agent.id, "test_label".to_string())
        .expect("Failed to create NewAgentLabel");
    fixture.dal.agent_labels().create(&new_label).expect("Failed to create agent label");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}/labels/test_label", test_agent.id))
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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test annotation
    let new_annotation = NewAgentAnnotation::new(test_agent.id, "test_key".to_string(), "test_value".to_string())
        .expect("Failed to create NewAgentAnnotation");
    fixture.dal.agent_annotations().create(&new_annotation).expect("Failed to create agent annotation");

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

    assert!(json.as_array().unwrap().len() > 0);
    assert_eq!(json[0]["key"], "test_key");
    assert_eq!(json[0]["value"], "test_value");
}

#[tokio::test]
async fn test_add_agent_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_annotation = NewAgentAnnotation::new(test_agent.id, "new_key".to_string(), "new_value".to_string())
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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Add a test annotation
    let new_annotation = NewAgentAnnotation::new(test_agent.id, "test_key".to_string(), "test_value".to_string())
        .expect("Failed to create NewAgentAnnotation");
    fixture.dal.agent_annotations().create(&new_annotation).expect("Failed to create agent annotation");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}/annotations/test_key", test_agent.id))
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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, Uuid::new_v4())
        .expect("Failed to create NewStack");
    let stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    // Add a test target
    let new_target = NewAgentTarget::new(test_agent.id, stack.id)
        .expect("Failed to create NewAgentTarget");
    fixture.dal.agent_targets().create(&new_target).expect("Failed to create agent target");

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

    assert!(json.as_array().unwrap().len() > 0);
    assert_eq!(json[0]["stack_id"], stack.id.to_string());
}

#[tokio::test]
async fn test_add_agent_target() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, Uuid::new_v4())
        .expect("Failed to create NewStack");
    let stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    let new_target = NewAgentTarget::new(test_agent.id, stack.id)
        .expect("Failed to create NewAgentTarget");

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

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a test stack
    let new_stack = NewStack::new("Test Stack".to_string(), None, Uuid::new_v4())
        .expect("Failed to create NewStack");
    let stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    // Add a test target
    let new_target = NewAgentTarget::new(test_agent.id, stack.id)
        .expect("Failed to create NewAgentTarget");
    fixture.dal.agent_targets().create(&new_target).expect("Failed to create agent target");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}/targets/{}", test_agent.id, stack.id))
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

