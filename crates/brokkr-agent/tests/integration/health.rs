/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use brokkr_agent::health::{configure_health_routes, BrokerStatus, HealthState};
use kube::Client;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tower::ServiceExt;

async fn create_test_health_state() -> HealthState {
    // Create a Kubernetes client (this will use the default kubeconfig or fail gracefully in CI)
    let k8s_client = Client::try_default()
        .await
        .unwrap_or_else(|_| panic!("Failed to create Kubernetes client for tests"));

    let broker_status = Arc::new(RwLock::new(BrokerStatus {
        connected: true,
        last_heartbeat: Some("2025-10-21T12:00:00Z".to_string()),
    }));

    HealthState {
        k8s_client,
        broker_status,
        start_time: SystemTime::now(),
    }
}

#[tokio::test]
async fn test_healthz_endpoint() {
    let state = create_test_health_state().await;
    let app = configure_health_routes(state);

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
    let state = create_test_health_state().await;
    let app = configure_health_routes(state);

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

    // Status could be OK or SERVICE_UNAVAILABLE depending on K8s connectivity
    // Just verify we get a response
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = create_test_health_state().await;
    let app = configure_health_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Status could be OK or SERVICE_UNAVAILABLE depending on K8s connectivity
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid JSON with expected fields
    let health_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert!(health_response["status"].is_string());
    assert!(health_response["kubernetes"].is_object());
    assert!(health_response["broker"].is_object());
    assert!(health_response["uptime_seconds"].is_number());
    assert!(health_response["version"].is_string());
    assert!(health_response["timestamp"].is_string());
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let state = create_test_health_state().await;
    let app = configure_health_routes(state);

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
    assert!(body_str.contains("brokkr_agent_poll_requests_total"));
    assert!(body_str.contains("brokkr_agent_poll_duration_seconds"));
    assert!(body_str.contains("brokkr_agent_kubernetes_operations_total"));
    assert!(body_str.contains("brokkr_agent_kubernetes_operation_duration_seconds"));
    assert!(body_str.contains("brokkr_agent_heartbeat_sent_total"));
    assert!(body_str.contains("brokkr_agent_last_successful_poll_timestamp"));
}
