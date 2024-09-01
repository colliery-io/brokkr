use std::usize;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

use brokkr_models::models::agent_events::AgentEvent;
use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let test_deployment_object = fixture.create_test_deployment_object(test_stack.id, "yaml_content: test".to_string(), false);

    let event1 = fixture.create_test_agent_event(&test_agent, &test_deployment_object, "TEST_EVENT", "SUCCESS", Some("Test message 1"));
    let event2 = fixture.create_test_agent_event(&test_agent, &test_deployment_object, "TEST_EVENT", "FAILURE", Some("Test message 2"));

    // Test listing all events for an agent
    let response = app.clone()
        .oneshot(Request::builder().uri(format!("/api/v1/agents/{}/events", test_agent.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let events: Vec<AgentEvent> = serde_json::from_slice(&body).unwrap();

    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|e| e.id == event1.id));
    assert!(events.iter().any(|e| e.id == event2.id));

    // Test filtering by status
    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/agents/{}/events?status=SUCCESS", test_agent.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let events: Vec<AgentEvent> = serde_json::from_slice(&body).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event1.id);
}

#[tokio::test]
async fn test_list_stack_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent1 = fixture.create_test_agent("Test Agent 1".to_string(), "Test Cluster".to_string());
    let test_agent2 = fixture.create_test_agent("Test Agent 2".to_string(), "Test Cluster".to_string());
    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let test_deployment_object = fixture.create_test_deployment_object(test_stack.id, "yaml_content: test".to_string(), false);

    let event1 = fixture.create_test_agent_event(&test_agent1, &test_deployment_object, "TEST_EVENT", "SUCCESS", Some("Test message 1"));
    let event2 = fixture.create_test_agent_event(&test_agent2, &test_deployment_object, "TEST_EVENT", "FAILURE", Some("Test message 2"));

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/stacks/{}/events", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let events: Vec<AgentEvent> = serde_json::from_slice(&body).unwrap();

    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|e| e.id == event1.id));
    assert!(events.iter().any(|e| e.id == event2.id));
}

#[tokio::test]
async fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let test_deployment_object = fixture.create_test_deployment_object(test_stack.id, "yaml_content: test".to_string(), false);

    let event = fixture.create_test_agent_event(&test_agent, &test_deployment_object, "TEST_EVENT", "SUCCESS", Some("Test message"));

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/agents/{}/events/{}", test_agent.id, event.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let retrieved_event: AgentEvent = serde_json::from_slice(&body).unwrap();

    assert_eq!(retrieved_event.id, event.id);
    assert_eq!(retrieved_event.agent_id, test_agent.id);
    assert_eq!(retrieved_event.deployment_object_id, test_deployment_object.id);
    assert_eq!(retrieved_event.event_type, "TEST_EVENT");
    assert_eq!(retrieved_event.status, "SUCCESS");
    assert_eq!(retrieved_event.message, Some("Test message".to_string()));
}

#[tokio::test]
async fn test_get_nonexistent_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let nonexistent_id = Uuid::new_v4();

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/agents/{}/events/{}", test_agent.id, nonexistent_id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}