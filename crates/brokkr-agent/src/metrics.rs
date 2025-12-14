/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Metrics Module
//!
//! This module provides Prometheus metrics for the Brokkr Agent.
//! It exposes metrics about broker polling, Kubernetes operations, and agent health.

use prometheus::{
    CounterVec, Encoder, Gauge, HistogramOpts, HistogramVec, IntCounter, Opts, Registry,
    TextEncoder,
};
use std::sync::OnceLock;

/// Global Prometheus registry for all agent metrics
static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::new)
}

/// Broker poll request counter
/// Labels: status (success/error)
pub fn poll_requests_total() -> &'static CounterVec {
    static COUNTER: OnceLock<CounterVec> = OnceLock::new();
    COUNTER.get_or_init(|| {
        let opts = Opts::new(
            "brokkr_agent_poll_requests_total",
            "Total number of broker poll requests",
        );
        let counter =
            CounterVec::new(opts, &["status"]).expect("Failed to create poll requests counter");
        registry()
            .register(Box::new(counter.clone()))
            .expect("Failed to register poll requests counter");
        counter
    })
}

/// Broker poll duration histogram
pub fn poll_duration_seconds() -> &'static HistogramVec {
    static HISTOGRAM: OnceLock<HistogramVec> = OnceLock::new();
    HISTOGRAM.get_or_init(|| {
        let opts = HistogramOpts::new(
            "brokkr_agent_poll_duration_seconds",
            "Broker poll request latency distribution in seconds",
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0]);
        let histogram =
            HistogramVec::new(opts, &[]).expect("Failed to create poll duration histogram");
        registry()
            .register(Box::new(histogram.clone()))
            .expect("Failed to register poll duration histogram");
        histogram
    })
}

/// Kubernetes operations counter
/// Labels: operation (apply/delete/get/list)
pub fn kubernetes_operations_total() -> &'static CounterVec {
    static COUNTER: OnceLock<CounterVec> = OnceLock::new();
    COUNTER.get_or_init(|| {
        let opts = Opts::new(
            "brokkr_agent_kubernetes_operations_total",
            "Total number of Kubernetes API operations by type",
        );
        let counter = CounterVec::new(opts, &["operation"])
            .expect("Failed to create Kubernetes operations counter");
        registry()
            .register(Box::new(counter.clone()))
            .expect("Failed to register Kubernetes operations counter");
        counter
    })
}

/// Kubernetes operation duration histogram
/// Labels: operation (apply/delete/get/list)
pub fn kubernetes_operation_duration_seconds() -> &'static HistogramVec {
    static HISTOGRAM: OnceLock<HistogramVec> = OnceLock::new();
    HISTOGRAM.get_or_init(|| {
        let opts = HistogramOpts::new(
            "brokkr_agent_kubernetes_operation_duration_seconds",
            "Kubernetes operation latency distribution in seconds",
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);
        let histogram = HistogramVec::new(opts, &["operation"])
            .expect("Failed to create Kubernetes operation duration histogram");
        registry()
            .register(Box::new(histogram.clone()))
            .expect("Failed to register Kubernetes operation duration histogram");
        histogram
    })
}

/// Heartbeat sent counter
pub fn heartbeat_sent_total() -> &'static IntCounter {
    static COUNTER: OnceLock<IntCounter> = OnceLock::new();
    COUNTER.get_or_init(|| {
        let opts = Opts::new(
            "brokkr_agent_heartbeat_sent_total",
            "Total number of heartbeats sent to broker",
        );
        let counter = IntCounter::with_opts(opts).expect("Failed to create heartbeat counter");
        registry()
            .register(Box::new(counter.clone()))
            .expect("Failed to register heartbeat counter");
        counter
    })
}

/// Last successful poll timestamp (Unix timestamp)
pub fn last_successful_poll_timestamp() -> &'static Gauge {
    static GAUGE: OnceLock<Gauge> = OnceLock::new();
    GAUGE.get_or_init(|| {
        let opts = Opts::new(
            "brokkr_agent_last_successful_poll_timestamp",
            "Unix timestamp of last successful broker poll",
        );
        let gauge = Gauge::with_opts(opts).expect("Failed to create last poll timestamp gauge");
        registry()
            .register(Box::new(gauge.clone()))
            .expect("Failed to register last poll timestamp gauge");
        gauge
    })
}

/// Encodes all registered metrics in Prometheus text format
///
/// # Returns
///
/// Returns a String containing all metrics in Prometheus exposition format
pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = registry().gather();
    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");
    String::from_utf8(buffer).expect("Failed to convert metrics to UTF-8")
}
