/// Module for interacting with the Brokkr broker service.
use brokkr_models::models::agent_events::NewAgentEvent;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::DeploymentObject;
use brokkr_utils::logging::prelude::*;
use brokkr_utils::Settings;
use reqwest::Client;
use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Waits for the broker service to become ready.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
pub async fn wait_for_broker_ready(config: &Settings) {
    let client = Client::new();
    let readyz_url = format!("{}/readyz", config.agent.broker_url);

    for attempt in 1..=config.agent.max_retries {
        match client.get(&readyz_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Broker is ready!");
                    return;
                }
            }
            Err(e) => {
                warn!("Error connecting to broker (attempt {}): {:?}", attempt, e);
            }
        }
        if attempt < config.agent.max_retries {
            info!("Waiting for broker to be ready (attempt {})", attempt);
            sleep(Duration::from_secs(1)).await;
        }
    }
    error!(
        "Failed to connect to broker after {} attempts. Exiting.",
        config.agent.max_retries
    );
    std::process::exit(1);
}

/// Verifies the agent's Personal Access Key (PAK) with the broker.
///
/// # Arguments
/// * `config` - Application settings containing the PAK
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn verify_agent_pak(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/auth/pak", config.agent.broker_url);
    debug!("Verifying agent PAK at {}", url);

    let response = reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .body("{}") // Empty JSON body
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send PAK verification request: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            info!("Successfully verified agent PAK");
            Ok(())
        }
        StatusCode::UNAUTHORIZED => {
            error!("Agent PAK verification failed: unauthorized");
            Err("Invalid agent PAK".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "PAK verification failed with status {}: {}",
                status, error_body
            );
            Err(format!(
                "PAK verification failed. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Fetches the details of the agent from the broker.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests to the broker
///
/// # Returns
/// * `Result<Agent, Box<dyn std::error::Error>>` - Agent details or error
pub async fn fetch_agent_details(
    config: &Settings,
    client: &Client,
) -> Result<Agent, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/?name={}&cluster_name={}",
        config.agent.broker_url, config.agent.agent_name, config.agent.cluster_name
    );
    debug!("Fetching agent details from {}", url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch agent details: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            let agent: Agent = response.json().await.map_err(|e| {
                error!("Failed to deserialize agent details: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

            info!(
                "Successfully fetched details for agent {} in cluster {}",
                agent.name, agent.cluster_name
            );

            Ok(agent)
        }
        StatusCode::NOT_FOUND => {
            error!(
                "Agent not found: name={}, cluster={}",
                config.agent.agent_name, config.agent.cluster_name
            );
            Err("Agent not found".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to fetch agent details. Status {}: {}",
                status, error_body
            );
            Err(format!(
                "Failed to fetch agent details. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Fetches applicable deployment objects from the broker for the given agent.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests to the broker
/// * `agent` - Agent details
///
/// # Returns
/// * `Result<Vec<DeploymentObject>, Box<dyn std::error::Error>>` - List of applicable deployment objects or error
pub async fn fetch_and_process_deployment_objects(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<Vec<DeploymentObject>, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/{}/applicable-deployment-objects",
        config.agent.broker_url, agent.id
    );

    debug!("Fetching deployment objects from {}", url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to broker: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            let deployment_objects: Vec<DeploymentObject> = response.json().await.map_err(|e| {
                error!("Failed to deserialize deployment objects: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

            info!(
                "Successfully fetched {} deployment objects for agent {}",
                deployment_objects.len(),
                agent.name
            );

            Ok(deployment_objects)
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Broker request failed with status {}: {}",
                status, error_body
            );
            Err(format!(
                "Broker request failed. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Sends a success event to the broker for the given deployment object.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests to the broker
/// * `agent` - Agent details
/// * `deployment_object_id` - ID of the deployment object
/// * `message` - Optional message to include in the event
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn send_success_event(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    deployment_object_id: Uuid,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/{}/events",
        config.agent.broker_url, agent.id
    );
    debug!(
        "Sending success event for deployment {} to {}",
        deployment_object_id, url
    );

    let event = NewAgentEvent {
        agent_id: agent.id,
        deployment_object_id,
        event_type: "DEPLOY".to_string(),
        status: "SUCCESS".to_string(),
        message,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .json(&event)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send success event: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK | StatusCode::CREATED => {
            info!(
                "Successfully reported deployment success for object {}",
                deployment_object_id
            );
            Ok(())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to send success event. Status {}: {}",
                status, error_body
            );
            Err(format!(
                "Failed to send success event. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Sends a failure event to the broker for the given deployment object.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests to the broker
/// * `agent` - Agent details
/// * `deployment_object_id` - ID of the deployment object
/// * `error_message` - Message to include in the event
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn send_failure_event(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    deployment_object_id: Uuid,
    error_message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/{}/events",
        config.agent.broker_url, agent.id
    );
    debug!(
        "Sending failure event for deployment {} to {}",
        deployment_object_id, url
    );

    let event = NewAgentEvent {
        agent_id: agent.id,
        deployment_object_id,
        event_type: "DEPLOY".to_string(),
        status: "FAILURE".to_string(),
        message: Some(error_message),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .json(&event)
        .send()
        .await
        .map_err(|e| {
            error!(
                "Failed to send failure event for deployment {}: {}",
                deployment_object_id, e
            );
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK | StatusCode::CREATED => {
            info!(
                "Successfully reported deployment failure for object {}",
                deployment_object_id
            );
            Ok(())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to send failure event. Status {}: {}",
                status, error_body
            );
            Err(format!(
                "Failed to send failure event. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Sends a heartbeat event to the broker for the given agent.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests to the broker
/// * `agent` - Agent details
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error with message
pub async fn send_heartbeat(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/{}/heartbeat",
        config.agent.broker_url, agent.id
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send heartbeat for agent {}: {}", agent.name, e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK | StatusCode::NO_CONTENT => {
            trace!("Heartbeat sent successfully for agent {}", agent.name);
            Ok(())
        }
        StatusCode::UNAUTHORIZED => {
            error!("Heartbeat unauthorized for agent {}", agent.name);
            Err("Unauthorized: Invalid agent PAK".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Heartbeat failed for agent {}. Status {}: {}",
                agent.name, status, error_body
            );
            Err(format!("Heartbeat failed. Status: {}, Body: {}", status, error_body).into())
        }
    }
}
