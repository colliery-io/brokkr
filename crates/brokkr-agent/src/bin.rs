
// mod broker_client;
// mod k8s_client;
// mod poller;

// use config::Config;
// use api_client::ApiClient;
// use k8s_client::K8sClient;
// use poller::Poller;

#[tokio::main]
async fn main() {
//     let config = Config::from_env();

//     let api_client = ApiClient::new(config.broker_api_url);
//     let k8s_client = K8sClient::new(config.kubeconfig_path).await;

//     let poller = Poller::new(api_client, k8s_client, config.poll_interval);
//     poller.start().await;
}