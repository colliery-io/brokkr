/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # API Module
//!
//! This module handles the API routes and configurations for the Brokkr Broker.
//! It includes versioned API endpoints and middleware for authentication and request handling.
//!
//! ## Submodules
//!
//! - `v1`: Contains the version 1 of the API endpoints.
//!
//! ## Main Functions
//!
//! - `configure_api_routes`: Sets up the main application router with all API routes.
//! - `healthz`: Health check endpoint handler.
//! - `readyz`: Ready check endpoint handler.
//! - `metrics`: Metrics endpoint handler.
//!
//! ## API Endpoints
//!
//! ### Authentication
//! - `POST /api/v1/auth/pak`: Verifies a Pre-Authentication Key (PAK).
//!   - Returns: AuthResponse with authentication details.
//!   - Required PAK: Any valid PAK (admin, agent, or generator).
//!
//! ### Agents
//! - `GET /api/v1/agents`: Lists all agents.
//!   - Returns: Array of Agent objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/agents`: Creates a new agent.
//!   - Returns: Created Agent object and initial PAK.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/agents/:id`: Retrieves a specific agent.
//!   - Returns: Agent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `PUT /api/v1/agents/:id`: Updates an existing agent.
//!   - Returns: Updated Agent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `DELETE /api/v1/agents/:id`: Soft deletes an agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/agents/:id/events`: Lists events for a specific agent.
//!   - Returns: Array of AgentEvent objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/events`: Creates a new event for a specific agent.
//!   - Returns: Created AgentEvent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `GET /api/v1/agents/:id/labels`: Lists labels for a specific agent.
//!   - Returns: Array of AgentLabel objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/labels`: Adds a new label to a specific agent.
//!   - Returns: Created AgentLabel object.
//!   - Required PAK: Admin PAK only.
//! - `DELETE /api/v1/agents/:id/labels/:label`: Removes a label from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK only.
//! - `GET /api/v1/agents/:id/annotations`: Lists annotations for a specific agent.
//!   - Returns: Array of AgentAnnotation objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/annotations`: Adds a new annotation to a specific agent.
//!   - Returns: Created AgentAnnotation object.
//!   - Required PAK: Admin PAK only.
//! - `DELETE /api/v1/agents/:id/annotations/:key`: Removes an annotation from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK only.
//! - `GET /api/v1/agents/:id/targets`: Lists targets for a specific agent.
//!   - Returns: Array of AgentTarget objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/targets`: Adds a new target to a specific agent.
//!   - Returns: Created AgentTarget object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `DELETE /api/v1/agents/:id/targets/:stack_id`: Removes a target from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/heartbeat`: Records a heartbeat for a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Matching Agent PAK.
//! - `GET /api/v1/agents/:id/applicable-deployment-objects`: Retrieves applicable deployment objects for a specific agent.
//!   - Returns: Array of DeploymentObject objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//!
//! ### Generators
//! - `GET /api/v1/generators`: Lists all generators.
//!   - Returns: Array of Generator objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/generators`: Creates a new generator.
//!   - Returns: Created Generator object and its PAK.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/generators/:id`: Retrieves a specific generator.
//!   - Returns: Generator object.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//! - `PUT /api/v1/generators/:id`: Updates an existing generator.
//!   - Returns: Updated Generator object.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//! - `DELETE /api/v1/generators/:id`: Soft deletes a generator.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//!
//! ### Stacks
//! - `GET /api/v1/stacks`: Lists all stacks.
//!   - Returns: Array of Stack objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/stacks`: Creates a new stack.
//!   - Returns: Created Stack object.
//!   - Required PAK: Admin PAK or Generator PAK (for self).
//! - `GET /api/v1/stacks/:id`: Retrieves a specific stack.
//!   - Returns: Stack object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `PUT /api/v1/stacks/:id`: Updates an existing stack.
//!   - Returns: Updated Stack object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id`: Soft deletes a stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/deployment-objects`: Lists deployment objects for a specific stack.
//!   - Returns: Array of DeploymentObject objects.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `POST /api/v1/stacks/:id/deployment-objects`: Creates a new deployment object for a specific stack.
//!   - Returns: Created DeploymentObject object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/labels`: Lists labels for a specific stack.
//!   - Returns: Array of StackLabel objects.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//! - `POST /api/v1/stacks/:id/labels`: Adds a new label to a specific stack.
//!   - Returns: Created StackLabel object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id/labels/:label`: Removes a label from a specific stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/annotations`: Lists annotations for a specific stack.
//!   - Returns: Array of StackAnnotation objects.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//! - `POST /api/v1/stacks/:id/annotations`: Adds a new annotation to a specific stack.
//!   - Returns: Created StackAnnotation object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id/annotations/:key`: Removes an annotation from a specific stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//!
//! ### Deployment Objects
//! - `GET /api/v1/deployment-objects/:id`: Retrieves a specific deployment object.
//!   - Returns: DeploymentObject object.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//!
//! ### Agent Events
//! - `GET /api/v1/agent-events`: Lists all agent events.
//!   - Returns: Array of AgentEvent objects.
//!   - Required PAK: Any valid PAK.
//! - `GET /api/v1/agent-events/:id`: Retrieves a specific agent event.
//!   - Returns: AgentEvent object.
//!   - Required PAK: Any valid PAK.

pub mod assets;
pub mod v1;
use crate::dal::DAL;
use crate::metrics;
use crate::ws::{
    ConnectionRegistry, FleetBroadcaster, LiveBroadcaster, RetentionConfig, fleet_subscribe_routes,
    internal_routes, spawn_eviction, subscribe_routes,
};
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use brokkr_utils::config::{Cors, ReloadableConfig};
use hyper::StatusCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::Level;

/// Configures and returns the main application router with all API routes
///
/// This function is responsible for setting up the entire API structure of the application.
/// It merges routes from all submodules and adds a health check endpoint.
///
/// # Arguments
///
/// * `dal` - An instance of the Data Access Layer
/// * `cors_config` - CORS configuration settings
/// * `reloadable_config` - Optional reloadable configuration for hot-reload support
///
/// # Returns
///
/// Returns a configured `Router` instance that includes all API routes and middleware.
pub fn configure_api_routes(
    dal: DAL,
    cors_config: &Cors,
    reloadable_config: Option<ReloadableConfig>,
) -> Router<DAL> {
    // Build a permissive CORS layer for health/metrics endpoints
    let root_cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let ws_registry: Arc<ConnectionRegistry> = ConnectionRegistry::new();
    let live_broadcaster: Arc<LiveBroadcaster> = LiveBroadcaster::new();
    let fleet_broadcaster: Arc<FleetBroadcaster> = FleetBroadcaster::new();

    // Periodic fleet live-push sweep (I-0028 Slice 2): the computed-signal half
    // of the hybrid trigger — re-broadcast agents whose backpressure/health
    // changed, complementing the event-driven producers. 20s cadence for v1.
    crate::utils::background_tasks::start_fleet_sweep_task(
        dal.clone(),
        ws_registry.clone(),
        fleet_broadcaster.clone(),
        20,
    );

    // Continuous eviction for the agent telemetry buffers. Hard 6h cap
    // per project_log_retention_stance; the worker is intentionally
    // fire-and-forget — production drops the JoinHandle and it runs for
    // the process lifetime.
    let _eviction_handle = spawn_eviction(dal.clone(), RetentionConfig::default_policy());

    let app = Router::new()
        .merge(v1::routes(dal.clone(), cors_config, reloadable_config))
        .merge(internal_routes(
            dal.clone(),
            ws_registry.clone(),
            live_broadcaster.clone(),
            fleet_broadcaster.clone(),
        ))
        .merge(subscribe_routes(dal.clone(), live_broadcaster.clone()))
        .merge(fleet_subscribe_routes(
            dal.clone(),
            fleet_broadcaster.clone(),
        ))
        // Make the registry + broadcasters available to v1 handlers
        // (post-commit push helpers in `ws::push`; fleet live-push producers
        // in `record_heartbeat`; future WS-13 metrics).
        .layer(axum::Extension(ws_registry))
        .layer(axum::Extension(live_broadcaster))
        .layer(axum::Extension(fleet_broadcaster))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        // Must be layered *after* the /readyz route so the handler can see it.
        .layer(axum::Extension(ReadinessCache::default()))
        .layer(root_cors)
        .layer(middleware::from_fn(metrics_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &hyper::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(
                    |response: &hyper::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status(),
                            latency_ms = latency.as_millis(),
                            "response"
                        );
                    },
                ),
        )
        // Outermost: turn any panic in a handler or inner layer (e.g. a DB
        // pool-acquisition failure) into a 500 response instead of dropping the
        // connection, which under load looks like the broker hanging up.
        .layer(CatchPanicLayer::new());

    // Operator console (brokkr-web) static serving + SPA fallback, on the OUTER
    // router so the `/api/v1` nest (and its auth) wins every route it owns
    // (BROKKR-T-0253). No-op-ish placeholder unless built with `--features embed-ui`.
    assets::attach(app)
}

/// Health check endpoint handler
///
/// This handler responds to GET requests at the "/healthz" endpoint.
/// It's used to verify that the API is up and running.
///
/// # Returns
///
/// Returns a 200 OK status code with "OK" in the body.
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// How long a `/readyz` result is reused before the database is probed again.
///
/// The kubelet probes every 5s per pod and the endpoint is unauthenticated, so
/// an uncached check would let anyone with network reach turn readiness into a
/// connection-pool amplifier. Two seconds keeps the answer fresh enough that a
/// real outage still shows up on the next probe.
const READYZ_CACHE_TTL: Duration = Duration::from_secs(2);

/// Budget for the readiness pool checkout.
///
/// The r2d2 pool is built with only `.max_size` (see [`crate::db`]), so a plain
/// `get()` waits out r2d2's 30s default connection timeout — an order of
/// magnitude past the chart's 3s probe timeout, which would make the probe fail
/// on timeout rather than on a clean 503. Bound the wait explicitly instead.
const READYZ_DB_TIMEOUT: Duration = Duration::from_millis(750);

/// Cached readiness verdict: `(observed_at, ready)`.
///
/// Owned by the router rather than being a process global so that each
/// `configure_api_routes` call — notably each integration-test router — gets its
/// own cache and cannot inherit another's verdict.
#[derive(Clone, Default)]
struct ReadinessCache(Arc<Mutex<Option<(Instant, bool)>>>);

impl ReadinessCache {
    /// Returns the cached verdict if it is still within the TTL.
    fn get(&self) -> Option<bool> {
        let guard = self.0.lock().ok()?;
        let (observed_at, ready) = (*guard)?;
        (observed_at.elapsed() < READYZ_CACHE_TTL).then_some(ready)
    }

    /// Records a verdict for subsequent probes inside the TTL window.
    fn store(&self, ready: bool) {
        if let Ok(mut guard) = self.0.lock() {
            *guard = Some((Instant::now(), ready));
        }
    }
}

/// Checks that the database is reachable: a bounded pool checkout plus
/// `SELECT 1`.
///
/// Deliberately does *not* verify migrations — those run once at startup and
/// abort the process on failure, so re-checking them per probe buys nothing.
async fn check_db_ready(dal: &DAL) -> Result<(), String> {
    let pool = dal.pool.pool.clone();

    // Both the checkout and the query are blocking, so keep them off the async
    // runtime's worker threads.
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;

        let mut conn = pool
            .get_timeout(READYZ_DB_TIMEOUT)
            .map_err(|e| format!("database pool checkout failed: {}", e))?;

        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .map(|_| ())
            .map_err(|e| format!("database readiness query failed: {}", e))
    })
    .await
    .map_err(|e| format!("readiness check task failed: {}", e))?
}

/// Ready check endpoint handler
///
/// This handler responds to GET requests at the "/readyz" endpoint. Readiness
/// gates Service endpoints, so it reports whether the broker can actually serve
/// traffic — which means the database must be reachable.
///
/// Deliberately distinct from `/healthz`: liveness stays process-only, because
/// a database blip that restart-loops every broker pod turns a recoverable
/// outage into a total one.
///
/// # Returns
///
/// 200 OK with "Ready" when the database answers, 503 Service Unavailable with
/// a short reason otherwise. Results are cached for [`READYZ_CACHE_TTL`].
async fn readyz(
    State(dal): State<DAL>,
    axum::Extension(cache): axum::Extension<ReadinessCache>,
) -> impl IntoResponse {
    if let Some(ready) = cache.get() {
        return readyz_response(ready);
    }

    let ready = match check_db_ready(&dal).await {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("readiness check failed: {}", e);
            false
        }
    };

    cache.store(ready);
    readyz_response(ready)
}

/// Maps a readiness verdict onto the HTTP response the kubelet sees.
fn readyz_response(ready: bool) -> (StatusCode, &'static str) {
    if ready {
        (StatusCode::OK, "Ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "database unavailable")
    }
}

/// Metrics endpoint handler
///
/// This handler responds to GET requests at the "/metrics" endpoint.
/// It's used to provide Prometheus metrics about the broker's operation.
///
/// # Returns
///
/// Returns a 200 OK status code with Prometheus metrics in text format.
async fn metrics_handler() -> impl IntoResponse {
    let metrics_data = metrics::encode_metrics();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics_data,
    )
}

/// Middleware to record HTTP request metrics
///
/// Records request count and duration for each HTTP request.
async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Process the request
    let response = next.run(request).await;

    // Record metrics (skip the /metrics endpoint itself to avoid recursion)
    if path != "/metrics" {
        let duration = start.elapsed().as_secs_f64();
        let status = response.status().as_u16();
        metrics::record_http_request(&path, &method, status, duration);
    }

    response
}
