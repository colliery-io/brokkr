use crate::broker;
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use reqwest::Client;

pub async fn register(
    admin_pak: String,
    agent_name: String,
    cluster_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new(None).expect("Failed to load configuration");
    let client = Client::new();

    info!("Waiting for broker to be ready");
    broker::wait_for_broker_ready(&config).await;
    info!("Broker is ready");

    let new_agent = brokkr_models::models::agents::NewAgent::new(agent_name, cluster_name)?;

    let response = client
        .post(&format!("{}/api/v1/agents", config.agent.broker_url))
        .header("Authorization", format!("Bearer {}", admin_pak))
        .json(&new_agent)
        .send()
        .await?;

    if response.status().is_success() {
        let agent: serde_json::Value = response.json().await?;
        println!("Agent registered successfully:");
        println!("Agent: {}", serde_json::to_string_pretty(&agent["agent"])?);
        println!("Initial PAK: {}", agent["initial_pak"]);

        // Update the PAK in the configuration
        std::env::set_var("BROKKR__AGENT_PAK", agent["initial_pak"].as_str().unwrap());
    } else {
        let error_message = response.text().await?;
        eprintln!("Failed to register agent: {}", error_message);
    }

    Ok(())
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new(None).expect("Failed to load configuration");
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");
    info!("Starting Brokkr Agent");

    // The rest of the agent logic remains the same as in the original main function
    // We'll just include a small part of it here for brevity

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

    // ... (rest of the agent logic)

    Ok(())
}
