use brokkr_agent::broker;
use brokkr_utils::Settings;
use reqwest::Client;
use std::sync::Once;
use brokkr_models::models::agents::NewAgent;
use serde_json::Value;

static INIT: Once = Once::new();
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;

static FIXTURE: OnceCell<Arc<Mutex<TestFixture>>> = OnceCell::new();

pub async fn get_or_init_fixture() -> Arc<Mutex<TestFixture>> {
    FIXTURE.get_or_init(|| {
        Arc::new(Mutex::new(TestFixture::new()))
    }).clone()
}

#[allow(dead_code)]
pub struct TestFixture {
    pub admin_settings: Settings,
    pub client: Client,
    pub agent_settings: Settings,
    pub initialized: bool,
}

impl TestFixture {
    pub fn new() -> Self {
        
        INIT.call_once(|| {
            });

        let admin_settings = Settings::new(None).expect("Failed to load settings");
        let client = Client::new();
        let agent_settings = admin_settings.clone();

        let test_fixture = TestFixture { admin_settings, client, agent_settings, initialized: false };
        test_fixture
    }

    pub async fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        let new_agent = NewAgent::new("test_agent".to_string(), "test_cluster".to_string())
            .expect("Failed to create NewAgent");
        
        // Create the agent
        let response = self.client.post(&format!("{}/api/v1/agents", self.admin_settings.agent.broker_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.admin_settings.agent.pak))
            .json(&new_agent)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let response_body: Value = response.json().await.expect("Failed to parse response body");
        let agent_pak = response_body["initial_pak"].as_str().expect("Failed to get initial_pak");

        self.agent_settings.agent.pak = agent_pak.to_string();
        self.initialized = true;
    }

    pub async fn wait_for_broker(&self) {
        broker::wait_for_broker_ready(&self.agent_settings).await;
    }
}
