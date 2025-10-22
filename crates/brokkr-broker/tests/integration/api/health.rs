/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::fixtures::TestFixture;

#[tokio::test]
async fn test_healthz_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "OK");
}

#[tokio::test]
async fn test_readyz_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "Ready");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check Content-Type header
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(content_type, "text/plain; version=0.0.4");

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it contains Prometheus metrics format
    // Check for some expected metrics we defined
    assert!(body_str.contains("brokkr_http_requests_total"));
    assert!(body_str.contains("brokkr_http_request_duration_seconds"));
    assert!(body_str.contains("brokkr_database_queries_total"));
    assert!(body_str.contains("brokkr_database_query_duration_seconds"));
    assert!(body_str.contains("brokkr_active_agents"));
    assert!(body_str.contains("brokkr_agent_heartbeat_age_seconds"));
    assert!(body_str.contains("brokkr_stacks_total"));
    assert!(body_str.contains("brokkr_deployment_objects_total"));
}
