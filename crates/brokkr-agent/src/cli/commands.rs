use crate::broker;
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
    info!("Broker is ready");

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
                            //  TODO: Process deployment object
                            todo!("Process deployment object: {}", obj.id);
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
