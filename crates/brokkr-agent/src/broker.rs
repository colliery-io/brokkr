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

pub async fn verify_agent_pak(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let auth_url = format!("{}/api/v1/auth/pak", config.agent.broker_url);

    info!("Verifying agent PAK with broker");
    let response = client
        .post(&auth_url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            info!("Agent PAK verified successfully");
            Ok(())
        }
        StatusCode::UNAUTHORIZED => {
            error!("Agent PAK verification failed: Unauthorized");
            Err("Unauthorized PAK".into())
        }
        _ => {
            error!(
                "Agent PAK verification failed with status: {}",
                response.status()
            );
            Err(format!("PAK verification failed with status: {}", response.status()).into())
        }
    }
}

pub async fn fetch_agent_details(
    config: &Settings,
    client: &Client,
) -> Result<Agent, Box<dyn std::error::Error>> {
    let agent_url = format!(
        "{}/api/v1/agents/?name={}&cluster_name={}",
        config.agent.broker_url, config.agent.agent_name, config.agent.cluster_name
    );

    let response = client
        .get(&agent_url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    if response.status().is_success() {
        let agent: Agent = response.json().await?;
        Ok(agent)
    } else {
        error!("Failed to fetch agent details: {}", response.status());
        Err(format!("Failed to fetch agent details: {}", response.status()).into())
    }
}

pub async fn fetch_and_process_deployment_objects(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<Vec<DeploymentObject>, Box<dyn std::error::Error>> {
    let applicable_objects_url = format!(
        "{}/api/v1/agents/{}/applicable-deployment-objects",
        config.agent.broker_url, agent.id
    );

    let response = client
        .get(&applicable_objects_url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    if response.status().is_success() {
        let deployment_objects: Vec<DeploymentObject> = response.json().await?;
        info!(
            "Fetched {} applicable deployment objects",
            deployment_objects.len()
        );
        Ok(deployment_objects)
    } else {
        error!(
            "Failed to fetch applicable deployment objects: {}",
            response.status()
        );
        Err(format!(
            "Failed to fetch applicable deployment objects: {}",
            response.status()
        )
        .into())
    }
}

pub async fn send_success_event(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    deployment_object_id: Uuid,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    send_event(
        config,
        client,
        agent,
        deployment_object_id,
        "DEPLOY",
        "SUCCESS",
        message,
    )
    .await
}

pub async fn send_failure_event(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    deployment_object_id: Uuid,
    error_message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    send_event(
        config,
        client,
        agent,
        deployment_object_id,
        "DEPLOY",
        "FAILURE",
        Some(error_message),
    )
    .await
}

async fn send_event(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    deployment_object_id: Uuid,
    event_type: &str,
    status: &str,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_url = format!(
        "{}/api/v1/agents/{}/events",
        config.agent.broker_url, agent.id
    );

    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object_id,
        event_type.to_string(),
        status.to_string(),
        message,
    )?;

    for attempt in 1..=config.agent.max_event_message_retries {
        let response = client
            .post(&event_url)
            .header("Authorization", format!("Bearer {}", config.agent.pak))
            .json(&new_event)
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                "Successfully sent {} event for deployment object {}",
                status, deployment_object_id
            );
            return Ok(());
        } else {
            error!(
                "Failed to send {} event (attempt {}): {}",
                status,
                attempt,
                response.status()
            );
            if attempt < config.agent.max_event_message_retries {
                sleep(Duration::from_secs(config.agent.event_message_retry_delay)).await;
            }
        }
    }

    Err(format!(
        "Failed to send {} event after {} attempts",
        status, config.agent.max_event_message_retries
    )
    .into())
}
