use std::usize;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;
use uuid::Uuid;

use brokkr_models::models::{
    stacks::{Stack, NewStack},
    deployment_objects::{NewDeploymentObject, DeploymentObject},
    stack_labels::StackLabel,
    stack_annotations::StackAnnotation,
};

use crate::fixtures::TestFixture;



#[tokio::test]
async fn test_list_stacks() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let response = app
        .oneshot(Request::builder().uri("/stacks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let stacks: Vec<Stack> = serde_json::from_slice(&body).unwrap();
    
    // Add assertions about the returned stacks
}

#[tokio::test]
async fn test_create_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let new_stack = NewStack {
        name: "Test Stack".to_string(),
        description: Some("A test stack".to_string()),
    };

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

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let created_stack: Stack = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_stack.name, new_stack.name);
    assert_eq!(created_stack.description, new_stack.description);
}

#[tokio::test]
async fn test_get_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("To Be Hard Deleted".to_string(), None);

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let stack: Stack = serde_json::from_slice(&body).unwrap();

    assert_eq!(stack.id, test_stack.id);
}

#[tokio::test]
async fn test_update_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Create a test stack
    let original_stack = fixture.create_test_stack("Original Stack Name".to_string(), Some("Original description".to_string()));

    let update_data = json!({
        "name": "Updated Stack Name",
        "description": "Updated description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/stacks/{}", original_stack.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_stack: Stack = serde_json::from_slice(&body).unwrap();

    // Assert that the stack was updated correctly
    assert_eq!(updated_stack.id, original_stack.id);
    assert_eq!(updated_stack.name, "Updated Stack Name");
    assert_eq!(updated_stack.description, Some("Updated description".to_string()));

    // Verify the update in the database
    let db_stack = fixture.dal.stacks().get(original_stack.id).unwrap().unwrap();
    assert_eq!(db_stack.name, "Updated Stack Name");
    assert_eq!(db_stack.description, Some("Updated description".to_string()));
}

#[tokio::test]
async fn test_delete_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack_id = Uuid::new_v4(); // Assume this ID exists in the test database

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
}

#[tokio::test]
async fn test_list_deployment_objects() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack_id = Uuid::new_v4(); // Assume this ID exists in the test database

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/deployment-objects", stack_id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deployment_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();

    // Add assertions about the returned deployment objects
}

#[tokio::test]
async fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), Some("Test Description".to_string()));

    let new_deployment_object = NewDeploymentObject::new(
        test_stack.id,
        "test yaml content".to_string(),
        false,
    ).expect("Failed to create NewDeploymentObject");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/stacks/{}/deployment-objects", test_stack.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_deployment_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_object: DeploymentObject = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_object.stack_id, test_stack.id);
    assert_eq!(created_object.yaml_content, new_deployment_object.yaml_content);
}

#[tokio::test]
async fn test_list_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack_id = Uuid::new_v4(); // Assume this ID exists in the test database

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/labels", stack_id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let labels: Vec<StackLabel> = serde_json::from_slice(&body).unwrap();

    // Add assertions about the returned labels
}

#[tokio::test]
async fn test_add_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), Some("Test Description".to_string()));

    let new_label = "test-label";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/stacks/{}/labels", test_stack.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_label).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_label: StackLabel = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_label.stack_id, test_stack.id);
    assert_eq!(created_label.label, new_label);
}


#[tokio::test]
async fn test_remove_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("To Be Hard Deleted".to_string(), None);
    let label = fixture.create_test_stack_label(test_stack.id, "test-label".to_string());
    

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/stacks/{}/labels/{}", test_stack.id, label.label))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the label is removed
    let labels = fixture.dal.stack_labels().list_for_stack(test_stack.id).unwrap();
    assert!(!labels.iter().any(|l| l.id == label.id));
}

#[tokio::test]
async fn test_list_annotations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let stack_id = Uuid::new_v4(); // Assume this ID exists in the test database

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/annotations", stack_id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let annotations: Vec<StackAnnotation> = serde_json::from_slice(&body).unwrap();

    // Add assertions about the returned annotations
}

#[tokio::test]
async fn test_add_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), Some("Test Description".to_string()));

    // Changed from json!() to a tuple, as the API might expect
    let new_annotation = ("test-key", "test-value");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/stacks/{}/annotations", test_stack.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let created_annotation: StackAnnotation = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_annotation.stack_id, test_stack.id);
    assert_eq!(created_annotation.key, new_annotation.0);
    assert_eq!(created_annotation.value, new_annotation.1);
}

#[tokio::test]
async fn test_remove_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Create a test stack
    let test_stack = fixture.create_test_stack("Test Stack for Annotation".to_string(), None);
    
    // Create a test annotation
    let annotation = fixture.create_test_stack_annotation(test_stack.id, "test-key", "test-value");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/stacks/{}/annotations/{}", test_stack.id, annotation.key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the annotation is removed
    let annotations = fixture.dal.stack_annotations().list_for_stack(test_stack.id).unwrap();
    assert!(!annotations.iter().any(|a| a.id == annotation.id));
}