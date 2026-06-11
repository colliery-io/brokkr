/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use brokkr_models::models::{
    deployment_objects::DeploymentObject,
    stack_annotations::NewStackAnnotation,
    stacks::{NewStack, Stack},
};

use tower::ServiceExt;

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

    assert_eq!(response.status(), StatusCode::CREATED);

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
async fn test_list_stacks_with_generator_pak_filters_to_own() {
    // BROKKR-T-0155: a generator PAK on GET /stacks must see ONLY its own
    // stacks (not 403, not all stacks).
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Generator A — has two stacks.
    let gen_a = fixture.create_test_generator(
        "Generator A".to_string(),
        None,
        "gen_a_initial_hash".to_string(),
    );
    let (gen_a_pak, gen_a_hash) = create_pak().unwrap();
    fixture
        .dal
        .generators()
        .update_pak_hash(gen_a.id, gen_a_hash)
        .unwrap();
    let a_stack_1 = fixture.create_test_stack("A Stack 1".to_string(), None, gen_a.id);
    let a_stack_2 = fixture.create_test_stack("A Stack 2".to_string(), None, gen_a.id);

    // Generator B — has one stack; must NOT appear in A's list.
    let gen_b = fixture.create_test_generator(
        "Generator B".to_string(),
        None,
        "gen_b_initial_hash".to_string(),
    );
    let _b_stack = fixture.create_test_stack("B Stack".to_string(), None, gen_b.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stacks")
                .header("Authorization", &gen_a_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let stacks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    let ids: Vec<&str> = stacks.iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids.len(),
        2,
        "Generator A should see exactly its 2 stacks, got {ids:?}"
    );
    assert!(ids.contains(&a_stack_1.id.to_string().as_str()));
    assert!(ids.contains(&a_stack_2.id.to_string().as_str()));
    for s in &stacks {
        assert_eq!(
            s["generator_id"].as_str().unwrap(),
            gen_a.id.to_string(),
            "stack {} leaked from another generator",
            s["id"]
        );
    }
}

#[tokio::test]
async fn test_list_stacks_without_pak_forbidden() {
    // BROKKR-T-0155: callers without admin or generator scope still get 403.
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // An agent PAK has neither admin nor generator scope.
    let (_agent, agent_pak) =
        fixture.create_test_agent_with_pak("contract-agent".to_string(), "cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stacks")
                .header("Authorization", &agent_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "stacks_list_denied");
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

    let response = app
        .clone()
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
    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::CREATED);

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
                .uri(format!("/api/v1/stacks/{}/annotations/test_key", stack.id))
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

    assert_eq!(
        status,
        StatusCode::CREATED,
        "Unexpected status code. Body: {}",
        body_str
    );

    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["label"], "test_label");
}

#[tokio::test]
async fn test_add_stack_label_duplicate_returns_409() {
    // Re-adding an existing label hits the UNIQUE (stack_id, label) constraint.
    // It must surface as 409 unique_violation (not a blanket 500) so idempotent
    // callers like the SDK `apply` helper can treat it as a no-op.
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_label(stack.id, "dup_label".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/labels", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(
                    serde_json::to_string(&"dup_label".to_string()).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "duplicate label should be 409, got body: {json}"
    );
    assert_eq!(json["code"], "unique_violation");
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

    let payload = serde_json::json!({
        "yaml_content": "test yaml",
        "is_deletion_marker": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_object: DeploymentObject = serde_json::from_slice(&body).unwrap();
    assert_eq!(created_object.stack_id, stack.id);
    assert_eq!(created_object.yaml_content, "test yaml");
    assert!(!created_object.is_deletion_marker);
    assert!(!created_object.yaml_checksum.is_empty());
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
    fixture
        .dal
        .generators()
        .update_pak_hash(generator.id, generator_hash)
        .unwrap();

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

    assert_eq!(response.status(), StatusCode::CREATED);
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
    fixture
        .dal
        .generators()
        .update_pak_hash(generator2.id, generator2_hash)
        .unwrap();

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
    fixture
        .dal
        .generators()
        .update_pak_hash(generator2.id, generator2_hash)
        .unwrap();

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
    fixture
        .dal
        .generators()
        .update_pak_hash(generator2.id, generator2_hash)
        .unwrap();

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
    fixture
        .dal
        .generators()
        .update_pak_hash(generator2.id, generator2_hash)
        .unwrap();

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

// --- BROKKR-T-0194: raw-YAML submission + Accept round-trip ---

#[tokio::test]
async fn test_create_deployment_object_yaml_body() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();
    let generator =
        fixture.create_test_generator("yaml-gen".to_string(), None, "h".to_string());
    let stack = fixture.create_test_stack("yaml-stack".to_string(), None, generator.id);

    let yaml = "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: foo\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bar\n";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/yaml")
                .header("Authorization", &admin_pak)
                .body(Body::from(yaml))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: DeploymentObject = serde_json::from_slice(&body).unwrap();
    assert_eq!(created.yaml_content, yaml, "raw YAML body stored verbatim");
    assert!(!created.is_deletion_marker);
    assert!(!created.yaml_checksum.is_empty());
}

#[tokio::test]
async fn test_create_deployment_object_yaml_deletion_marker_empty() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();
    let generator =
        fixture.create_test_generator("del-gen".to_string(), None, "h".to_string());
    let stack = fixture.create_test_stack("del-stack".to_string(), None, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/stacks/{}/deployment-objects?deletion_marker=true",
                    stack.id
                ))
                .header("Content-Type", "application/yaml")
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "empty body + deletion_marker=true should be accepted"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: DeploymentObject = serde_json::from_slice(&body).unwrap();
    assert!(created.is_deletion_marker);
    assert!(created.yaml_content.is_empty());
}

#[tokio::test]
async fn test_create_deployment_object_malformed_yaml_rejected() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();
    let generator =
        fixture.create_test_generator("bad-gen".to_string(), None, "h".to_string());
    let stack = fixture.create_test_stack("bad-stack".to_string(), None, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/yaml")
                .header("Authorization", &admin_pak)
                .body(Body::from("kind: : : [unbalanced"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_deployment_object_accept_yaml_roundtrip() {
    let fixture = TestFixture::new();
    let admin_pak = fixture.admin_pak.clone();
    let generator =
        fixture.create_test_generator("rt-gen".to_string(), None, "h".to_string());
    let stack = fixture.create_test_stack("rt-stack".to_string(), None, generator.id);
    let yaml = "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: rt\n";

    // create via YAML body
    let created: DeploymentObject = {
        let app = fixture.create_test_router().with_state(fixture.dal.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                    .header("Content-Type", "application/yaml")
                    .header("Authorization", &admin_pak)
                    .body(Body::from(yaml))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let b = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&b).unwrap()
    };

    // GET it back as raw YAML
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/deployment-objects/{}", created.id))
                .header("Accept", "application/yaml")
                .header("Authorization", &admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(ct.starts_with("application/yaml"), "got content-type {ct}");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(String::from_utf8(body.to_vec()).unwrap(), yaml);
}

#[tokio::test]
async fn test_create_deployment_object_json_still_works() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();
    let generator =
        fixture.create_test_generator("json-gen".to_string(), None, "h".to_string());
    let stack = fixture.create_test_stack("json-stack".to_string(), None, generator.id);

    let payload = serde_json::json!({
        "yaml_content": "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: j\n",
        "is_deletion_marker": false
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/stacks/{}/deployment-objects", stack.id))
                .header("Content-Type", "application/json")
                .header("Authorization", &admin_pak)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
