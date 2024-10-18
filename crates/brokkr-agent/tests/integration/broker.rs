use crate::fixtures::get_or_init_fixture;
use brokkr_agent::broker;
use tokio::time::{timeout, Duration};

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
    
    let result = broker::fetch_agent_details(&fixture_guard.agent_settings, &fixture_guard.client).await;
    
    assert!(result.is_ok(), "Fetching agent details should succeed");
    let agent = result.unwrap();
    assert_eq!(agent.name, fixture_guard.agent_settings.agent.agent_name, "Agent name should match the one in settings");
    assert_eq!(agent.cluster_name, fixture_guard.agent_settings.agent.cluster_name, "Cluster name should match the one in settings");
}

#[tokio::test]
async fn test_fetch_and_process_deployment_objects() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;
    
    // First, fetch agent details
    let agent = broker::fetch_agent_details(&fixture_guard.agent_settings, &fixture_guard.client).await.unwrap();

    let result = broker::fetch_and_process_deployment_objects(&fixture_guard.agent_settings, &fixture_guard.client, &agent).await;
    
    assert!(result.is_ok(), "Fetching deployment objects should succeed");
    let deployment_objects = result.unwrap();
    assert!(!deployment_objects.is_empty(), "There should be at least one deployment object");
}

#[tokio::test]
async fn test_send_success_event() {
    let fixture = get_or_init_fixture().await;
    let mut fixture_guard = fixture.lock().await;
    fixture_guard.initialize().await;
    
    // First, fetch agent details
    let agent = broker::fetch_agent_details(&fixture.settings, &fixture.client).await.unwrap();
    
    // Then, fetch a deployment object
    let deployment_objects = broker::fetch_and_process_deployment_objects(&fixture_guard.agent_settings, &fixture_guard.client, &agent).await.unwrap();
    let deployment_object = deployment_objects.first().unwrap();
    
    let result = broker::send_success_event(
        &fixture.settings,
        &fixture.client,
        &agent,
        deployment_object.id,
        Some("Test success event".to_string())
    ).await;
    
    assert!(result.is_ok(), "Sending success event should succeed");
}

#[tokio::test]
async fn test_send_failure_event() {
    let fixture = TestFixture::new();
    
    // First, fetch agent details
    let agent = broker::fetch_agent_details(&fixture.settings, &fixture.client).await.unwrap();
    
    // Then, fetch a deployment object
    let deployment_objects = broker::fetch_and_process_deployment_objects(&fixture_guard.agent_settings, &fixture_guard.client, &agent).await.unwrap();
    let deployment_object = deployment_objects.first().unwrap();
    
    let result = broker::send_failure_event(
        &fixture.settings,
        &fixture.client,
        &agent,
        deployment_object.id,
        "Test failure event".to_string()
    ).await;
    
    assert!(result.is_ok(), "Sending failure event should succeed");
}
