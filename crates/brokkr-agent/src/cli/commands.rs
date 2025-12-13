/*
 * Copyright (c) 2025 Dylan Storey
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

use crate::{broker, health, k8s, work_orders};
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new(None).expect("Failed to load configuration");
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");
    info!("Starting Brokkr Agent");

    info!("Waiting for broker to be ready");
    broker::wait_for_broker_ready(&config).await;

    info!("Verifying agent PAK");
    broker::verify_agent_pak(&config).await?;
    info!("Agent PAK verified successfully");

    let client = Client::new();
    info!("HTTP client created");

    info!("Fetching agent details");
    let mut agent = broker::fetch_agent_details(&config, &client).await?;
    info!(
        "Agent details fetched successfully for agent: {}",
        agent.name
    );

    // Initialize Kubernetes client
    info!("Initializing Kubernetes client");
    let k8s_client = k8s::api::create_k8s_client(config.agent.kubeconfig_path.as_deref())
        .await
        .expect("Failed to create Kubernetes client");

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

    // Set up ctrl-c handler
    tokio::spawn(async move {
        if let Ok(()) = ctrl_c().await {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(());
            r.store(false, Ordering::SeqCst);
        }
    });

    // Create interval timers for periodic tasks
    let mut heartbeat_interval = interval(Duration::from_secs(config.agent.polling_interval));
    let mut deployment_check_interval =
        interval(Duration::from_secs(config.agent.polling_interval));
    let mut work_order_interval = interval(Duration::from_secs(config.agent.polling_interval));

    // Main control loop
    while running.load(Ordering::SeqCst) {
        select! {
            _ = heartbeat_interval.tick() => {
                match broker::send_heartbeat(&config, &client, &agent).await {
                    Ok(_) => {
                        debug!("Successfully sent heartbeat for agent '{}' (id: {})", agent.name, agent.id);
                        // Update broker status for health endpoints
                        {
                            let mut status = broker_status.write().await;
                            status.connected = true;
                            status.last_heartbeat = Some(chrono::Utc::now().to_rfc3339());
                        }
                        // Fetch updated agent details after heartbeat
                        match broker::fetch_agent_details(&config, &client).await {
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

                match broker::fetch_and_process_deployment_objects(&config, &client, &agent).await {
                    Ok(objects) => {
                        for obj in objects {
                            let k8s_objects = k8s::objects::create_k8s_objects(obj.clone(),agent.id)?;
                            match k8s::api::reconcile_target_state(
                                &k8s_objects,
                                k8s_client.clone(),
                                &obj.stack_id.to_string(),
                                &obj.yaml_checksum,
                            ).await {
                                Ok(_) => {
                                    info!("Successfully applied {} Kubernetes objects for deployment object {} in agent '{}' (id: {})",
                                        k8s_objects.len(), obj.id, agent.name, agent.id);
                                    if let Err(e) = broker::send_success_event(
                                        &config,
                                        &client,
                                        &agent,
                                        obj.id,
                                        None,
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
                                        &client,
                                        &agent,
                                        obj.id,
                                        e.to_string(),
                                    ).await {
                                        error!("Failed to send failure event for deployment {} in agent '{}' (id: {}): {}",
                                            obj.id, agent.name, agent.id, send_err);
                                    }
                                }
                            }
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

                // Process pending work orders
                match work_orders::process_pending_work_orders(&config, &client, &k8s_client, &agent).await {
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

    Ok(())
}
