use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use brokkr_broker::api;
use brokkr_models::models::stacks::NewStack;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_stacks() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    fixture.create_test_stack();

    let response = app
        .oneshot(Request::builder().uri("/stacks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let stacks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert!(!stacks.is_empty());
}

#[tokio::test]
async fn test_create_stack() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        Some(vec!["test".to_string()]),
        Some(vec![("key".to_string(), "value".to_string())]),
        Some(vec!["agent1".to_string()]),
    )
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/stacks")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created_stack: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_stack["name"], "Test Stack");
}

#[tokio::test]
async fn test_get_stack() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/stacks/{}", stack_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let fetched_stack: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(fetched_stack["id"], stack_id.to_string());
}

#[tokio::test]
async fn test_update_stack() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();

    let updated_stack = json!({
        "id": stack_id,
        "name": "Updated Stack",
        "description": "Updated Description",
        "labels": ["updated"],
        "annotations": {"key": "updated_value"},
        "agent_target": ["updated_agent"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/stacks/{}", stack_id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_stack: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_stack["name"], "Updated Stack");
}

#[tokio::test]
async fn test_delete_stack() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/stacks/{}", stack_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the stack is soft-deleted
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/stacks/{}", stack_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_active_stacks() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    fixture.create_test_stack();
    let deleted_stack_id = fixture.create_test_stack();

    // Delete one stack
    app.oneshot(
        Request::builder()
            .method("DELETE")
            .uri(format!("/stacks/{}", deleted_stack_id))
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    let response = app
        .oneshot(Request::builder().uri("/stacks/active").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let active_stacks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(active_stacks.len(), 1);
    assert!(active_stacks.iter().all(|stack| stack["id"] != deleted_stack_id.to_string()));
}