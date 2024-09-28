use brokkr_agent::broker_client::ApiClient;
use std::env;
use tokio;

#[tokio::test]
async fn test_send_heartbeat() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.send_heartbeat().await;
    assert!(result.is_ok(), "Failed to send heartbeat: {:?}", result.err());
}

#[tokio::test]
async fn test_get_deployment_objects() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.get_deployment_objects().await;
    assert!(result.is_ok(), "Failed to get deployment objects: {:?}", result.err());
    let deployment_objects = result.unwrap();
    assert!(!deployment_objects.is_empty(), "Deployment objects list is empty");
}

#[tokio::test]
async fn test_get_ready_deployment_objects() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.get_ready_deployment_objects().await;
    assert!(result.is_ok(), "Failed to get ready deployment objects: {:?}", result.err());
    let deployment_objects = result.unwrap();
    assert!(!deployment_objects.is_empty(), "Ready deployment objects list is empty");
}

#[tokio::test]
async fn test_get_object_uuid() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";
    let cluster_name = "test_cluster";
    let agent_name = "test_agent";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.get_object_uuid(cluster_name, agent_name).await;
    assert!(result.is_ok(), "Failed to get object UUID: {:?}", result.err());
    let uuid = result.unwrap();
    assert!(!uuid.is_empty(), "UUID is empty");
}

#[tokio::test]
async fn test_check_readyz() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.check_readyz().await;
    assert!(result.is_ok(), "Failed to check readyz: {:?}", result.err());
}

#[tokio::test]
async fn test_check_healthz() {
    let base_url = "http://localhost:8080";
    let pak = "test_pak";

    let api_client = ApiClient::new(base_url.to_string(), pak.to_string());

    let result = api_client.check_healthz().await;
    assert!(result.is_ok(), "Failed to check healthz: {:?}", result.err());
}