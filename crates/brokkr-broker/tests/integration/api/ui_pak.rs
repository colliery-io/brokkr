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
