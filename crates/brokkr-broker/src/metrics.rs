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

/// Encodes all registered metrics in Prometheus text format
///
/// # Returns
///
/// Returns a String containing all metrics in Prometheus exposition format
pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");
    String::from_utf8(buffer).expect("Failed to convert metrics to UTF-8")
}
