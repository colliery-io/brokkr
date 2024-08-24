use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fixtures::TestFixture;
use crate::fixtures::{create_test_stack, create_test_agent, create_test_deployment_object, create_test_agent_event};

#[tokio::test]
async fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let agent = create_test_agent(&app).await;
    let deployment_object = create_test_deployment_object(&app, stack.id).await;

    let created_event = create_test_agent_event(&app, agent.id, deployment_object.id).await;
    
    assert!(!created_event.id.is_nil());
    assert_eq!(created_event.event_type, "TEST_EVENT");
    assert_eq!(created_event.status, "SUCCESS");
    assert_eq!(created_event.agent_id, agent.id);
    assert_eq!(created_event.deployment_object_id, deployment_object.id);
}

#[tokio::test]
async fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let agent = create_test_agent(&app).await;
    let deployment_object = create_test_deployment_object(&app, stack.id).await;
    let created_event = create_test_agent_event(&app, agent.id, deployment_object.id).await;

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agent-events/{}", created_event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
    let retrieved_event: AgentEvent = serde_json::from_slice(&body).unwrap();
    assert_eq!(retrieved_event.id, created_event.id);
}

#[tokio::test]
async fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let agent = create_test_agent(&app).await;
    let deployment_object = create_test_deployment_object(&app, stack.id).await;
    let created_event = create_test_agent_event(&app, agent.id, deployment_object.id).await;

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agent-events?stack_id={}&agent_id={}", stack.id, agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
    let events: Vec<AgentEvent> = serde_json::from_slice(&body).unwrap();
    assert!(events.iter().any(|e| e.id == created_event.id));
}

#[tokio::test]
async fn test_soft_delete_agent_event() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let agent = create_test_agent(&app).await;
    let deployment_object = create_test_deployment_object(&app, stack.id).await;
    let created_event = create_test_agent_event(&app, agent.id, deployment_object.id).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/agent-events/{}", created_event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify the event is soft deleted
    let get_deleted_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agent-events/{}", created_event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
}