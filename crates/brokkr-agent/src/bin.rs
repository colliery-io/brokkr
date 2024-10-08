
// mod broker_client;
// mod k8s_client;
// mod poller;

use brokkr_utils::config::{self, Config};
// use api_client::ApiClient;
// use k8s_client::K8sClient;
// use poller::Poller;

#[tokio::main]
async fn main() {
    let config = Settings::new(None).expect("Failed to load configuration");


    loop {
        // objects_to_deploy = client.get_deployment_objects().await;
        // for object in objects_to_deploy {
        //     println!("Deploying object: {}", object);
        //     get object hash + uuid
        //     docs = object.split_into_docs
        //     for doc in docs {
        //     
        // }    
    }
}
