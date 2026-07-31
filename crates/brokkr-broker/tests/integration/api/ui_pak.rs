/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for the ephemeral read-only UI PAK (BROKKR-T-0267).

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use brokkr_broker::utils::ui_pak;

use crate::fixtures::TestFixture;

/// The fixture mints the UI PAK; grab the raw token like the served console would.
fn ui_token() -> String {
    ui_pak::token().expect("UI PAK not minted").to_string()
}

#[tokio::test]
async fn test_ui_pak_verifies_as_readonly_admin() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/pak")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(auth["admin"], true);
    assert_eq!(auth["readonly"], true);
    assert_eq!(auth["agent"], json!(null));
    assert_eq!(auth["generator"], json!(null));
}

#[tokio::test]
async fn test_ui_pak_can_read() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    for path in ["/api/v1/fleet", "/api/v1/stacks", "/api/v1/agent-events"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(path)
                    .header("Authorization", format!("Bearer {}", ui_token()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "GET {path} with UI PAK");
    }
}

#[tokio::test]
async fn test_ui_pak_cannot_mutate() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let agent = fixture.create_test_agent("ui-pak-victim".to_string(), "cluster".to_string());

    // POST: stack creation
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/stacks")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::from(
                    json!({"name": "nope", "description": null}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "POST /stacks");

    // PUT: agent update
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/agents/{}", agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::from(
                    json!({"name": "renamed", "cluster_name": "cluster"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "PUT /agents/:id");

    // DELETE: agent deletion
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/agents/{}", agent.id))
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "DELETE /agents/:id"
    );

    // The agent must be untouched.
    let still_there = fixture.dal.agents().get(agent.id).unwrap().unwrap();
    assert_eq!(still_there.name, "ui-pak-victim");
}

/// The POST leg of the console's diagnostic flow, in isolation: the readonly
/// allowlist admits `POST /deployment-objects/:id/diagnostics`.
///
/// See `test_ui_pak_walks_console_diagnostic_path` for the full path the
/// console actually drives (target-state fetch, then this POST).
#[tokio::test]
async fn test_ui_pak_can_create_diagnostics() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let (generator, _generator_pak) =
        fixture.create_test_generator_with_pak("diag-gen".to_string(), None);
    let stack = fixture.create_test_stack("diag-stack".to_string(), None, generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "kind: ConfigMap".to_string(), false);
    let agent = fixture.create_test_agent("diag-agent".to_string(), "cluster".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/deployment-objects/{}/diagnostics",
                    deployment_object.id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::from(
                    json!({"agent_id": agent.id, "requested_by": "ui-pak-test"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // The diagnostics allowlist must let the readonly credential through the
    // middleware; the handler then accepts it because the UI PAK is admin.
    assert_eq!(response.status(), StatusCode::CREATED);
}

/// The console's full "Run diagnostic" path with the injected read-only UI PAK
/// (BROKKR-T-0275): the Fleet modal has only an agent id, so it fetches the
/// agent's target state to populate a deployment-object picker, then POSTs to
/// the deployment-object-scoped diagnostics route with the chosen id.
///
/// This is the leg the earlier test does not cover — that the *read* the picker
/// depends on also passes for the UI PAK, and that its payload carries the
/// fields the console's `TargetStateObject` mirror deserializes.
#[tokio::test]
async fn test_ui_pak_walks_console_diagnostic_path() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let agent = fixture.create_test_agent("console-diag-agent".to_string(), "cluster".to_string());
    let stack = fixture.create_test_stack(
        "console-diag-stack".to_string(),
        None,
        fixture.admin_generator.id,
    );
    fixture.create_test_agent_target(agent.id, stack.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "kind: ConfigMap".to_string(), false);

    // 1. The picker's read: GET /agents/:id/target-state?mode=full.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/agents/{}/target-state?mode=full",
                    agent.id
                ))
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /agents/:id/target-state?mode=full with UI PAK"
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let objects: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let objects = objects.as_array().expect("target state is an array");
    assert_eq!(objects.len(), 1, "the agent's one deployment object");

    // The fields the console's `TargetStateObject` mirror needs to build a
    // picker label and the POST path.
    let object = &objects[0];
    assert_eq!(object["id"], json!(deployment_object.id.to_string()));
    assert_eq!(object["stack_id"], json!(stack.id.to_string()));
    assert!(object["sequence_id"].is_i64(), "sequence_id is a number");
    assert!(
        object["is_deletion_marker"].is_boolean(),
        "is_deletion_marker is a bool"
    );

    // 2. The picked object drives the write.
    let picked = object["id"].as_str().unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/deployment-objects/{picked}/diagnostics"))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::from(
                    json!({"agent_id": agent.id, "requested_by": "operator-console"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        created["deployment_object_id"],
        json!(deployment_object.id.to_string())
    );
    assert_eq!(created["agent_id"], json!(agent.id.to_string()));
}

#[tokio::test]
async fn test_admin_pak_is_not_readonly() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/pak")
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(auth["admin"], true);
    assert_eq!(auth["readonly"], false);
}

/// The console's tenant-minting panel (BROKKR-T-0318) works by having the
/// *operator* supply an admin PAK for that one request. This pins the reason
/// that design is safe: the console's **own** credential still cannot mint a
/// generator, so reaching the page grants no ability to create credentials.
///
/// If this ever starts passing with a 201, the console's security model has
/// changed and the "network reach is the authentication boundary" statements in
/// `security-model.md` are no longer true.
#[tokio::test]
async fn test_ui_pak_cannot_mint_a_generator() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generators")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", ui_token()))
                .body(Body::from(
                    json!({"name": "minted-by-the-console", "description": null}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "the read-only console credential must not be able to mint a tenant"
    );

    // The generator must not exist either -- a 403 that still wrote the row
    // would be worse than no check at all.
    let generators = fixture
        .dal
        .generators()
        .list()
        .expect("failed to list generators");
    assert!(
        !generators.iter().any(|g| g.name == "minted-by-the-console"),
        "no generator may be created on the rejected path"
    );
}
