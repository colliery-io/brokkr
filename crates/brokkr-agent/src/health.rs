/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Health Check Module
//!
//! This module provides HTTP health check endpoints for the Brokkr Agent.
//! These endpoints are used by Kubernetes for liveness and readiness probes.
//!
//! ## Endpoints
//!
//! - `GET /healthz`: Simple liveness check (returns 200 OK if process is alive)
//! - `GET /readyz`: Readiness check with Kubernetes API connectivity validation
//! - `GET /health`: Detailed health status with JSON response
//!
//! ## Health Status Structure
//!
//! The `/health` endpoint returns:
//! - Overall health status
//! - Kubernetes API connection status
//! - Broker connection status
//! - Service uptime
//! - Application version
//! - Timestamp

use crate::metrics;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use brokkr_utils::logging::prelude::*;
use kube::Client;
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Shared state for health endpoints
#[derive(Clone)]
pub struct HealthState {
    pub k8s_client: Client,
    pub broker_status: Arc<RwLock<BrokerStatus>>,
    pub start_time: SystemTime,
}

/// Broker connection status
#[derive(Clone)]
pub struct BrokerStatus {
    pub connected: bool,
    pub last_heartbeat: Option<String>,
}

/// Health status response structure
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    kubernetes: KubernetesStatus,
    broker: BrokerStatusResponse,
    uptime_seconds: u64,
    version: String,
    timestamp: String,
}

/// Kubernetes health status
#[derive(Serialize)]
struct KubernetesStatus {
    connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Broker health status for response
#[derive(Serialize)]
struct BrokerStatusResponse {
    connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_heartbeat: Option<String>,
}

/// Configures and returns the health check router
pub fn configure_health_routes(state: HealthState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

/// Simple liveness check endpoint
///
/// Returns 200 OK if the process is running.
/// This is used for Kubernetes liveness probes.
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Readiness check endpoint
///
/// Validates Kubernetes API connectivity.
/// Returns 200 OK if K8s API is accessible, 503 if not.
async fn readyz(State(state): State<HealthState>) -> impl IntoResponse {
    // Test Kubernetes API connectivity by checking API health
    match state.k8s_client.apiserver_version().await {
        Ok(_) => (StatusCode::OK, "Ready"),
        Err(e) => {
            error!("Kubernetes API connectivity check failed: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Kubernetes API unavailable",
            )
        }
    }
}

/// Detailed health check endpoint
///
/// Provides comprehensive JSON status including:
/// - Kubernetes API connectivity
/// - Broker connection status
/// - Service uptime
/// - Application version
/// - Timestamp
///
/// Returns 200 OK if all checks pass, 503 if any check fails.
async fn health(State(state): State<HealthState>) -> impl IntoResponse {
    // Get current timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Calculate uptime
    let uptime = now.as_secs().saturating_sub(
        state
            .start_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs(),
    );

    // Check Kubernetes API connectivity
    let (k8s_connected, k8s_error) = match state.k8s_client.apiserver_version().await {
        Ok(_) => (true, None),
        Err(e) => {
            error!("Kubernetes API connectivity check failed: {:?}", e);
            (false, Some(format!("{:?}", e)))
        }
    };

    // Get broker status
    let broker_status = state.broker_status.read().await;
    let broker_connected = broker_status.connected;
    let broker_last_heartbeat = broker_status.last_heartbeat.clone();

    // Determine overall status
    let overall_status = if k8s_connected && broker_connected {
        "healthy"
    } else {
        "unhealthy"
    };
    let status_code = if k8s_connected && broker_connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        kubernetes: KubernetesStatus {
            connected: k8s_connected,
            error: k8s_error,
        },
        broker: BrokerStatusResponse {
            connected: broker_connected,
            last_heartbeat: broker_last_heartbeat,
        },
        uptime_seconds: uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
    };

    (status_code, Json(health_status))
}

/// Prometheus metrics endpoint
///
/// Returns Prometheus metrics in text exposition format.
/// Metrics include broker polling, Kubernetes operations, and agent health.
async fn metrics_handler() -> impl IntoResponse {
    let metrics_data = metrics::encode_metrics();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics_data,
    )
}
