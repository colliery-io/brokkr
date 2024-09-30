use brokkr_utils::Settings;
use reqwest::Client;
use brokkr_models::models::agents::{NewAgent, Agent};
use brokkr_models::models::stacks::{NewStack, Stack};
use brokkr_models::models::deployment_objects::{NewDeploymentObject, DeploymentObject};
use brokkr_models::models::generator::{NewGenerator, Generator};
use std::fs;
use uuid::Uuid;
use reqwest::StatusCode;
use serde_json;
use reqwest::Body;

use once_cell::sync::Lazy;
use std::sync::Arc;

static FIXTURE: Lazy<Arc<TestFixture>> = Lazy::new(|| {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    Arc::new(runtime.block_on(TestFixture::new()))
});
pub struct TestFixture {
    pub settings: Settings,
    pub admin_pak: String,
    pub test_agent_pak: String,
    pub test_agent: Agent,
    pub client: Client,
    pub stack: Stack,
    pub test_generator_pak: String,
    pub test_generator: Generator,
}

impl TestFixture {
    pub fn get() -> Arc<TestFixture> {
        FIXTURE.clone()
    }

    pub async fn new() -> Self {
        let client = Client::new();
        // Load default settings
        let settings = Settings::new(None).expect("Failed to load default settings");
        
        // Read the admin PAK from the new location
        let admin_pak = fs::read_to_string("/tmp/brokkr-keys/key.txt")
            .expect("Failed to read admin PAK")
            .trim()
            .to_string();
        
        broker_ready(&settings.agent.broker_url).await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;// wait a few seconds extra for everything to be ready.

        validate_pak(&admin_pak, &settings.agent.broker_url).await;

        let (test_generator, test_generator_pak) = create_generator(&client, &settings.agent.broker_url, &admin_pak, "integration-test-generator").await;
        
        let (test_agent, test_agent_pak) = create_agent(&settings.agent.broker_url, &admin_pak).await;
        let stack = create_stack(&client, &settings.agent.broker_url, &test_generator_pak, "integration-agent-test-stack", test_generator.id).await;

        TestFixture {
            settings,
            admin_pak,
            test_agent_pak,
            test_agent,
            client,
            stack,
            test_generator_pak,
            test_generator,
        }
    }

    pub async fn create_deployment_object(&self, yaml_content: &str) -> DeploymentObject {
        let new_deployment_object = NewDeploymentObject {
            stack_id: self.stack.id,
            yaml_content: yaml_content.to_string(),
            yaml_checksum: "test_checksum".to_string(),
            is_deletion_marker: false,
        };

        let do_response = self.client
            .post(format!("{}/api/v1/stacks/{}/deployment-objects", self.settings.agent.broker_url, self.stack.id))
            .header("Content-Type", "application/json")
            .header("Authorization", &self.admin_pak)
            .body(Body::from(serde_json::to_string(&new_deployment_object).unwrap()))
            .send()
            .await
            .expect("Failed to send deployment object creation request");

        assert_eq!(do_response.status(), StatusCode::OK);
        do_response.json().await.expect("Failed to parse deployment object JSON")
    }

}

async fn create_generator(client: &Client, broker_url: &str, admin_pak: &str, generator_name: &str) -> (Generator, String) {
    let new_generator = NewGenerator {
        name: generator_name.to_string(),
        description: Some("Test Generator".to_string()),
    };

    let generator_response = client
        .post(format!("{}/api/v1/generators", broker_url))
        .header("Content-Type", "application/json")
        .header("Authorization", admin_pak)
        .body(Body::from(serde_json::to_string(&new_generator).unwrap()))
        .send()
        .await
        .expect("Failed to send generator creation request");

    assert_eq!(generator_response.status(), StatusCode::OK);
    let json: serde_json::Value = generator_response.json().await.expect("Failed to parse JSON");
    
    let generator = serde_json::from_value(json["generator"].clone()).expect("Failed to parse generator");
    let generator_pak = json["pak"].as_str().expect("Failed to get initial PAK").to_string();
    (generator, generator_pak)
}

async fn create_stack(client: &Client, broker_url: &str, pak: &str, stack_name: &str, generator_id: Uuid) -> Stack {
    let new_stack = NewStack::new(stack_name.to_string(), None, generator_id)
        .expect("Failed to create NewStack");

    let stack_response = client
        .post(format!("{}/api/v1/stacks", broker_url))
        .header("Content-Type", "application/json")
        .header("Authorization", pak)
        .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
        .send()
        .await
        .expect("Failed to send stack creation request");

    assert_eq!(stack_response.status(), StatusCode::OK);
    stack_response.json().await.expect("Failed to parse stack JSON")
}

async fn broker_ready(base_url: &str) -> (){
    let client = Client::new();
    let max_retries = 30;
    let retry_interval = std::time::Duration::from_secs(1);
    let mut retries = 0;
    loop {
        let response = client
            .get(format!("{}/healthz", base_url))
            .send()
            .await;
        match response {
            Ok(resp) if resp.status() == StatusCode::OK => {
                tokio::time::sleep(retry_interval).await;
                break
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
}


async fn create_agent( base_url: &str, admin_pak: &str) -> (Agent, String){
    let client = Client::new();
    let new_agent = NewAgent::new("integration-agent-test-agent".to_string(), "integration-agent-test-cluster".to_string())
        .expect("Failed to create NewAgent");

        let url = format!("{}/api/v1/agents",base_url);
        let body = serde_json::to_string(&new_agent).unwrap();

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization",  admin_pak)
            .body(Body::from(body.clone()))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");

        let test_agent = serde_json::from_value(json["agent"].clone()).expect("Failed to parse agent");
        let test_agent_pak = json["initial_pak"].as_str().expect("Failed to get initial PAK").to_string();

        (test_agent,test_agent_pak)
}



pub async fn validate_pak(pak: &str, base_url: &str) -> bool {
    let client = Client::new();
    let response = client
        .post(format!("{}/api/v1/auth/pak", base_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", pak))
        .send()
        .await
        .expect("Failed to send PAK validation request");

    response.status() == StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_deployment_object() {
        let fixture = TestFixture::new().await;
        let yaml_content = "test: deployment_object";
        let deployment_object = fixture.create_deployment_object(yaml_content).await;

        assert_eq!(deployment_object.stack_id, fixture.stack.id);
        assert_eq!(deployment_object.yaml_content, yaml_content);
        assert_eq!(deployment_object.is_deletion_marker, false);
    }
}

