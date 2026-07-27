/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use axum::{
    body::{Body, to_bytes},
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

/// BROKKR-T-0291: `/readyz` performs a real database check, so this exercises
/// the happy path against the fixture's live database.
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

/// The verdict is cached for a couple of seconds so a 5s-per-pod kubelet probe
/// cannot hammer the pool. Back-to-back probes on the same router must both
/// answer Ready — the second one off the cache.
#[tokio::test]
async fn test_readyz_result_is_cached_across_probes() {
    let fixture = TestFixture::new();
    let router = fixture.create_test_router().with_state(fixture.dal.clone());

    for _ in 0..3 {
        let response = router
            .clone()
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
        assert_eq!(String::from_utf8(body.to_vec()).unwrap(), "Ready");
    }
}

/// A broker whose database is unreachable must fall out of Service endpoints.
///
/// The unreachable database is simulated with an unchecked r2d2 pool pointed at
/// a closed port; `connection_timeout` is kept short so the router's background
/// tasks (fleet sweep, telemetry eviction), which also share this DAL, fail fast
/// instead of blocking on r2d2's 30s default.
#[tokio::test]
async fn test_readyz_returns_503_when_database_unreachable() {
    use brokkr_broker::api;
    use brokkr_broker::dal::DAL;
    use brokkr_broker::db::ConnectionPool;
    use brokkr_utils::config::Cors;
    use diesel::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool};
    use std::time::Duration;

    let manager = ConnectionManager::<PgConnection>::new(
        "postgres://brokkr:brokkr@127.0.0.1:1/brokkr_unreachable",
    );
    let pool = Pool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_millis(200))
        .build_unchecked(manager);
    let dal = DAL::new(ConnectionPool { pool, schema: None });

    let cors = Cors {
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec!["GET".to_string()],
        allowed_headers: vec!["Content-Type".to_string()],
        max_age_seconds: 3600,
    };

    let app = api::configure_api_routes(dal.clone(), &cors, None).with_state(dal);

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

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

/// Liveness must stay process-only: a dead database must not restart-loop the
/// fleet. Same broken DAL as above, but `/healthz` still answers 200.
#[tokio::test]
async fn test_healthz_stays_up_when_database_unreachable() {
    use brokkr_broker::api;
    use brokkr_broker::dal::DAL;
    use brokkr_broker::db::ConnectionPool;
    use brokkr_utils::config::Cors;
    use diesel::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool};
    use std::time::Duration;

    let manager = ConnectionManager::<PgConnection>::new(
        "postgres://brokkr:brokkr@127.0.0.1:1/brokkr_unreachable",
    );
    let pool = Pool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_millis(200))
        .build_unchecked(manager);
    let dal = DAL::new(ConnectionPool { pool, schema: None });

    let cors = Cors {
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec!["GET".to_string()],
        allowed_headers: vec!["Content-Type".to_string()],
        max_age_seconds: 3600,
    };

    let app = api::configure_api_routes(dal.clone(), &cors, None).with_state(dal);

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

    // Verify metric types that are always present in the output.
    // Note: CounterVec/HistogramVec metrics only appear after at least one value
    // is recorded. GaugeVec metrics like agent_heartbeat_age_seconds also need
    // data first.
    let expected_metrics = [
        "brokkr_http_requests_total",
        "brokkr_http_request_duration_seconds",
        "brokkr_active_agents",
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
