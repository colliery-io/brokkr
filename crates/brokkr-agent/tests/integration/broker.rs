use crate::fixtures::get_or_init_fixture;
use brokkr_agent::broker;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::DeploymentObject;
use reqwest::StatusCode;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

const TEST_NAMESPACE_YAML: &str = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: rust-app-namespace
  labels:
    name: rust-app-namespace
    environment: production
"#;

#[tokio::test]
async fn test_wait_for_broker() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;

    // Use a timeout to ensure the test doesn't hang indefinitely
    let result = timeout(Duration::from_secs(30), fixture_guard.wait_for_broker()).await;

    assert!(result.is_ok(), "wait_for_broker timed out");
}

#[tokio::test]
async fn test_verify_agent_pak() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;

    // Assuming we have a valid PAK in our test settings
    let result = broker::verify_agent_pak(&fixture_guard.agent_settings).await;
    assert!(result.is_ok(), "Agent PAK verification should succeed");

    // Assuming we have a valid PAK in our test settings
    let result = broker::verify_agent_pak(&fixture_guard.admin_settings).await;
    assert!(result.is_ok(), "Admin PAK verification should succeed");
}

#[tokio::test]
async fn test_fetch_agent_details() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;

    let result =
        broker::fetch_agent_details(&fixture_guard.agent_settings, &fixture_guard.client).await;
    assert!(
        result.is_ok(),
        "Agent details should be fetched successfully"
    );
    let agent = result.unwrap();
    assert_eq!(agent.name, fixture_guard.agent_settings.agent.agent_name);
    assert_eq!(
        agent.cluster_name,
        fixture_guard.agent_settings.agent.cluster_name
    );
}

#[tokio::test]
async fn test_fetch_and_process_deployment_objects() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;

    let deployment_object = fixture_guard
        .create_deployment(
            "agent-integration-test-stack",
            TEST_NAMESPACE_YAML.to_string(),
        )
        .await;
    let result = broker::fetch_and_process_deployment_objects(
        &fixture_guard.agent_settings,
        &fixture_guard.client,
        &fixture_guard.agent.as_ref().unwrap(),
    )
    .await;
    assert!(
        result.is_ok(),
        "Failed to fetch and process deployment objects"
    );

    let deployment_objects = result.unwrap();
    assert!(
        !deployment_objects.is_empty(),
        "No deployment objects fetched"
    );
    assert_eq!(deployment_objects[0].id, deployment_object.id);
}
