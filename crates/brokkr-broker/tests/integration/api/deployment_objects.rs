use std::usize;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use brokkr_models::models::deployment_objects::DeploymentObject;
use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_list_deployment_objects_by_agent() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let test_stack1 = fixture.create_test_stack("Test Stack 1".to_string(), None);
    let test_stack2 = fixture.create_test_stack("Test Stack 2".to_string(), None);
    
    fixture.create_test_agent_target(test_agent.id, test_stack1.id);
    fixture.create_test_agent_target(test_agent.id, test_stack2.id);

    let object1 = fixture.create_test_deployment_object(test_stack1.id, "yaml_content: object1".to_string(), false);
    let object2 = fixture.create_test_deployment_object(test_stack2.id, "yaml_content: object2".to_string(), false);
    
    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/deployment-objects?agent_id={}", test_agent.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deployment_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(deployment_objects.len(), 2);
    assert!(deployment_objects.iter().any(|obj| obj.id == object1.id));
    assert!(deployment_objects.iter().any(|obj| obj.id == object2.id));
}

#[tokio::test]
async fn test_list_deployment_objects_by_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    
    let object1 = fixture.create_test_deployment_object(test_stack.id, "yaml_content: object1".to_string(), false);
    let object2 = fixture.create_test_deployment_object(test_stack.id, "yaml_content: object2".to_string(), false);

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/deployment-objects?stack_id={}", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deployment_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(deployment_objects.len(), 2);
    assert!(deployment_objects.iter().any(|obj| obj.id == object1.id));
    assert!(deployment_objects.iter().any(|obj| obj.id == object2.id));
}

#[tokio::test]
async fn test_list_deployment_objects_bad_request() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    // Test with no parameters
    let response = app.clone()
        .oneshot(Request::builder().uri("/api/v1/deployment-objects").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test with both agent_id and stack_id
    let test_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/deployment-objects?agent_id={}&stack_id={}", test_agent.id, test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let test_object = fixture.create_test_deployment_object(test_stack.id, "yaml_content: test_object".to_string(), false);

    let response = app
        .oneshot(Request::builder().uri(format!("/api/v1/deployment-objects/{}", test_object.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let deployment_object: DeploymentObject = serde_json::from_slice(&body).unwrap();

    assert_eq!(deployment_object.id, test_object.id);
    assert_eq!(deployment_object.yaml_content, "yaml_content: test_object");
    assert!(!deployment_object.is_deletion_marker);
}

#[tokio::test]
async fn test_delete_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router();

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let test_object = fixture.create_test_deployment_object(test_stack.id, "yaml_content: to_be_deleted".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/deployment-objects/{}", test_object.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the deployment object is soft deleted
    let deleted_object = fixture.dal.deployment_objects().get_including_deleted(test_object.id).unwrap().unwrap();
    assert!(deleted_object.deleted_at.is_some());
}