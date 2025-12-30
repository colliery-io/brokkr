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
}

#[tokio::test]
async fn test_metrics_records_http_requests() {
    let fixture = TestFixture::new();

    // Make a request to healthz first to generate metrics
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let _ = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Now check metrics endpoint for recorded data
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

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify HTTP request metrics are present
    assert!(
        body_str.contains("brokkr_http_requests_total"),
        "Should contain HTTP request counter metric"
    );
    assert!(
        body_str.contains("brokkr_http_request_duration_seconds"),
        "Should contain HTTP request duration histogram"
    );

    // Verify the metrics have the expected labels
    assert!(
        body_str.contains("endpoint=") || body_str.contains("method="),
        "HTTP metrics should have endpoint and method labels"
    );
}

#[tokio::test]
async fn test_metrics_contains_all_defined_metrics() {
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

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify all defined metric types are present in the output
    // These metrics may have no data yet, but their definitions should exist
    let expected_metrics = [
        "brokkr_http_requests_total",
        "brokkr_http_request_duration_seconds",
        "brokkr_database_queries_total",
        "brokkr_database_query_duration_seconds",
        "brokkr_active_agents",
        "brokkr_agent_heartbeat_age_seconds",
        "brokkr_stacks_total",
        "brokkr_deployment_objects_total",
    ];

    for metric_name in expected_metrics {
        assert!(
            body_str.contains(metric_name),
            "Metrics output should contain '{}' definition",
            metric_name
        );
    }
}
