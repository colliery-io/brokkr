use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use brokkr_models::models::stacks::{Stack, NewStack};
use tower::ServiceExt;
use uuid::Uuid;
use serde_json::json;

// Import the TestFixture
use crate::fixtures::TestFixture;
use crate::fixtures::create_test_stack;


#[tokio::test]
async fn test_create_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;
    assert!(!created_stack.id.is_nil());
    assert_eq!(created_stack.name, "Test Stack");
}

#[tokio::test]
async fn test_get_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/stacks/{}", created_stack.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
    let retrieved_stack: Stack = serde_json::from_slice(&body).unwrap();
    assert_eq!(retrieved_stack.id, created_stack.id);
}

#[tokio::test]
async fn test_list_stacks() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/stacks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
    let stacks: Vec<Stack> = serde_json::from_slice(&body).unwrap();
    assert!(stacks.iter().any(|s| s.id == created_stack.id));
}

#[tokio::test]
async fn test_update_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;

    let mut updated_stack = created_stack.clone();
    updated_stack.name = "Updated Stack".to_string();

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("/stacks/{}", created_stack.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(update_response.into_body()).await.unwrap();
    let updated_stack: Stack = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_stack.name, "Updated Stack");
}

#[tokio::test]
async fn test_delete_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/stacks/{}", created_stack.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify the stack is soft deleted (not returned in list)
    let list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/stacks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
    let stacks: Vec<Stack> = serde_json::from_slice(&body).unwrap();
    assert!(!stacks.iter().any(|s| s.id == created_stack.id));
}