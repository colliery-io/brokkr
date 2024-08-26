use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use brokkr_models::models::agents::Agent;

use tower::ServiceExt;

// Import the TestFixture
use crate::fixtures::create_test_agent;
use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_create_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;
    assert!(!created_agent.id.is_nil());
    assert_eq!(created_agent.name, "Test Agent");
}

#[tokio::test]
async fn test_get_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agents/{}", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let retrieved_agent: Agent = serde_json::from_slice(&body).unwrap();
    assert_eq!(retrieved_agent.id, created_agent.id);
}

#[tokio::test]
async fn test_list_agents() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/agents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let agents: Vec<Agent> = serde_json::from_slice(&body).unwrap();
    assert!(agents.iter().any(|a| a.id == created_agent.id));
}

#[tokio::test]
async fn test_update_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;

    let mut updated_agent = created_agent.clone();
    updated_agent.name = "Updated Agent".to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("/agents/{}", created_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_agent.name, "Updated Agent");
}

#[tokio::test]
async fn test_update_agent_heartbeat() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;

    let heartbeat_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("/agents/{}/heartbeat", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(heartbeat_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_agent_status() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Create a test agent
    let created_agent = create_test_agent(&app).await;

    // Get the created agent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agents/{}", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let mut agent: Agent = serde_json::from_slice(&body).unwrap();

    // Update the agent's status
    agent.status = "ACTIVE".to_string();

    // Send the updated agent back
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("/agents/{}", created_agent.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_response.status(), StatusCode::OK);

    // Verify the update
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agents/{}", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_agent: Agent = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_agent.status, "ACTIVE");
}

#[tokio::test]
async fn test_soft_delete_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_agent = create_test_agent(&app).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/agents/{}", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify the agent is soft deleted
    let get_deleted_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/agents/{}", created_agent.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
}
