/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use brokkr_agent::broker;
use brokkr_models::models::agents::NewAgent;
use brokkr_utils::Settings;
use reqwest::Client;
use serde_json::Value;
use std::sync::Once;

static INIT: Once = Once::new();
use brokkr_models::models::agent_targets::NewAgentTarget;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::models::generator::{Generator, NewGenerator};
use brokkr_models::models::stacks::NewStack;
use brokkr_models::models::stacks::Stack;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
static FIXTURE: OnceCell<Arc<Mutex<TestFixture>>> = OnceCell::new();

/// Gets or initializes a test fixture singleton
///
/// This function ensures only one test fixture exists across all tests,
/// initializing it if necessary.
///
/// # Returns
/// An Arc<Mutex<TestFixture>> pointing to the shared test fixture instance
pub async fn get_or_init_fixture() -> Arc<Mutex<TestFixture>> {
    FIXTURE
        .get_or_init(|| Arc::new(Mutex::new(TestFixture::new())))
        .clone()
}

#[allow(dead_code)]
pub struct TestFixture {
    pub admin_settings: Settings,
    pub client: Client,
    pub agent_settings: Settings,
    pub initialized: bool,
    pub generator: Option<Generator>,
    pub generator_pak: Option<String>,
    pub stack: Option<Stack>,
    pub agent: Option<Agent>,
}

impl TestFixture {
    /// Creates a new TestFixture instance with default values
    pub fn new() -> Self {
        INIT.call_once(|| {});

        let admin_settings = Settings::new(None).expect("Failed to load settings");
        let client = Client::new();
        let agent_settings = admin_settings.clone();

        let test_fixture = TestFixture {
            admin_settings,
            client,
            agent_settings,
            initialized: false,
            agent: None,
            generator: None,
            generator_pak: None,
            stack: None,
        };
        test_fixture
    }

    /// Initializes the test fixture by setting up necessary resources
    ///
    /// # Panics
    /// Panics if initialization fails
    pub async fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        let new_agent = NewAgent::new("test_agent".to_string(), "test_cluster".to_string())
            .expect("Failed to create NewAgent");

        // Create the agent
        let response = self
            .client
            .post(&format!(
                "{}/api/v1/agents",
                self.admin_settings.agent.broker_url
            ))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.admin_settings.agent.pak),
            )
            .json(&new_agent)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let response_body: Value = response
            .json()
            .await
            .expect("Failed to parse response body");
        let agent_value = response_body
            .get("agent")
            .expect("Failed to get agent from response");
        let agent: Agent =
            serde_json::from_value(agent_value.clone()).expect("Failed to parse agent");

        let agent_pak = response_body["initial_pak"]
            .as_str()
            .expect("Failed to get initial_pak");

        self.agent = Some(agent);
        self.agent_settings.agent.pak = agent_pak.to_string();
        self.agent_settings.agent.agent_name = "test_agent".to_string();
        self.agent_settings.agent.cluster_name = "test_cluster".to_string();
        self.initialized = true;
        self.generator = None;
        self.generator_pak = None;

        self.create_generator("agent-integration-test-generator".to_string(), None)
            .await;
        self.create_stack("agent-integration-test-stack").await;
    }

    /// Waits for the broker to become available
    ///
    /// # Panics
    /// Panics if the broker doesn't become available within the timeout period
    pub async fn wait_for_broker(&self) {
        broker::wait_for_broker_ready(&self.agent_settings).await;
    }

    /// Creates a new generator resource
    ///
    /// # Arguments
    /// * `name` - Name of the generator
    /// * `description` - Optional description for the generator
    ///
    /// # Panics
    /// Panics if generator creation fails
    pub async fn create_generator(&mut self, name: String, description: Option<String>) {
        let new_generator = NewGenerator { name, description };

        let response = self
            .client
            .post(&format!(
                "{}/api/v1/generators",
                self.admin_settings.agent.broker_url
            ))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.admin_settings.agent.pak),
            )
            .json(&new_generator)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let result: Value = serde_json::from_slice(
            &response
                .bytes()
                .await
                .expect("Failed to read response body"),
        )
        .expect("Failed to parse response body");
        let generator_value = result
            .get("generator")
            .expect("Failed to get generator from response");
        let generator: Generator =
            serde_json::from_value(generator_value.clone()).expect("Failed to parse generator");
        let pak = result
            .get("pak")
            .expect("Failed to get PAK from response")
            .as_str()
            .expect("PAK is not a string")
            .to_string();

        self.generator = Some(generator);
        self.generator_pak = Some(pak);
    }

    /// Creates a new stack resource
    ///
    /// # Arguments
    /// * `name` - Name of the stack
    ///
    /// # Panics
    /// Panics if stack creation fails
    pub async fn create_stack(&mut self, stack_name: &str) {
        let new_stack = NewStack::new(
            stack_name.to_string(),
            None,
            self.generator.as_ref().unwrap().id,
        )
        .expect("Failed to create NewStack");

        let response = self
            .client
            .post(&format!(
                "{}/api/v1/stacks",
                self.admin_settings.agent.broker_url
            ))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.admin_settings.agent.pak),
            )
            .json(&new_stack)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let result: Stack = serde_json::from_slice(
            &response
                .bytes()
                .await
                .expect("Failed to read response body"),
        )
        .expect("Failed to parse response body");
        self.stack = Some(result);

        let new_target = NewAgentTarget::new(
            self.agent.as_ref().unwrap().id,
            self.stack.as_ref().unwrap().id,
        )
        .unwrap();

        let response = self
            .client
            .post(&format!(
                "{}/api/v1/agents/{}/targets",
                self.admin_settings.agent.broker_url,
                self.agent.as_ref().unwrap().id
            ))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.admin_settings.agent.pak),
            )
            .json(&new_target)
            .send()
            .await
            .expect("Failed to send request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }

    /// Creates a new deployment from YAML content
    ///
    /// # Arguments
    /// * `yaml_content` - YAML string containing deployment configuration
    ///
    /// # Returns
    /// The created DeploymentObject
    ///
    /// # Panics
    /// Panics if deployment creation fails
    pub async fn create_deployment(&self, yaml_content: String) -> DeploymentObject {
        let new_deployment_object = NewDeploymentObject::new(
            self.stack.as_ref().expect("Stack not created").id,
            yaml_content,
            false,
        )
        .unwrap();

        let response = self
            .client
            .post(&format!(
                "{}/api/v1/stacks/{}/deployment-objects",
                self.admin_settings.agent.broker_url,
                self.stack.as_ref().unwrap().id
            ))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.generator_pak.as_ref().expect("Generator PAK not set")
                ),
            )
            .json(&new_deployment_object)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let result: DeploymentObject = serde_json::from_slice(
            &response
                .bytes()
                .await
                .expect("Failed to read response body"),
        )
        .expect("Failed to parse response body");
        result
    }
}
