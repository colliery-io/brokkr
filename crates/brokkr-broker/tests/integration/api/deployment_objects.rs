/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use brokkr_broker::utils::pak::create_pak;
use brokkr_models::models::deployment_objects::NewDeploymentObject;
use brokkr_models::models::stacks::Stack;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_deployment_object_admin_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_agent_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let (pak, hash) = create_pak().unwrap();
    fixture
        .dal
        .agents()
        .update_pak_hash(agent.id, hash)
        .unwrap();

    let stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_target(agent.id, stack.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_generator_success() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (pak, hash) = create_pak().unwrap();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        hash,
    );

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("body: {:?}", body);

    // let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    // assert_eq!(response.status(), StatusCode::OK);

    let fetched_object: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_object["id"], deployment_object.id.to_string());
}

#[tokio::test]
async fn test_get_deployment_object_agent_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let (pak, hash) = create_pak().unwrap();
    fixture
        .dal
        .agents()
        .update_pak_hash(agent.id, hash)
        .unwrap();

    let stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    // Note: We're not creating an agent target, so the agent shouldn't have access

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_deployment_object_generator_forbidden() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (generator1_pak, generator1_hash) = create_pak().unwrap();
    let (_generator2_pak, generator2_hash) = create_pak().unwrap();

    let _generator1 = fixture.create_test_generator(
        "Test Generator 1".to_string(),
        Some("Test Description 1".to_string()),
        generator1_hash,
    );

    let generator2 = fixture.create_test_generator(
        "Test Generator 2".to_string(),
        Some("Test Description 2".to_string()),
        generator2_hash,
    );

    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator2.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .header("Authorization", format!("Bearer {}", generator1_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_deployment_object_not_found() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let non_existent_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", non_existent_id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_deployment_object_unauthorized() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let stack =
        fixture.create_test_stack("Test Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/deployment-objects/{}",
                    deployment_object.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_stack_with_admin_pak() {
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
async fn test_update_stack_with_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

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

    let (generator_pak, generator_hash) = create_pak().unwrap();
    fixture
        .dal
        .generators()
        .update_pak_hash(generator.id, generator_hash)
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &generator_pak)
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_stack_with_bad_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

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

    let bad_pak = "brokkr_BR555Hk5_EnacJW2sQCxRCr1rhRTLyf8NP3a55555".to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/stacks/{}", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &bad_pak)
                .body(Body::from(serde_json::to_string(&updated_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_deployment_object_with_admin_pak() {
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
                .body(Body::from(
                    serde_json::to_string(&new_deployment_object).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["stack_id"], stack.id.to_string());
    assert_eq!(json["yaml_content"], "test yaml");
}

#[tokio::test]
async fn test_create_deployment_object_with_generator_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let payload = serde_json::json!({
        "yaml_content": "test yaml",
        "is_deletion_marker": false
    });

    let (generator_pak, generator_hash) = create_pak().unwrap();
    fixture
        .dal
        .generators()
        .update_pak_hash(generator.id, generator_hash)
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &generator_pak)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_deployment_object_with_bad_pak() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

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

    let bad_pak = "brokkr_BR555Hk5_EnacJW2sQCxRCr1rhRTLyf8NP3a55555".to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &bad_pak)
                .body(Body::from(
                    serde_json::to_string(&new_deployment_object).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
