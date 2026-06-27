/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # CLI Commands Module
//!
//! Implements the command-line interface for the Brokkr agent.
//!
//! ## Main Command
//!
//! ```rust
//! pub async fn start() -> Result<(), Box<dyn std::error::Error>>
//! ```
//!
//! The primary entry point for the agent, which:
//! 1. Loads configuration
//! 2. Initializes logging
//! 3. Verifies broker connectivity
//! 4. Starts the main processing loop
//!
//! ## Startup Sequence
//!
//! ```mermaid
//! flowchart TD
//!     A[Load Config] --> B[Init Logger]
//!     B --> C[Wait for Broker]
//!     C --> D[Verify PAK]
//!     D --> E[Create HTTP Client]
//!     E --> F[Fetch Agent Details]
//!     F --> G[Start Main Loop]
//!
//!     G --> H{Process Deployments}
//!     H --> I[Apply Objects]
//!     I --> J[Report Status]
//!     J --> H
//! ```
//!
//! ## Signal Handling
//!
//! The module implements graceful shutdown handling:
//! - Catches SIGTERM/SIGINT signals
//! - Completes in-progress deployments
//! - Performs cleanup operations
//!
//! ## Configuration
//!
//! Supports configuration through:
//! - Environment variables
//! - Configuration files
//! - Command line arguments
//!
//! ## Logging
//!
//! Implements structured logging with:
//! - Multiple log levels
//! - JSON output format
//! - Contextual information

use crate::{
    broker, broker_sdk, broker_ws, deployment_health, diagnostics, health, k8s, kube_events,
    pod_logs, webhooks, work_orders,
};
use brokkr_utils::config::Settings;
use brokkr_wire::WsMessage;
use brokkr_utils::telemetry::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use uuid::Uuid;

/// What an inbound broker→agent WS push frame should trigger in the control
/// loop. Uplink-typed frames the agent emits never arrive inbound, so they map
/// to `Ignore`.
#[derive(Debug, PartialEq, Eq)]
enum PushAction {
    /// `StackChanged` / `TargetChanged` — fetch and reconcile deployment objects.
    Reconcile,
    /// `WorkOrder` — poll pending work orders.
    PollWorkOrders,
    /// Anything else (e.g. an echoed uplink frame) — no action.
    Ignore,
}

/// Route an inbound WS frame to the control-loop action it should trigger.
fn classify_push_frame(msg: &WsMessage) -> PushAction {
    match msg {
        WsMessage::StackChanged(_) | WsMessage::TargetChanged(_) => PushAction::Reconcile,
        WsMessage::WorkOrder(_) => PushAction::PollWorkOrders,
        _ => PushAction::Ignore,
    }
}

/// Resolve the comma-separated generator-scope string in precedence order:
///   1. `flag` — the `--generator-ids` CLI flag
///   2. `config` — `BROKKR__AGENT__GENERATOR_IDS` / `agent.generator_ids` (file)
///   3. `legacy_env` — the deprecated bare `BROKKR_GENERATOR_IDS` env var
///
/// Returns the resolved string and whether the deprecated legacy source supplied
/// a non-empty value (so the caller can emit a one-time deprecation warning).
fn resolve_generator_ids(
    flag: Option<String>,
    config: Option<String>,
    legacy_env: Option<String>,
) -> (String, bool) {
    if let Some(v) = flag {
        (v, false)
    } else if let Some(v) = config {
        (v, false)
    } else if let Some(v) = legacy_env {
        let used_legacy = !v.trim().is_empty();
        (v, used_legacy)
    } else {
        (String::new(), false)
    }
}

pub async fn start(
    generator_ids_override: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // BROKKR_CONFIG_FILE adds an optional file layer between embedded
    // defaults and BROKKR__ env vars (BROKKR-T-0187).
    let config = Settings::new(std::env::var("BROKKR_CONFIG_FILE").ok())
        .expect("Failed to load configuration");

    // Initialize telemetry (includes tracing/logging setup)
    let telemetry_config = config.telemetry.for_agent();
    brokkr_utils::telemetry::init(&telemetry_config, &config.log.level, &config.log.format)
        .expect("Failed to initialize telemetry");

    info!("Starting Brokkr Agent");

    info!("Waiting for broker to be ready");
    broker::wait_for_broker_ready(&config).await;

    info!("Verifying agent PAK");
    broker::verify_agent_pak(&config).await?;
    info!("Agent PAK verified successfully");

    let sdk_client = broker_sdk::build_client(&config)?;

    // Open the internal WS channel to the broker. Per ADR-0008 it's
    // opt-out: if the user set `agent.ws_force_rest = true`, this spawn
    // pins the state at ForceRestOnly and no dial is ever attempted.
    // Outbound emissions still call .uplink() — try_send short-circuits
    // when the channel is not Up and the caller falls back to REST.
    let mut ws_client = broker_ws::spawn(&config);
    let ws_uplink = ws_client.uplink();

    // Spawn telemetry tailers. They run for the lifetime of the agent
    // and use ws_uplink for emission. K8s client is created below and
    // shared with the tailers via spawn(). See WS-07.
    // (Spawn happens after `k8s_client` is created — see the call below.)
    info!("Broker SDK client created");

    info!("Fetching agent details");
    let mut agent = broker::fetch_agent_details(&config, &sdk_client).await?;
    info!(
        "Agent details fetched successfully for agent: {}",
        agent.name
    );

    // Resolve the generator scope to self-register with (see resolve_generator_ids
    // for precedence). Comma-separated UUIDs. 409 on register = already registered
    // (idempotent); other errors are logged but non-fatal.
    let (generator_ids_raw, used_legacy_env) = resolve_generator_ids(
        generator_ids_override,
        config.agent.generator_ids.clone(),
        std::env::var("BROKKR_GENERATOR_IDS").ok(),
    );
    if used_legacy_env {
        warn!(
            "BROKKR_GENERATOR_IDS is deprecated; set BROKKR__AGENT__GENERATOR_IDS \
             (or agent.generator_ids in config, or --generator-ids) instead"
        );
    }
    if !generator_ids_raw.trim().is_empty() {
        let mut registered = vec![];
        let mut failed = vec![];
        for part in generator_ids_raw.split(',') {
            let part = part.trim();
            match Uuid::parse_str(part) {
                Err(_) => warn!(%part, "generator_ids: skipping malformed UUID"),
                Ok(gid) => {
                    let result = sdk_client
                        .api()
                        .register_agent()
                        .id(gid)
                        .body_map(|b| b)
                        .send()
                        .await;
                    match result {
                        Ok(_) => registered.push(gid),
                        Err(e) => {
                            let status = e.status().map(|s| s.as_u16());
                            if status == Some(409) {
                                // Already registered — treat as success.
                                registered.push(gid);
                            } else {
                                error!(%gid, ?status, "failed to register with generator");
                                failed.push(gid);
                            }
                        }
                    }
                }
            }
        }
        info!(
            registered = ?registered,
            failed = ?failed,
            "generator registration complete"
        );
    }

    // Initialize Kubernetes client
    info!("Initializing Kubernetes client");
    let k8s_client = k8s::api::create_k8s_client(config.agent.kubeconfig_path.as_deref())
        .await
        .expect("Failed to create Kubernetes client");

    // WS-07: tail kube Events for objects this agent manages and stream
    // them upstream via the WS uplink. Always-on (no per-stack opt-in;
    // Events are cheap signal). See crates/brokkr-agent/src/kube_events.rs.
    let _kube_events_handle = kube_events::spawn(
        k8s_client.clone(),
        ws_uplink.clone(),
        agent.id,
        config
            .agent
            .kube_event_uid_cache_cap
            .unwrap_or(kube_events::DEFAULT_UID_CACHE_CAP),
        config.agent.watch_namespace.clone(),
    );

    // WS-08: tail pod logs for stacks that opt in via the
    // `brokkr.io/stream-logs: "true"` annotation on the pod template.
    // Rate-limited per container; over-rate lines surface as LogGap
    // markers so the UI renders visible gaps rather than swallowing data.
    let _pod_logs_handle = pod_logs::spawn(
        k8s_client.clone(),
        ws_uplink.clone(),
        agent.id,
        config.agent.watch_namespace.clone(),
    );

    // Initialize health state for health endpoints
    let broker_status = Arc::new(RwLock::new(health::BrokerStatus {
        connected: true,
        last_heartbeat: None,
    }));
    let health_state = health::HealthState {
        k8s_client: k8s_client.clone(),
        broker_status: broker_status.clone(),
        start_time: SystemTime::now(),
    };

    // Start health check HTTP server
    let health_port = config.agent.health_port.unwrap_or(8080);
    info!("Starting health check server on port {}", health_port);
    let health_router = health::configure_health_routes(health_state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", health_port))
        .await
        .expect("Failed to bind health check server");

    let _health_server = tokio::spawn(async move {
        axum::serve(listener, health_router)
            .await
            .expect("Health check server failed");
    });

    info!("Starting main control loop");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Create channels for shutdown coordination
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Shutdown on SIGINT *or* SIGTERM. Kubernetes terminates pods with
    // SIGTERM, so handling only ctrl-c (SIGINT) meant the agent died on the
    // default disposition with no graceful drain or telemetry flush.
    tokio::spawn(async move {
        let terminate = async {
            #[cfg(unix)]
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut s) => {
                    s.recv().await;
                }
                Err(e) => {
                    error!("failed to install SIGTERM handler: {}", e);
                    std::future::pending::<()>().await;
                }
            }
            #[cfg(not(unix))]
            std::future::pending::<()>().await;
        };
        select! {
            _ = ctrl_c() => info!("Received SIGINT; shutting down"),
            _ = terminate => info!("Received SIGTERM; shutting down"),
        }
        let _ = shutdown_tx.send(());
        r.store(false, Ordering::SeqCst);
    });

    // Create interval timers for periodic tasks
    let mut heartbeat_interval = interval(Duration::from_secs(config.agent.polling_interval));
    let mut deployment_check_interval =
        interval(Duration::from_secs(config.agent.polling_interval));
    let mut work_order_interval = interval(Duration::from_secs(config.agent.polling_interval));

    // Health checking configuration
    let health_check_enabled = config.agent.deployment_health_enabled.unwrap_or(true);
    let health_check_interval_secs = config.agent.deployment_health_interval.unwrap_or(60);
    let mut health_check_interval = interval(Duration::from_secs(health_check_interval_secs));

    // Track deployment objects we've applied for health checking
    let tracked_deployment_objects: Arc<RwLock<HashSet<Uuid>>> =
        Arc::new(RwLock::new(HashSet::new()));

    // Track stack IDs from the previous reconcile cycle. Used for gap detection:
    // if a stack disappears between cycles its K8s resources must be cleaned up.
    let mut previous_stack_ids: HashSet<Uuid> = HashSet::new();

    // Create health checker
    let health_checker = deployment_health::HealthChecker::new(k8s_client.clone())
        .with_watch_namespace(config.agent.watch_namespace.clone());

    if health_check_enabled {
        info!(
            "Deployment health checking enabled with {}s interval",
            health_check_interval_secs
        );
    } else {
        info!("Deployment health checking is disabled");
    }

    // Diagnostics configuration - poll every 10 seconds for diagnostic requests
    let mut diagnostics_interval = interval(Duration::from_secs(10));
    let diagnostics_handler = diagnostics::DiagnosticsHandler::new(k8s_client.clone());

    // Webhook delivery configuration - poll every 10 seconds for pending webhooks
    let mut webhook_interval = interval(Duration::from_secs(10));

    // Consume broker→agent push frames. A StackChanged/TargetChanged/WorkOrder
    // pushed over the WS channel resets the relevant interval to fire now, so
    // the agent reconciles on push instead of waiting for the next tick. Single
    // consumer; if the WS task ends we fall back to plain interval polling.
    let mut inbound_rx = ws_client.take_inbound();
    let mut inbound_open = inbound_rx.is_some();

    // In-flight work-order pass, if any. Keeps long builds off the select loop
    // (see the work_order_interval arm) while ensuring only one pass runs at a
    // time.
    let mut work_order_task: Option<tokio::task::JoinHandle<()>> = None;

    // Main control loop
    while running.load(Ordering::SeqCst) {
        select! {
            maybe_frame = async {
                match inbound_rx.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending::<Option<WsMessage>>().await,
                }
            }, if inbound_open => {
                match maybe_frame {
                    Some(frame) => match classify_push_frame(&frame) {
                        PushAction::Reconcile => {
                            debug!("WS push: control-plane change — reconciling immediately");
                            deployment_check_interval.reset_immediately();
                        }
                        PushAction::PollWorkOrders => {
                            debug!("WS push: work order — polling immediately");
                            work_order_interval.reset_immediately();
                        }
                        PushAction::Ignore => debug!("WS push: ignoring non-control frame"),
                    },
                    None => {
                        debug!("WS inbound channel closed; falling back to interval polling");
                        inbound_open = false;
                    }
                }
            }
            _ = heartbeat_interval.tick() => {
                // BROKKR-T-0227: probe whether we can reach our own K8s API and
                // self-report it on the heartbeat. Probe failure → reachable=false.
                let reachability = k8s::api::probe_k8s_reachability(&k8s_client).await;
                match broker::send_heartbeat(
                    &config,
                    &sdk_client,
                    &agent,
                    Some(&ws_uplink),
                    Some(reachability.reachable),
                    reachability.latency_ms,
                ).await {
                    Ok(_) => {
                        debug!("Successfully sent heartbeat for agent '{}' (id: {})", agent.name, agent.id);
                        // Update broker status for health endpoints
                        {
                            let mut status = broker_status.write().await;
                            status.connected = true;
                            status.last_heartbeat = Some(chrono::Utc::now().to_rfc3339());
                        }
                        // Fetch updated agent details after heartbeat
                        match broker::fetch_agent_details(&config, &sdk_client).await {
                            Ok(updated_agent) => {
                                debug!("Successfully fetched updated agent details. Status: {}", updated_agent.status);
                                agent = updated_agent;
                            }
                            Err(e) => error!("Failed to fetch updated agent details: {}", e),
                        }
                    },
                    Err(e) => {
                        error!("Failed to send heartbeat for agent '{}' (id: {}): {}", agent.name, agent.id, e);
                        // Update broker status for health endpoints
                        let mut status = broker_status.write().await;
                        status.connected = false;
                    }
                }
            }
            _ = deployment_check_interval.tick() => {
                // Skip deployment object requests if agent is inactive
                if agent.status != "ACTIVE" {
                    debug!("Agent '{}' (id: {}) is not active (status: {}), skipping deployment object requests",
                        agent.name, agent.id, agent.status);
                    continue;
                }

                match broker::fetch_and_process_deployment_objects(&config, &sdk_client, &agent).await {
                    Ok(objects) => {
                        // Gap detection: find stacks present last cycle but absent now.
                        // Their agent_targets were removed (e.g. generator deregistration)
                        // so we must clean up their K8s resources locally — no broker call.
                        let current_stack_ids: HashSet<Uuid> =
                            objects.iter().map(|o| o.stack_id).collect();
                        let removed_stacks: Vec<Uuid> = previous_stack_ids
                            .difference(&current_stack_ids)
                            .copied()
                            .collect();
                        for stack_id in &removed_stacks {
                            info!(%stack_id, "stack removed from targets; cleaning up K8s resources");
                            if let Err(e) = k8s::api::delete_stack_resources(
                                &stack_id.to_string(),
                                k8s_client.clone(),
                                &agent.id,
                                config.agent.watch_namespace.as_deref(),
                            )
                            .await
                            {
                                error!(%stack_id, "failed to clean up resources for removed stack: {}", e);
                            }
                        }
                        previous_stack_ids = current_stack_ids;

                        // Collect this cycle's successfully-applied ids and
                        // replace the tracked set at the end, so superseded
                        // deployment objects stop being health-checked and the
                        // set can't grow without bound (was insert-only).
                        let mut applied_ids: Vec<Uuid> = Vec::new();
                        for obj in objects {
                            // A malformed bundle must not crash the agent: log it,
                            // report a failure event, and move on. Otherwise the
                            // `?` exits the process and the restart re-fetches the
                            // same bad object — a permanent crash loop.
                            let k8s_objects = match k8s::objects::create_k8s_objects(obj.clone(), agent.id) {
                                Ok(objs) => objs,
                                Err(e) => {
                                    error!("Failed to parse deployment object {} into Kubernetes objects for agent '{}' (id: {}): {}",
                                        obj.id, agent.name, agent.id, e);
                                    if let Err(send_err) = broker::send_failure_event(
                                        &config,
                                        &sdk_client,
                                        &agent,
                                        obj.id,
                                        e.to_string(),
                                        Some(&ws_uplink),
                                    ).await {
                                        error!("Failed to send failure event for deployment {} in agent '{}' (id: {}): {}",
                                            obj.id, agent.name, agent.id, send_err);
                                    }
                                    continue;
                                }
                            };
                            match k8s::api::reconcile_target_state(
                                &k8s_objects,
                                k8s_client.clone(),
                                &obj.stack_id.to_string(),
                                &obj.yaml_checksum,
                                &agent.id,
                                config.agent.watch_namespace.as_deref(),
                            ).await {
                                Ok(_) => {
                                    info!("Successfully applied {} Kubernetes objects for deployment object {} in agent '{}' (id: {})",
                                        k8s_objects.len(), obj.id, agent.name, agent.id);

                                    // Track this deployment object for health checking
                                    applied_ids.push(obj.id);

                                    if let Err(e) = broker::send_success_event(
                                        &config,
                                        &sdk_client,
                                        &agent,
                                        obj.id,
                                        None,
                                        Some(&ws_uplink),
                                    ).await {
                                        error!("Failed to send success event for deployment {} in agent '{}' (id: {}): {}",
                                            obj.id, agent.name, agent.id, e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to apply Kubernetes objects for deployment {} in agent '{}' (id: {}). Error: {}",
                                        obj.id, agent.name, agent.id, e);
                                    if let Err(send_err) = broker::send_failure_event(
                                        &config,
                                        &sdk_client,
                                        &agent,
                                        obj.id,
                                        e.to_string(),
                                        Some(&ws_uplink),
                                    ).await {
                                        error!("Failed to send failure event for deployment {} in agent '{}' (id: {}): {}",
                                            obj.id, agent.name, agent.id, send_err);
                                    }
                                }
                            }
                        }
                        // Rebuild the health-tracking set to exactly this
                        // cycle's applied objects.
                        {
                            let mut tracked = tracked_deployment_objects.write().await;
                            *tracked = applied_ids.into_iter().collect();
                        }
                    }
                    Err(e) => error!("Failed to fetch deployment objects for agent '{}' (id: {}): {}",
                        agent.name, agent.id, e),
                }
            }
            _ = work_order_interval.tick() => {
                // Skip work order processing if agent is inactive
                if agent.status != "ACTIVE" {
                    debug!("Agent '{}' (id: {}) is not active (status: {}), skipping work order processing",
                        agent.name, agent.id, agent.status);
                    continue;
                }

                // Work-order processing (especially image builds, up to 15 min)
                // must not run inline: a blocked select arm starves heartbeats
                // and deployment polling, so the broker marks a healthy agent
                // offline. Run it in a detached task and keep the loop ticking.
                // Only one pass runs at a time — a still-running pass means new
                // work waits for the next tick rather than processing twice.
                if work_order_task.as_ref().is_some_and(|h| !h.is_finished()) {
                    debug!("Work-order pass still running; skipping this tick");
                } else {
                    let config = config.clone();
                    let sdk_client = sdk_client.clone();
                    let k8s_client = k8s_client.clone();
                    let agent = agent.clone();
                    work_order_task = Some(tokio::spawn(async move {
                        match work_orders::process_pending_work_orders(&config, &sdk_client, &k8s_client, &agent).await {
                            Ok(count) => {
                                if count > 0 {
                                    info!("Processed {} work orders for agent '{}' (id: {})",
                                        count, agent.name, agent.id);
                                }
                            }
                            Err(e) => {
                                error!("Failed to process work orders for agent '{}' (id: {}): {}",
                                    agent.name, agent.id, e);
                            }
                        }
                    }));
                }
            }
            _ = health_check_interval.tick(), if health_check_enabled => {
                // Skip health checking if agent is inactive
                if agent.status != "ACTIVE" {
                    debug!("Agent '{}' (id: {}) is not active, skipping health check",
                        agent.name, agent.id);
                    continue;
                }

                // Get the list of tracked deployment objects
                let deployment_ids: Vec<Uuid> = {
                    let tracked = tracked_deployment_objects.read().await;
                    tracked.iter().cloned().collect()
                };

                if deployment_ids.is_empty() {
                    debug!("No deployment objects to check health for");
                    continue;
                }

                debug!("Checking health for {} deployment objects", deployment_ids.len());

                // Check health of all tracked deployment objects
                let health_statuses = health_checker
                    .check_deployment_objects(&deployment_ids)
                    .await;

                // Convert to health updates for broker
                let health_updates: Vec<deployment_health::DeploymentObjectHealthUpdate> =
                    health_statuses.into_iter().map(|s| s.into()).collect();

                // Send health status to broker
                match broker::send_health_status(&config, &sdk_client, &agent, health_updates, Some(&ws_uplink)).await { Err(e) => {
                    error!("Failed to send health status for agent '{}': {}", agent.name, e);
                } _ => {
                    debug!("Successfully sent health status for {} deployment objects",
                        deployment_ids.len());
                }}
            }
            _ = diagnostics_interval.tick() => {
                // Skip diagnostics processing if agent is inactive
                if agent.status != "ACTIVE" {
                    debug!("Agent '{}' (id: {}) is not active, skipping diagnostics",
                        agent.name, agent.id);
                    continue;
                }

                // Fetch pending diagnostic requests
                match broker::fetch_pending_diagnostics(&config, &sdk_client, &agent).await {
                    Ok(requests) => {
                        for request in requests {
                            info!("Processing diagnostic request {} for deployment object {}",
                                request.id, request.deployment_object_id);

                            // Claim the request
                            match broker::claim_diagnostic_request(&config, &sdk_client, request.id).await {
                                Ok(_claimed) => {
                                    // Collect diagnostics. The namespaces to
                                    // search are derived from the deployment
                                    // object's manifests (BROKKR-T-0190);
                                    // documents without an explicit namespace
                                    // contribute "default".
                                    let label_selector = format!("brokkr.io/deployment-object-id={}", request.deployment_object_id);
                                    let namespaces = match broker::fetch_deployment_object(&sdk_client, request.deployment_object_id).await {
                                        Ok(obj) => crate::utils::manifest_namespaces(&obj.yaml_content),
                                        Err(e) => {
                                            warn!("Failed to fetch deployment object {} to derive diagnostic namespaces; falling back to 'default': {}",
                                                request.deployment_object_id, e);
                                            vec!["default".to_string()]
                                        }
                                    };

                                    match diagnostics_handler.collect_diagnostics_in(&namespaces, &label_selector).await {
                                        Ok(result) => {
                                            // Submit the result
                                            match broker::submit_diagnostic_result(
                                                &config,
                                                &sdk_client,
                                                request.id,
                                                result,
                                            ).await { Err(e) => {
                                                error!("Failed to submit diagnostic result for request {}: {}",
                                                    request.id, e);
                                            } _ => {
                                                info!("Successfully submitted diagnostic result for request {}",
                                                    request.id);
                                            }}
                                        }
                                        Err(e) => {
                                            error!("Failed to collect diagnostics for request {}: {}",
                                                request.id, e);
                                            // Submit an error result
                                            let error_result = diagnostics::SubmitDiagnosticResult {
                                                pod_statuses: "[]".to_string(),
                                                // Build via serde_json so an error message containing
                                                // quotes can't produce invalid JSON.
                                                events: serde_json::json!([{ "error": e.to_string() }])
                                                    .to_string(),
                                                log_tails: None,
                                                collected_at: chrono::Utc::now(),
                                            };
                                            if let Err(submit_err) = broker::submit_diagnostic_result(
                                                &config,
                                                &sdk_client,
                                                request.id,
                                                error_result,
                                            ).await {
                                                error!("Failed to submit diagnostic error result for request {}: {}",
                                                    request.id, submit_err);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to claim diagnostic request {}: {}",
                                        request.id, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch pending diagnostics: {}", e);
                    }
                }
            }
            _ = webhook_interval.tick() => {
                // Skip webhook processing if agent is inactive
                if agent.status != "ACTIVE" {
                    debug!("Agent '{}' (id: {}) is not active, skipping webhook delivery",
                        agent.name, agent.id);
                    continue;
                }

                // Process pending webhook deliveries
                match webhooks::process_pending_webhooks(&config, &sdk_client, &agent).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Processed {} webhook deliveries for agent '{}' (id: {})",
                                count, agent.name, agent.id);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to process webhook deliveries: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Initiating shutdown for agent '{}' (id: {})...", agent.name, agent.id);
                break;
            }
        }
    }

    info!(
        "Shutdown complete for agent '{}' (id: {})",
        agent.name, agent.id
    );

    // Shutdown telemetry, flushing any pending traces
    brokkr_utils::telemetry::shutdown();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use brokkr_models::models::{agent_targets::AgentTarget, stacks::Stack, work_orders::WorkOrder};
    use chrono::Utc;

    fn stack() -> Stack {
        Stack {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            name: "s".into(),
            description: None,
            generator_id: Uuid::new_v4(),
        }
    }

    fn target() -> AgentTarget {
        AgentTarget {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            stack_id: Uuid::new_v4(),
        }
    }

    fn work_order() -> WorkOrder {
        WorkOrder {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            work_type: "image_build".into(),
            yaml_content: String::new(),
            status: "pending".into(),
            claimed_by: None,
            claimed_at: None,
            claim_timeout_seconds: 300,
            max_retries: 3,
            retry_count: 0,
            backoff_seconds: 10,
            next_retry_after: None,
            last_error: None,
            last_error_at: None,
        }
    }

    #[test]
    fn stack_and_target_changes_trigger_reconcile() {
        assert_eq!(
            classify_push_frame(&WsMessage::StackChanged(stack())),
            PushAction::Reconcile
        );
        assert_eq!(
            classify_push_frame(&WsMessage::TargetChanged(target())),
            PushAction::Reconcile
        );
    }

    #[test]
    fn work_order_triggers_poll() {
        assert_eq!(
            classify_push_frame(&WsMessage::WorkOrder(work_order())),
            PushAction::PollWorkOrders
        );
    }

    #[test]
    fn uplink_frames_are_ignored() {
        let hb = WsMessage::Heartbeat(brokkr_wire::Heartbeat {
            agent_id: Uuid::nil(),
            sent_at: Utc::now(),
            k8s_reachable: None,
            k8s_api_latency_ms: None,
        });
        assert_eq!(classify_push_frame(&hb), PushAction::Ignore);
    }

    #[test]
    fn generator_ids_flag_wins_over_config_and_env() {
        let (resolved, legacy) = resolve_generator_ids(
            Some("flag".into()),
            Some("config".into()),
            Some("env".into()),
        );
        assert_eq!(resolved, "flag");
        assert!(!legacy);
    }

    #[test]
    fn generator_ids_config_wins_over_legacy_env() {
        let (resolved, legacy) =
            resolve_generator_ids(None, Some("config".into()), Some("env".into()));
        assert_eq!(resolved, "config");
        assert!(!legacy);
    }

    #[test]
    fn generator_ids_falls_back_to_legacy_env_and_flags_it() {
        let (resolved, legacy) = resolve_generator_ids(None, None, Some("env".into()));
        assert_eq!(resolved, "env");
        assert!(legacy, "non-empty legacy env should be flagged as deprecated use");
    }

    #[test]
    fn generator_ids_empty_legacy_env_is_not_flagged() {
        let (resolved, legacy) = resolve_generator_ids(None, None, Some("  ".into()));
        assert_eq!(resolved, "  ");
        assert!(!legacy, "blank legacy env should not trigger the deprecation warning");
    }

    #[test]
    fn generator_ids_default_when_all_absent() {
        let (resolved, legacy) = resolve_generator_ids(None, None, None);
        assert_eq!(resolved, "");
        assert!(!legacy);
    }
}
