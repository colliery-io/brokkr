use reqwest::Client;
use brokkr_models::models::deployment_objects::DeploymentObject;
use serde_json::Value;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_deployment_objects(&self) -> Result<Vec<DeploymentObject>, reqwest::Error> {
        let url = format!("{}/api/v1/deployment-objects", self.base_url);
        let response = self.client.get(&url).send().await?;
        let deployment_objects: Vec<DeploymentObject> = response.json().await?;
        Ok(deployment_objects)
    }
}