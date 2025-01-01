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

use crate::{broker, k8s};
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::select;
use tokio::signal::ctrl_c;
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
    let agent = broker::fetch_agent_details(&config, &client).await?;
    info!(
        "Agent details fetched successfully for agent: {}",
        agent.name
    );

    // Initialize Kubernetes client
    info!("Initializing Kubernetes client");
    let k8s_client = k8s::api::create_k8s_client(config.agent.kubeconfig_path.as_deref())
        .await
        .expect("Failed to create Kubernetes client");

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

    // Main control loop
    while running.load(Ordering::SeqCst) {
        select! {
            _ = heartbeat_interval.tick() => {
                match broker::send_heartbeat(&config, &client, &agent).await {
                    Ok(_) => debug!("Heartbeat sent successfully"),
                    Err(e) => error!("Failed to send heartbeat: {}", e),
                }
            }
            _ = deployment_check_interval.tick() => {
                match broker::fetch_and_process_deployment_objects(&config, &client, &agent).await {
                    Ok(objects) => {
                        for obj in objects {
                            let k8s_objects = k8s::objects::create_k8s_objects(obj.clone(),agent.id)?;
                            match k8s::api::apply_k8s_objects_with_rollback(&k8s_objects, k8s_client.clone()).await {
                                Ok(_) => {
                                    info!("Successfully applied Kubernetes objects");
                                    if let Err(e) = broker::send_success_event(
                                        &config,
                                        &client,
                                        &agent,
                                        obj.id,
                                        None,
                                    ).await {
                                        error!("Failed to send success event: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to apply Kubernetes objects with rollback: {}", e);
                                    if let Err(send_err) = broker::send_failure_event(
                                        &config,
                                        &client,
                                        &agent,
                                        obj.id,
                                        e.to_string(),
                                    ).await {
                                        error!("Failed to send failure event: {}", send_err);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Failed to fetch deployment objects: {}", e),
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Shutting down agent...");
                break;
            }
        }
    }

    info!("Agent shutdown complete");

    Ok(())
}
