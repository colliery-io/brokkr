use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};

use tower::ServiceExt;

use brokkr_models::models::deployment_objects::DeploymentObject;

// Import the TestFixture and helper functions
use crate::fixtures::TestFixture;
use crate::fixtures::{create_test_deployment_object, create_test_stack};

#[tokio::test]
async fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let created_stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, created_stack.id).await;
    assert!(!created_object.id.is_nil());
    assert_eq!(created_object.stack_id, created_stack.id);
}

#[tokio::test]
async fn test_get_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, stack.id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/deployment-objects/{}", created_object.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let retrieved_object: DeploymentObject = serde_json::from_slice(&body).unwrap();
    assert_eq!(retrieved_object.id, created_object.id);
}

#[tokio::test]
async fn test_list_deployment_objects() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, stack.id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/stacks/{}/deployment-objects", stack.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();
    assert!(objects.iter().any(|o| o.id == created_object.id));
}

#[tokio::test]
async fn test_update_deployment_object_immutability() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, stack.id).await;

    let mut updated_object = created_object.clone();
    updated_object.yaml_content = "updated: content".to_string();

    println!(
        "Sending update request for deployment object: {:?}",
        updated_object
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("/deployment-objects/{}", created_object.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    println!("Update response status: {}", status);
    println!("Update response body: {}", body_str);

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "Expected update to be rejected"
    );
    assert!(
        body_str.contains("cannot be modified"),
        "Error message should indicate that modification is not allowed"
    );
}

#[tokio::test]
async fn test_soft_delete_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, stack.id).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/deployment-objects/{}", created_object.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify the object is soft deleted
    let get_deleted_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/deployment-objects/{}", created_object.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_deleted_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_active_deployment_objects() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack = create_test_stack(&app).await;
    let created_object = create_test_deployment_object(&app, stack.id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/deployment-objects/active")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let active_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();
    assert!(active_objects.iter().any(|o| o.id == created_object.id));
}
