/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for audit log API endpoints.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::fixtures::TestFixture;

/// Test that the audit logs endpoint requires authentication.
#[tokio::test]
async fn test_audit_logs_requires_auth() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    // Send request without authentication
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit-logs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that non-admin users cannot access audit logs.
#[tokio::test]
async fn test_audit_logs_requires_admin() {
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
                .method("GET")
                .uri("/api/v1/admin/audit-logs")
                .header("Authorization", format!("Bearer {}", agent_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Test that admin users can access audit logs.
#[tokio::test]
async fn test_audit_logs_success_with_admin() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    let admin_pak = fixture.admin_pak.clone();

    // Send request with admin PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit-logs")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(list_response["logs"].is_array());
    assert!(list_response["total"].is_number());
    assert!(list_response["count"].is_number());
    assert!(list_response["limit"].is_number());
    assert!(list_response["offset"].is_number());
}

/// Test audit logs with pagination parameters.
#[tokio::test]
async fn test_audit_logs_pagination() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    let admin_pak = fixture.admin_pak.clone();

    // Send request with pagination parameters
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit-logs?limit=10&offset=0")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify pagination is applied
    assert_eq!(list_response["limit"], 10);
    assert_eq!(list_response["offset"], 0);
}

/// Test audit logs with filter parameters.
#[tokio::test]
async fn test_audit_logs_filtering() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    let admin_pak = fixture.admin_pak.clone();

    // Send request with filter parameters
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit-logs?actor_type=admin&action=agent.*")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Response should be valid even if no logs match
    assert!(list_response["logs"].is_array());
}

/// Test that generator PAK cannot access audit logs (admin only).
#[tokio::test]
async fn test_audit_logs_denied_for_generator() {
    let fixture = TestFixture::new();
    let app = fixture
        .create_test_router()
        .with_state(fixture.dal.clone());

    // Create a generator with PAK
    let (_generator, generator_pak) = fixture.create_test_generator_with_pak(
        "Test Generator".to_string(),
        Some("Test generator for audit tests".to_string()),
    );

    // Send request with generator PAK (not admin)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit-logs")
                .header("Authorization", format!("Bearer {}", generator_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
