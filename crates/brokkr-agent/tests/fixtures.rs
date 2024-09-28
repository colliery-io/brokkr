use brokkr_utils::Settings;
use reqwest::Client;
use brokkr_models::models::agents::{NewAgent, Agent};
use std::fs;
use reqwest::StatusCode;
use serde_json;
use reqwest::Body;
pub struct TestFixture {
    pub settings: Settings,
    pub admin_pak: String,
    pub test_agent_pak: String,
    pub test_agent: Agent,
    pub client: Client,
}

impl TestFixture {
    pub async fn new() -> Self {
        let client = Client::new();
        // Load default settings
        let settings = Settings::new(None).expect("Failed to load default settings");
        
        let new_agent = NewAgent::new("test-agent".to_string(), "test-cluster".to_string())
            .expect("Failed to create NewAgent");

        let admin_pak = fs::read_to_string("/tmp/key.txt").expect("Failed to read admin PAK");
        let admin_pak = admin_pak.trim().to_string();
        println!("Admin PAK: {}", admin_pak);
    
        // Wait for the broker to be ready
        let max_retries = 30;
        let retry_interval = std::time::Duration::from_secs(1);
        let mut retries = 0;
        loop {
            let response = client
                .get(format!("{}/healthz", settings.agent.broker_url))
                .send()
                .await;
            match response {
                Ok(resp) if resp.status() == StatusCode::OK => {
                    break;
                }
                _ => {
                    retries += 1;
                    if retries >= max_retries {
                        panic!("Broker failed to become ready after {} attempts", max_retries);
                    }
                    tokio::time::sleep(retry_interval).await;
                }
            }
        }
        let url = format!("{}/api/v1/agents", settings.agent.broker_url);
        let body = serde_json::to_string(&new_agent).unwrap();

        let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization",  admin_pak.clone())
        .body(Body::from(body.clone()))
        .send()
        .await
        .unwrap()
        ;

        println!(
            "Executing request: POST {} with headers: {{'Content-Type': 'application/json', 'Authorization': '{}'}} and body: {}",
            settings.agent.broker_url,
            admin_pak,
            body
        );

        assert_eq!(response.status(), StatusCode::OK);

        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        let test_agent: Agent = serde_json::from_value(json["agent"].clone()).expect("Failed to parse agent");
        let test_agent_pak = json["initial_pak"].as_str().expect("Failed to get initial PAK").to_string();

        TestFixture {
            settings,
            admin_pak,
            test_agent_pak,
            test_agent,
            client,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = TestFixture::new().await;
        
        assert!(!fixture.admin_pak.is_empty(), "Admin PAK should not be empty");
        assert!(!fixture.test_agent_pak.is_empty(), "Test agent PAK should not be empty");
        assert_eq!(fixture.test_agent.name, "test-agent", "Test agent name should be 'Test Agent'");
        assert_eq!(fixture.test_agent.cluster_name, "test-cluster", "Test agent cluster should be 'Test Cluster'");
    }
}

