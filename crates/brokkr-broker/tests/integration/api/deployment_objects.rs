use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use brokkr_broker::api;
use brokkr_models::models::deployment_objects::NewDeploymentObject;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_deployment_objects() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    fixture.create_test_deployment_object(stack_id);

    let response = app
        .oneshot(Request::builder().uri("/deployment_objects").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let objects: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert!(!objects.is_empty());
}

#[tokio::test]
async fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    let new_deployment_object = NewDeploymentObject::new(
        stack_id,
        "key: value".to_string(),
        "checksum123".to_string(),
        1,
        false,
    )
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/deployment_objects")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_deployment_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created_object: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_object["yaml_content"], "key: value");
}

#[tokio::test]
async fn test_get_deployment_object() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    let deployment_object = fixture.create_test_deployment_object(stack_id);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/deployment_objects/{}", deployment_object.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(fetched_object["uuid"], deployment_object.uuid.to_string());
}

#[tokio::test]
async fn test_update_deployment_object() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    let deployment_object = fixture.create_test_deployment_object(stack_id);

    let updated_object = json!({
        "uuid": deployment_object.uuid,
        "stack_id": stack_id,
        "yaml_content": "updated: content",
        "yaml_checksum": "updated_checksum",
        "sequence_id": deployment_object.sequence_id,
        "is_deletion_marker": false,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/deployment_objects/{}", deployment_object.uuid))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // The update should fail due to the immutability constraint
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_deployment_object() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    let deployment_object = fixture.create_test_deployment_object(stack_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/deployment_objects/{}", deployment_object.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the object is soft-deleted
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/deployment_objects/{}", deployment_object.uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_deployment_objects_by_stack() {
    let fixture = TestFixture::new();
    let app = api::create_router(fixture.dal.clone());

    let stack_id = fixture.create_test_stack();
    fixture.create_test_deployment_object(stack_id);
    fixture.create_test_deployment_object(stack_id);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/deployment_objects/stack/{}", stack_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let objects: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(objects.len(), 2);
    assert!(objects.iter().all(|obj| obj["stack_id"] == stack_id.to_string()));
}