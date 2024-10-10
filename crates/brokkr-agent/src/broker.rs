use brokkr_utils::Settings;
use reqwest::Client;
use brokkr_utils::logging::prelude::*;
use tokio::time::sleep;
use std::time::Duration;
use reqwest::StatusCode;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::DeploymentObject;

pub async fn wait_for_broker_ready(config: &Settings) {
    let client = Client::new();
    let readyz_url = format!("{}/readyz", config.agent.broker_url);

    for attempt in 1..=config.agent.max_retries{
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
            sleep(Duration::from_secs(1)).await;
        }
    }
    error!("Failed to connect to broker after {} attempts. Exiting.", config.agent.max_retries);
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
            error!("Agent PAK verification failed with status: {}", response.status());
            Err(format!("PAK verification failed with status: {}", response.status()).into())
        }
    }
}

pub async fn fetch_agent_details(config: &Settings, client: &Client) -> Result<Agent, Box<dyn std::error::Error>> {
    let agent_url = format!(
        "{}/api/v1/agents/?name={}&cluster_name={}",
        config.agent.broker_url,
        config.agent.agent_name,
        config.agent.cluster_name
    );

    let response = client
        .get(&agent_url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    if response.status().is_success() {
        let agents: Vec<Agent> = response.json().await?;
        if let Some(agent) = agents.first() {
            info!("Successfully fetched agent details for {}", agent.name);
            Ok(agent.clone())
        } else {
            error!("No agent found with the given name and cluster name");
            Err("No agent found".into())
        }
    } else {
        error!("Failed to fetch agent details: {}", response.status());
        Err(format!("Failed to fetch agent details: {}", response.status()).into())
    }
}

pub async fn fetch_and_process_deployment_objects(config: &Settings, client: &Client, agent: &Agent) -> Result<Vec<DeploymentObject>, Box<dyn std::error::Error>> {
    let applicable_objects_url = format!(
        "{}/api/v1/agents/{}/applicable-deployment-objects",
        config.agent.broker_url,
        agent.id
    );

    let response = client
        .get(&applicable_objects_url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await?;

    if response.status().is_success() {
        let deployment_objects: Vec<DeploymentObject> = response.json().await?;
        info!("Fetched {} applicable deployment objects", deployment_objects.len());
        Ok((deployment_objects))
    } else {
        error!("Failed to fetch applicable deployment objects: {}", response.status());
        Err(format!("Failed to fetch applicable deployment objects: {}", response.status()).into())
    }
}