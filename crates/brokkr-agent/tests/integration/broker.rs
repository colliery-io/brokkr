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
