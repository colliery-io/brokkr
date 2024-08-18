use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use brokkr_broker::api;
use brokkr_models::models::agent_events::NewAgentEvent;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();
    let deployment_object = fixture.create_test_deployment_object(fixture.create_test_stack());
    fixture.create_test_agent_event(agent.uuid, deployment_object.uuid);

    let response = app
        .oneshot(Request::builder().uri("/agent_events").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();
    let deployment_object = fixture.create_test_deployment_object(fixture.create_test_stack());

    let new_event = NewAgentEvent::new(
        agent.uuid,
        deployment_object.uuid,
        "test_event".to_string(),
        "success".to_string(),
        Some("Test message".to_string()),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agent_events")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created_event: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_event["event_type"], "test_event");
}

#[tokio::test]
async fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();
    let deployment_object = fixture.create_test_deployment_object(fixture.create_test_stack());
    let event = fixture.create_test_agent_event(agent.uuid, deployment_object.uuid);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/agent_events/{}", event.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let fetched_event: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(fetched_event["uuid"], event.uuid.to_string());
}

#[tokio::test]
async fn test_delete_agent_event() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let agent = fixture.create_test_agent();
    let deployment_object = fixture.create_test_deployment_object(fixture.create_test_stack());
    let event = fixture.create_test_agent_event(agent.uuid, deployment_object.uuid);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/agent_events/{}", event.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the event is deleted
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/agent_events/{}", event.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let new_event = AgentEvent {
        uuid: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        deployment_object_id: Uuid::new_v4(),
        event_type: "test".to_string(),
        status: "success".to_string(),
        message: Some("Test event".to_string()),
    };

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agent_events")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created_event: AgentEvent = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_event.uuid, new_event.uuid);
    assert_eq!(created_event.event_type, "test");
}

#[tokio::test]
async fn test_delete_agent_event() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    // First, create an event to delete
    let event = AgentEvent {
        uuid: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        deployment_object_id: Uuid::new_v4(),
        event_type: "test".to_string(),
        status: "success".to_string(),
        message: Some("Test event".to_string()),
    };
    fixture.dal.agent_events().create(&event).unwrap();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/agent_events/{}", event.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the event is deleted
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/agent_events/{}", event.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}