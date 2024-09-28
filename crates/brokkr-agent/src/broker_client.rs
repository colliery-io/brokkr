use reqwest::Client;
use brokkr_models::models::deployment_objects::DeploymentObject;
use serde_json::Value;

pub struct ApiClient {
    client: Client,
    base_url: String,
    pak: String,
}

impl ApiClient {
    pub fn new(base_url: String, pak: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            pak,
        }
    }

    pub async fn get_deployment_objects(&self) -> Result<Vec<DeploymentObject>, reqwest::Error> {
        let url = format!("{}/api/v1/deployment-objects", self.base_url);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        let deployment_objects: Vec<DeploymentObject> = response.json().await?;
        Ok(deployment_objects)
    }

    pub async fn send_heartbeat(&self) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        Ok(())
    }

    pub async fn get_ready_deployment_objects(&self) -> Result<Vec<DeploymentObject>, reqwest::Error> {
        let url = format!("{}/api/v1/deployment-objects/ready", self.base_url);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        let deployment_objects: Vec<DeploymentObject> = response.json().await?;
        Ok(deployment_objects)
    }

    pub async fn get_object_uuid(&self, cluster_name: &str, agent_name: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/api/v1/objects/uuid?cluster={}&agent={}", self.base_url, cluster_name, agent_name);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        let uuid: String = response.json().await?;
        Ok(uuid)
    }

    pub async fn check_readyz(&self) -> Result<(), reqwest::Error> {
        let url = format!("{}/readyz", self.base_url);
        self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        Ok(())
    }

    pub async fn check_healthz(&self) -> Result<(), reqwest::Error> {
        let url = format!("{}/healthz", self.base_url);
        self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.pak))
            .send().await?;
        Ok(())
    }
}