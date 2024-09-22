use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use brokkr_models::models::agent_events::NewAgentEvent;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_list_agent_events_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create a test agent and event
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_event(&agent, &deployment_object, "TEST", "SUCCESS", Some("Test message"));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agent-events")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_list_agent_events_unauthorized_non_existent_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agent-events")
                .header("Authorization", "Bearer non_existent_pak")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_agent_events_unauthorized_no_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agent-events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn test_create_agent_event_unauthorized_non_existent_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let new_event = NewAgentEvent {
        agent_id: Uuid::new_v4(),
        deployment_object_id: Uuid::new_v4(),
        event_type: "TEST".to_string(),
        status: "SUCCESS".to_string(),
        message: Some("Test message".to_string()),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agent-events")
                .header("Authorization", "Bearer non_existent_pak")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_agent_event_unauthorized_no_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let new_event = NewAgentEvent {
        agent_id: Uuid::new_v4(),
        deployment_object_id: Uuid::new_v4(),
        event_type: "TEST".to_string(),
        status: "SUCCESS".to_string(),
        message: Some("Test message".to_string()),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agent-events")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_agent_event_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    let event = fixture.create_test_agent_event(&agent, &deployment_object, "TEST", "SUCCESS", Some("Test message"));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agent-events/{}", event.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_event: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_event["id"], event.id.to_string());
}

#[tokio::test]
async fn test_get_agent_event_unauthorized_non_existent_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agent-events/{}", Uuid::new_v4()))
                .header("Authorization", "Bearer non_existent_pak")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_agent_event_unauthorized_no_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agent-events/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_agent_event_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let non_existent_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agent-events/{}", non_existent_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}