/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for admin API endpoints.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::fixtures::TestFixture;

/// Test that the config reload endpoint requires authentication.
#[tokio::test]
async fn test_config_reload_requires_auth() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    // Send request without authentication
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/config/reload")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that non-admin users cannot access config reload.
#[tokio::test]
async fn test_config_reload_requires_admin() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    // Create a non-admin agent with PAK
    let (_agent, agent_pak) = fixture.create_test_agent_with_pak(
        "Test Agent".to_string(),
        "Test Cluster".to_string(),
    );

    // Send request with agent PAK (not admin)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/config/reload")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Test that admin users can successfully reload configuration.
#[tokio::test]
async fn test_config_reload_success_with_admin() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    let admin_pak = fixture.admin_pak.clone();

    // Send request with admin PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/config/reload")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let reload_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(reload_response["reloaded_at"].is_string());
    assert!(reload_response["changes"].is_array());
    assert_eq!(reload_response["success"], true);
    assert!(reload_response["message"].is_string());
}

/// Test that config reload returns no changes when config hasn't changed.
#[tokio::test]
async fn test_config_reload_no_changes() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    let admin_pak = fixture.admin_pak.clone();

    // First reload
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/config/reload")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let reload_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Since config file hasn't changed, there should be no changes
    let changes = reload_response["changes"].as_array().unwrap();
    assert!(changes.is_empty(), "Expected no changes on reload without config file change");
}

/// Test that generator PAK cannot access config reload (admin only).
#[tokio::test]
async fn test_config_reload_denied_for_generator() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    // Create a generator with PAK
    let (_generator, generator_pak) = fixture.create_test_generator_with_pak(
        "Test Generator".to_string(),
        Some("Test generator for admin tests".to_string()),
    );

    // Send request with generator PAK (not admin)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/config/reload")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
