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

    // Create multiple test stacks
    let stack1 = fixture.create_test_stack("Test Stack 1".to_string(), Some("Description 1".to_string()));
    let stack2 = fixture.create_test_stack("Test Stack 2".to_string(), Some("Description 2".to_string()));
    let stack3 = fixture.create_test_stack("Test Stack 3".to_string(), None);

    let response = app
        .oneshot(Request::builder().uri("/stacks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let stacks: Vec<Stack> = serde_json::from_slice(&body).unwrap();
    
    // Verify that at least our created stacks are returned
    assert!(stacks.len() >= 3, "Expected at least 3 stacks, got {}", stacks.len());

    // Helper function to find a stack by ID
    let find_stack = |id: Uuid| stacks.iter().find(|s| s.id == id);

    // Verify stack1
    let found_stack1 = find_stack(stack1.id).expect("Stack 1 not found in the response");
    assert_eq!(found_stack1.name, "Test Stack 1");
    assert_eq!(found_stack1.description, Some("Description 1".to_string()));

    // Verify stack2
    let found_stack2 = find_stack(stack2.id).expect("Stack 2 not found in the response");
    assert_eq!(found_stack2.name, "Test Stack 2");
    assert_eq!(found_stack2.description, Some("Description 2".to_string()));

    // Verify stack3
    let found_stack3 = find_stack(stack3.id).expect("Stack 3 not found in the response");
    assert_eq!(found_stack3.name, "Test Stack 3");
    assert_eq!(found_stack3.description, None);

    // Verify that all returned stacks have non-nil IDs and non-empty names
    for stack in &stacks {
        assert!(!stack.id.is_nil(), "Stack has a nil UUID");
        assert!(!stack.name.is_empty(), "Stack has an empty name");
    }
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

    // Create a test stack
    let test_stack = fixture.create_test_stack("Test Stack for Deployment Objects".to_string(), None);

    // Create multiple deployment objects for the stack
    let object1 = fixture.create_test_deployment_object(
        test_stack.id,
        "yaml_content: object1".to_string(),
        false
    );
    let object2 = fixture.create_test_deployment_object(
        test_stack.id,
        "yaml_content: object2".to_string(),
        false
    );
    let object3 = fixture.create_test_deployment_object(
        test_stack.id,
        "yaml_content: deletion_marker".to_string(),
        true // This one is a deletion marker
    );

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/deployment-objects", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let deployment_objects: Vec<DeploymentObject> = serde_json::from_slice(&body).unwrap();

    // Verify that the correct number of deployment objects are returned
    assert_eq!(deployment_objects.len(), 3, "Expected 3 deployment objects, got {}", deployment_objects.len());

    // Helper function to find a deployment object by ID
    let find_object = |id: Uuid| deployment_objects.iter().find(|obj| obj.id == id);

    // Verify object1
    let found_object1 = find_object(object1.id).expect("Deployment object 1 not found in the response");
    assert_eq!(found_object1.yaml_content, "yaml_content: object1");
    assert!(!found_object1.is_deletion_marker);

    // Verify object2
    let found_object2 = find_object(object2.id).expect("Deployment object 2 not found in the response");
    assert_eq!(found_object2.yaml_content, "yaml_content: object2");
    assert!(!found_object2.is_deletion_marker);

    // Verify object3 (deletion marker)
    let found_object3 = find_object(object3.id).expect("Deployment object 3 (deletion marker) not found in the response");
    assert_eq!(found_object3.yaml_content, "yaml_content: deletion_marker");
    assert!(found_object3.is_deletion_marker);

    // Verify that all returned deployment objects belong to the correct stack
    for obj in &deployment_objects {
        assert_eq!(obj.stack_id, test_stack.id, 
                   "Deployment object {} does not belong to the correct stack", obj.id);
    }

    // Verify that the deployment objects are returned in descending order of sequence_id
    let sequence_ids: Vec<i64> = deployment_objects.iter().map(|obj| obj.sequence_id).collect();
    assert!(sequence_ids.windows(2).all(|w| w[0] >= w[1]), 
            "Deployment objects are not in descending order of sequence_id");
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

    let test_stack = fixture.create_test_stack("Test Stack".to_string(), Some("Test Description".to_string()));
     
    // Create multiple labels
    let label1 = fixture.create_test_stack_label(test_stack.id, "test-label-1".to_string());
    let label2 = fixture.create_test_stack_label(test_stack.id, "test-label-2".to_string());

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/labels", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let labels: Vec<StackLabel> = serde_json::from_slice(&body).unwrap();

    // Verify that the returned labels match the ones we created
    assert!(labels.len() >= 2);
    assert!(labels.iter().any(|l| l.id == label1.id && l.label == "test-label-1"));
    assert!(labels.iter().any(|l| l.id == label2.id && l.label == "test-label-2"));

    // Verify that all returned labels belong to the correct stack
    for label in labels {
        assert_eq!(label.stack_id, test_stack.id);
    }
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

    // Create a test stack
    let test_stack = fixture.create_test_stack("Test Stack for Annotations".to_string(), None);

    // Create multiple annotations for the stack
    let annotation1 = fixture.create_test_stack_annotation(test_stack.id, "key1", "value1");
    let annotation2 = fixture.create_test_stack_annotation(test_stack.id, "key2", "value2");

    let response = app
        .oneshot(Request::builder().uri(format!("/stacks/{}/annotations", test_stack.id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let annotations: Vec<StackAnnotation> = serde_json::from_slice(&body).unwrap();

    // Verify that the correct number of annotations are returned
    assert_eq!(annotations.len(), 2, "Expected 2 annotations, got {}", annotations.len());

    // Verify that the returned annotations match the ones we created
    assert!(annotations.iter().any(|a| a.id == annotation1.id && a.key == "key1" && a.value == "value1"),
            "Annotation 1 not found in the response");
    assert!(annotations.iter().any(|a| a.id == annotation2.id && a.key == "key2" && a.value == "value2"),
            "Annotation 2 not found in the response");

    // Verify that all returned annotations belong to the correct stack
    for annotation in annotations {
        assert_eq!(annotation.stack_id, test_stack.id, 
                   "Annotation {} does not belong to the correct stack", annotation.id);
    }
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