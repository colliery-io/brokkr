/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use brokkr_broker::utils::pak;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_verify_pak_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Create a test agent
    let test_agent =
        fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Generate a PAK for the test agent
    let (pak, hash) = pak::create_pak().unwrap();

    // Update the agent's PAK hash
    fixture
        .dal
        .agents()
        .update_pak_hash(test_agent.id, hash)
        .unwrap();

    // Send a request to verify the PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/pak")
                .header("Content-Type", "application/json")
                .header("Authorization", pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response["admin"], false);
    assert_eq!(auth_response["agent"], test_agent.id.to_string());
    assert_eq!(auth_response["generator"], json!(null));
}

#[tokio::test]
async fn test_verify_admin_pak_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    // Generate an admin PAK
    let admin_pak = fixture.admin_pak.clone();

    // Send a request to verify the admin PAK
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/pak")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response["admin"], true);
    assert_eq!(auth_response["agent"], json!(null));
    assert_eq!(auth_response["generator"], json!(null));
}

/// BROKKR-T-0384: a refused PAK answers 401 with an ErrorResponse body, so a
/// client that expects JSON on 401 (the SDKs do) does not fail on an empty
/// payload and can say why the request was refused.
#[tokio::test]
async fn a_refused_pak_answers_with_a_json_body() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agents")
                .header("Authorization", "Bearer brokkr_nope_not_a_real_pak")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("a JSON body");
    assert_eq!(json["code"], "unauthorized");
    assert!(
        json["message"]
            .as_str()
            .unwrap_or_default()
            .contains("refused the PAK"),
        "{json}"
    );
}
