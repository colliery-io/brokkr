/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use axum::{
    body::{Body, to_bytes},
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

    assert_eq!(response.status(), StatusCode::CREATED);

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

    assert_eq!(response.status(), StatusCode::CREATED);
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

// ---------------------------------------------------------------------------
// BROKKR-T-0395: an agent reads the objects of each stack it serves, by a
// target, a label or an annotation. The agent's diagnostics loop makes this
// read.
// ---------------------------------------------------------------------------

/// `GET /deployment-objects/{id}` as the bearer of `pak`.
async fn read_object_status(fixture: &TestFixture, pak: &str, object: Uuid) -> StatusCode {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    app.oneshot(
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/deployment-objects/{object}"))
            .header("Authorization", format!("Bearer {pak}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
    .status()
}

#[tokio::test]
async fn test_get_deployment_object_agent_by_label_alone() {
    let fixture = TestFixture::new();
    let (agent, pak) =
        fixture.create_test_agent_with_pak("t0395-label".to_string(), "c".to_string());
    let stack =
        fixture.create_test_stack("t0395-label".to_string(), None, fixture.admin_generator.id);
    let object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_label(agent.id, "t0395-team".to_string());
    fixture.create_test_stack_label(stack.id, "t0395-team".to_string());

    assert_eq!(
        read_object_status(&fixture, &pak, object.id).await,
        StatusCode::OK,
        "an agent that has the stack by a label reads its object"
    );
}

#[tokio::test]
async fn test_get_deployment_object_agent_by_annotation_alone() {
    let fixture = TestFixture::new();
    let (agent, pak) =
        fixture.create_test_agent_with_pak("t0395-annot".to_string(), "c".to_string());
    let stack =
        fixture.create_test_stack("t0395-annot".to_string(), None, fixture.admin_generator.id);
    let object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_annotation(agent.id, "t0395-tier".to_string(), "gold".to_string());
    fixture.create_test_stack_annotation(stack.id, "t0395-tier", "gold");

    assert_eq!(
        read_object_status(&fixture, &pak, object.id).await,
        StatusCode::OK,
        "an agent that has the stack by an annotation reads its object"
    );
}

/// Registration is the consent boundary (BROKKR-T-0287): a shared label with
/// a stack of a generator the agent is not registered with gives no read.
#[tokio::test]
async fn test_get_deployment_object_agent_label_without_registration_forbidden() {
    let fixture = TestFixture::new();
    let (agent, pak) =
        fixture.create_test_agent_with_pak("t0395-noreg".to_string(), "c".to_string());
    let other = fixture.create_test_generator(
        "t0395-other".to_string(),
        None,
        "t0395-other-hash".to_string(),
    );
    let stack = fixture.create_test_stack("t0395-noreg".to_string(), None, other.id);
    let object = fixture.create_test_deployment_object(stack.id, "test yaml".to_string(), false);
    fixture.create_test_agent_label(agent.id, "t0395-shared".to_string());
    fixture.create_test_stack_label(stack.id, "t0395-shared".to_string());

    assert_eq!(
        read_object_status(&fixture, &pak, object.id).await,
        StatusCode::FORBIDDEN
    );
}
