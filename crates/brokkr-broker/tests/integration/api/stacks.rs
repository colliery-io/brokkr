use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use brokkr_models::models::{
    stack_annotations::NewStackAnnotation,
    stack_labels::NewStackLabel,
    stacks::{NewStack, Stack},
    deployment_objects::NewDeploymentObject,
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fixtures::TestFixture;
use brokkr_broker::utils::pak::create_pak;

#[tokio::test]
async fn test_create_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        generator.id,
    )
    .expect("Failed to create NewStack");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stacks")
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "Test Stack");
    assert_eq!(json["description"], "Test Description");
    assert_eq!(json["generator_id"], generator.id.to_string());
}

#[tokio::test]
async fn test_get_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], stack.id.to_string());
    assert_eq!(json["name"], "Test Stack");
}

#[tokio::test]
async fn test_list_stacks() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stacks")
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let updated_stack = Stack {
        name: "Updated Stack".to_string(),
        description: Some("Updated Description".to_string()),
        ..stack.clone()
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "Updated Stack");
    assert_eq!(json["description"], "Updated Description");
}

#[tokio::test]
async fn test_soft_delete_stack() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify the stack is soft deleted
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}


#[tokio::test]
async fn test_add_stack_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_annotation = NewStackAnnotation {
        stack_id: stack.id,
        key: "test_key".to_string(),
        value: "test_value".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/annotations", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&new_annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["key"], "test_key");
    assert_eq!(json["value"], "test_value");
}

#[tokio::test]
async fn test_remove_stack_annotation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_annotation(stack.id, "test_key", "test_value");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/stacks/{}/annotations/test_key",
                    stack.id
                ))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_stack_annotations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_annotation(stack.id, "test_key1", "test_value1");
    fixture.create_test_stack_annotation(stack.id, "test_key2", "test_value2");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stacks/{}/annotations", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_add_stack_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_label = "test_label".to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/labels", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&new_label).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    println!("Response status: {}", status);
    println!("Response body: {}", body_str);

    assert_eq!(status, StatusCode::OK, "Unexpected status code. Body: {}", body_str);

    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["label"], "test_label");
}

#[tokio::test]
async fn test_remove_stack_label() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_label(stack.id, "test_label".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/stacks/{}/labels/test_label", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_stack_labels() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_label(stack.id, "test_label1".to_string());
    fixture.create_test_stack_label(stack.id, "test_label2".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/stacks/{}/labels", stack.id))
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_deployment_object = NewDeploymentObject {
        stack_id: stack.id,
        yaml_content: "test yaml".to_string(),
        yaml_checksum: "test checksum".to_string(),
        is_deletion_marker: false,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&new_deployment_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["stack_id"], stack.id.to_string());
    assert_eq!(json["yaml_content"], "test yaml");
    assert_eq!(json["yaml_checksum"], "test checksum");
    assert_eq!(json["is_deletion_marker"], false);
}


#[tokio::test]
async fn test_create_stack_with_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    let (generator_pak, generator_hash) = create_pak().unwrap();
    fixture.dal.generators().update_pak_hash(generator.id, generator_hash).unwrap();

    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        generator.id,
    )
    .expect("Failed to create NewStack");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stacks")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_stack_with_wrong_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator1 = fixture.create_test_generator(
        "Generator 1".to_string(),
        None,
        "test_api_key_hash_1".to_string(),
    );
    let generator2 = fixture.create_test_generator(
        "Generator 2".to_string(),
        None,
        "test_api_key_hash_2".to_string(),
    );

    let (generator2_pak, generator2_hash) = create_pak().unwrap();
    fixture.dal.generators().update_pak_hash(generator2.id, generator2_hash).unwrap();

    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        generator1.id,
    )
    .expect("Failed to create NewStack");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stacks")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator2_pak))
                .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_stack_with_wrong_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator1 = fixture.create_test_generator(
        "Generator 1".to_string(),
        None,
        "test_api_key_hash_1".to_string(),
    );
    let generator2 = fixture.create_test_generator(
        "Generator 2".to_string(),
        None,
        "test_api_key_hash_2".to_string(),
    );

    let (generator2_pak, generator2_hash) = create_pak().unwrap();
    fixture.dal.generators().update_pak_hash(generator2.id, generator2_hash).unwrap();

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator1.id);

    let updated_stack = Stack {
        name: "Updated Stack".to_string(),
        description: Some("Updated Description".to_string()),
        ..stack.clone()
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator2_pak))
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_stack_with_wrong_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator1 = fixture.create_test_generator(
        "Generator 1".to_string(),
        None,
        "test_api_key_hash_1".to_string(),
    );
    let generator2 = fixture.create_test_generator(
        "Generator 2".to_string(),
        None,
        "test_api_key_hash_2".to_string(),
    );

    let (generator2_pak, generator2_hash) = create_pak().unwrap();
    fixture.dal.generators().update_pak_hash(generator2.id, generator2_hash).unwrap();

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator1.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Authorization", format!("Bearer {}", generator2_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_add_stack_annotation_with_wrong_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator1 = fixture.create_test_generator(
        "Generator 1".to_string(),
        None,
        "test_api_key_hash_1".to_string(),
    );
    let generator2 = fixture.create_test_generator(
        "Generator 2".to_string(),
        None,
        "test_api_key_hash_2".to_string(),
    );

    let (generator2_pak, generator2_hash) = create_pak().unwrap();
    fixture.dal.generators().update_pak_hash(generator2.id, generator2_hash).unwrap();

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator1.id);

    let new_annotation = NewStackAnnotation {
        stack_id: stack.id,
        key: "test_key".to_string(),
        value: "test_value".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/annotations", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator2_pak))
                .body(Body::from(serde_json::to_string(&new_annotation).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}