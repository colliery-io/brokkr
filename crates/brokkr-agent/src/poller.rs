use tokio::time::{sleep, Duration};
use crate::{api_client::ApiClient, k8s_client::K8sClient};

pub struct Poller {
    api_client: ApiClient,
    k8s_client: K8sClient,
    poll_interval: u64,
}

impl Poller {
    pub fn new(api_client: ApiClient, k8s_client: K8sClient, poll_interval: u64) -> Self {
        Self {
            api_client,
            k8s_client,
            poll_interval,
        }
    }

    pub async fn start(&self) {
        loop {
            match self.api_client.get_deployment_objects().await {
                Ok(deployment_objects) => {
                    for deployment_object in deployment_objects {
                        if let Err(e) = self.k8s_client.apply_deployment_object(&deployment_object).await {
                            eprintln!("Failed to apply deployment object: {:?}", e);
                        }
                    }
                }
                Err(e) => eprintln!("Failed to fetch deployment objects: {:?}", e),
            }
            sleep(Duration::from_secs(self.poll_interval)).await;
        }
    }
}