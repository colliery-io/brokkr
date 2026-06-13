/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! API-level tests for the agent-reported K8s connectivity signal in the
//! fleet surface (BROKKR-T-0227).

use crate::fixtures::TestFixture;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

/// An agent that reports `k8s_reachable = false` surfaces as `false` in its
/// `/fleet` record; an agent that never reports surfaces `null` for both
/// connectivity fields without breaking the rollup.
#[tokio::test]
async fn test_fleet_surfaces_agent_reported_k8s_connectivity() {
    let fixture = TestFixture::new();
    let app: Router = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Reporting agent: stores k8s_reachable = false with a latency sample.
    let reporting = fixture.create_test_agent("reporting".to_string(), "c".to_string());
    fixture
        .dal
        .agents()
        .record_k8s_connectivity(reporting.id, false, Some(42))
        .expect("Failed to record k8s connectivity");

    // Silent agent: never reports — connectivity columns stay NULL.
    let silent = fixture.create_test_agent("silent".to_string(), "c".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/fleet")
                .header("Authorization", admin_pak)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let records: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let reporting_record = records
        .iter()
        .find(|r| r["agent_id"] == serde_json::json!(reporting.id))
        .expect("reporting agent missing from fleet rollup");
    assert_eq!(reporting_record["k8s_reachable"], Value::Bool(false));
    assert_eq!(reporting_record["k8s_api_latency_ms"], serde_json::json!(42));

    let silent_record = records
        .iter()
        .find(|r| r["agent_id"] == serde_json::json!(silent.id))
        .expect("silent agent missing from fleet rollup");
    assert_eq!(silent_record["k8s_reachable"], Value::Null);
    assert_eq!(silent_record["k8s_api_latency_ms"], Value::Null);
}
