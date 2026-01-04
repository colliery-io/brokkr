/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Metrics Module
//!
//! This module provides Prometheus metrics for the Brokkr Broker.
//! It exposes metrics about HTTP requests, database queries, and system state.

use once_cell::sync::Lazy;
use prometheus::{
    CounterVec, Encoder, GaugeVec, HistogramOpts, HistogramVec, IntGauge, Opts, Registry,
    TextEncoder,
};

/// Global Prometheus registry for all broker metrics
pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// HTTP request counter
/// Labels: endpoint, method, status
pub static HTTP_REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "brokkr_http_requests_total",
        "Total number of HTTP requests by endpoint and status",
    );
    let counter = CounterVec::new(opts, &["endpoint", "method", "status"])
        .expect("Failed to create HTTP requests counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("Failed to register HTTP requests counter");
    counter
});

/// HTTP request duration histogram
/// Labels: endpoint, method
pub static HTTP_REQUEST_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "brokkr_http_request_duration_seconds",
        "HTTP request latency distribution in seconds",
    )
    .buckets(vec![
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ]);
    let histogram = HistogramVec::new(opts, &["endpoint", "method"])
        .expect("Failed to create HTTP request duration histogram");
    REGISTRY
        .register(Box::new(histogram.clone()))
        .expect("Failed to register HTTP request duration histogram");
    histogram
});

/// Database query counter
/// Labels: query_type
pub static DATABASE_QUERIES_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "brokkr_database_queries_total",
        "Total number of database queries by type",
    );
    let counter =
        CounterVec::new(opts, &["query_type"]).expect("Failed to create database queries counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("Failed to register database queries counter");
    counter
});

/// Database query duration histogram
/// Labels: query_type
pub static DATABASE_QUERY_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "brokkr_database_query_duration_seconds",
        "Database query latency distribution in seconds",
    )
    .buckets(vec![
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
    ]);
    let histogram = HistogramVec::new(opts, &["query_type"])
        .expect("Failed to create database query duration histogram");
    REGISTRY
        .register(Box::new(histogram.clone()))
        .expect("Failed to register database query duration histogram");
    histogram
});

/// Number of active agents
pub static ACTIVE_AGENTS: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new("brokkr_active_agents", "Number of active agents");
    let gauge = IntGauge::with_opts(opts).expect("Failed to create active agents gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("Failed to register active agents gauge");
    gauge
});

/// Agent heartbeat age gauge
/// Labels: agent_id, agent_name
pub static AGENT_HEARTBEAT_AGE_SECONDS: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "brokkr_agent_heartbeat_age_seconds",
        "Time since last heartbeat per agent in seconds",
    );
    let gauge = GaugeVec::new(opts, &["agent_id", "agent_name"])
        .expect("Failed to create agent heartbeat age gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("Failed to register agent heartbeat age gauge");
    gauge
});

/// Total number of stacks
pub static STACKS_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new("brokkr_stacks_total", "Total number of stacks");
    let gauge = IntGauge::with_opts(opts).expect("Failed to create stacks total gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("Failed to register stacks total gauge");
    gauge
});

/// Total number of deployment objects
pub static DEPLOYMENT_OBJECTS_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new(
        "brokkr_deployment_objects_total",
        "Total number of deployment objects",
    );
    let gauge = IntGauge::with_opts(opts).expect("Failed to create deployment objects total gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("Failed to register deployment objects total gauge");
    gauge
});

/// Initializes all metrics by forcing lazy static evaluation
///
/// This ensures all metric definitions are registered with the registry
/// before attempting to encode/export them. Should be called once at startup.
pub fn init() {
    // Force initialization of all lazy statics by accessing them
    let _ = &*HTTP_REQUESTS_TOTAL;
    let _ = &*HTTP_REQUEST_DURATION_SECONDS;
    let _ = &*DATABASE_QUERIES_TOTAL;
    let _ = &*DATABASE_QUERY_DURATION_SECONDS;
    let _ = &*ACTIVE_AGENTS;
    let _ = &*AGENT_HEARTBEAT_AGE_SECONDS;
    let _ = &*STACKS_TOTAL;
    let _ = &*DEPLOYMENT_OBJECTS_TOTAL;
}

/// Encodes all registered metrics in Prometheus text format
///
/// # Returns
///
/// Returns a String containing all metrics in Prometheus exposition format
pub fn encode_metrics() -> String {
    // Ensure all metrics are initialized before encoding
    init();

    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");
    String::from_utf8(buffer).expect("Failed to convert metrics to UTF-8")
}

// =============================================================================
// Helper Functions for Recording Metrics
// =============================================================================

/// Records an HTTP request metric
///
/// # Arguments
/// * `endpoint` - The request path/endpoint
/// * `method` - The HTTP method (GET, POST, etc.)
/// * `status` - The response status code
/// * `duration_seconds` - The request duration in seconds
pub fn record_http_request(endpoint: &str, method: &str, status: u16, duration_seconds: f64) {
    // Normalize endpoint to avoid high cardinality from path parameters
    let normalized_endpoint = normalize_endpoint(endpoint);
    let status_str = status.to_string();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&normalized_endpoint, method, &status_str])
        .inc();

    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&normalized_endpoint, method])
        .observe(duration_seconds);
}

/// Normalizes an endpoint path to reduce cardinality
/// Replaces UUIDs and numeric IDs with placeholders
fn normalize_endpoint(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = parts
        .iter()
        .map(|part| {
            // Check if it's a UUID (36 chars with hyphens)
            if part.len() == 36 && part.chars().filter(|c| *c == '-').count() == 4 {
                ":id".to_string()
            // Check if it's purely numeric
            } else if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) {
                ":id".to_string()
            } else {
                (*part).to_string()
            }
        })
        .collect();
    normalized.join("/")
}

/// Records a database query metric
///
/// # Arguments
/// * `query_type` - The type of query (select, insert, update, delete)
/// * `duration_seconds` - The query duration in seconds
pub fn record_db_query(query_type: &str, duration_seconds: f64) {
    DATABASE_QUERIES_TOTAL
        .with_label_values(&[query_type])
        .inc();

    DATABASE_QUERY_DURATION_SECONDS
        .with_label_values(&[query_type])
        .observe(duration_seconds);
}

/// Updates the active agents gauge
pub fn set_active_agents(count: i64) {
    ACTIVE_AGENTS.set(count);
}

/// Updates the total stacks gauge
pub fn set_stacks_total(count: i64) {
    STACKS_TOTAL.set(count);
}

/// Updates the total deployment objects gauge
pub fn set_deployment_objects_total(count: i64) {
    DEPLOYMENT_OBJECTS_TOTAL.set(count);
}

/// Updates the heartbeat age for a specific agent
pub fn set_agent_heartbeat_age(agent_id: &str, agent_name: &str, age_seconds: f64) {
    AGENT_HEARTBEAT_AGE_SECONDS
        .with_label_values(&[agent_id, agent_name])
        .set(age_seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_registers_all_metrics() {
        init();

        // Record at least one value for each counter/histogram to make them visible
        record_http_request("/test", "GET", 200, 0.01);
        record_db_query("select", 0.001);
        set_active_agents(0);
        set_agent_heartbeat_age("test-id", "test-agent", 0.0);
        set_stacks_total(0);
        set_deployment_objects_total(0);

        let output = encode_metrics();

        // Verify all metric definitions are present
        assert!(
            output.contains("brokkr_http_requests_total"),
            "Should contain HTTP requests counter"
        );
        assert!(
            output.contains("brokkr_http_request_duration_seconds"),
            "Should contain HTTP request duration histogram"
        );
        assert!(
            output.contains("brokkr_database_queries_total"),
            "Should contain database queries counter"
        );
        assert!(
            output.contains("brokkr_database_query_duration_seconds"),
            "Should contain database query duration histogram"
        );
        assert!(
            output.contains("brokkr_active_agents"),
            "Should contain active agents gauge"
        );
        assert!(
            output.contains("brokkr_agent_heartbeat_age_seconds"),
            "Should contain agent heartbeat age gauge"
        );
        assert!(
            output.contains("brokkr_stacks_total"),
            "Should contain stacks total gauge"
        );
        assert!(
            output.contains("brokkr_deployment_objects_total"),
            "Should contain deployment objects total gauge"
        );
    }

    #[test]
    fn test_normalize_endpoint_replaces_uuids() {
        let endpoint = "/api/v1/agents/550e8400-e29b-41d4-a716-446655440000";
        let normalized = normalize_endpoint(endpoint);
        assert_eq!(normalized, "/api/v1/agents/:id");
    }

    #[test]
    fn test_normalize_endpoint_replaces_numeric_ids() {
        let endpoint = "/api/v1/items/12345";
        let normalized = normalize_endpoint(endpoint);
        assert_eq!(normalized, "/api/v1/items/:id");
    }

    #[test]
    fn test_normalize_endpoint_preserves_regular_paths() {
        let endpoint = "/api/v1/agents";
        let normalized = normalize_endpoint(endpoint);
        assert_eq!(normalized, "/api/v1/agents");

        let endpoint2 = "/healthz";
        let normalized2 = normalize_endpoint(endpoint2);
        assert_eq!(normalized2, "/healthz");
    }

    #[test]
    fn test_record_http_request_increments_counter() {
        init();
        record_http_request("/api/v1/test", "GET", 200, 0.05);

        let output = encode_metrics();
        assert!(
            output.contains("brokkr_http_requests_total"),
            "Should have HTTP requests metric"
        );
        // The counter should have been incremented
        assert!(
            output.contains("endpoint=\"/api/v1/test\"") || output.contains("method=\"GET\""),
            "Should have request labels"
        );
    }

    #[test]
    fn test_set_active_agents() {
        init();
        set_active_agents(5);

        let output = encode_metrics();
        assert!(
            output.contains("brokkr_active_agents 5"),
            "Should have active agents set to 5"
        );
    }

    #[test]
    fn test_set_stacks_total() {
        init();
        set_stacks_total(10);

        let output = encode_metrics();
        assert!(
            output.contains("brokkr_stacks_total 10"),
            "Should have stacks total set to 10"
        );
    }
}
