use brokkr_models::models::agents::NewAgent;
use crate::fixtures::TestFixture;

/// Tests the creation of an agent.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates a new agent using the NewAgent struct.
/// 3. Calls the create method of AgentsDAL.
/// 4. Verifies that the created agent matches the input data.
#[test]
fn test_create_agent() {
    let fixture = TestFixture::new();
    let new_agent = NewAgent::new(
        "Test Agent".to_string(),
        "Test Cluster".to_string(),
        Some(vec!["label1".to_string(), "label2".to_string()]),
        Some(vec![("key1".to_string(), "value1".to_string())]),
    ).expect("Failed to create NewAgent");

    let created_agent = fixture.dal.agents().create(&new_agent)
        .expect("Failed to create agent");

    assert_eq!(created_agent.name, "Test Agent");
    assert_eq!(created_agent.cluster_name, "Test Cluster");
    assert!(created_agent.labels.is_some());
    assert!(created_agent.annotations.is_some());
}

/// Tests retrieving a single agent by its UUID.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Retrieves the agent using its UUID.
/// 3. Verifies that the retrieved agent matches the inserted agent.
#[test]
fn test_get_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.insert_test_agent();

    let retrieved_agent = fixture.dal.agents().get(created_agent.uuid).unwrap();
    assert_eq!(retrieved_agent.uuid, created_agent.uuid);
    assert_eq!(retrieved_agent.name, created_agent.name);
    assert_eq!(retrieved_agent.cluster_name, created_agent.cluster_name);
}

/// Tests listing all agents.
///
/// This test:
/// 1. Sets up a test fixture and inserts two test agents.
/// 2. Calls the list method of AgentsDAL.
/// 3. Verifies that the correct number of agents are returned.
#[test]
fn test_list_agents() {
    let fixture = TestFixture::new();
    fixture.insert_test_agent();
    fixture.insert_test_agent();

    let agents = fixture.dal.agents().list(false)
        .expect("Failed to list agents");

    assert_eq!(agents.len(), 2);
}

/// Tests updating an agent's information.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Modifies the agent's name and cluster_name.
/// 3. Calls the update method of AgentsDAL.
/// 4. Verifies that the updated agent reflects the changes.
#[test]
fn test_update_agent() {
    let fixture = TestFixture::new();
    let mut agent = fixture.insert_test_agent();

    agent.name = "Updated Agent".to_string();
    agent.cluster_name = "Updated Cluster".to_string();

    let updated_agent = fixture.dal.agents().update(agent.uuid, &agent)
        .expect("Failed to update agent");

    assert_eq!(updated_agent.name, "Updated Agent");
    assert_eq!(updated_agent.cluster_name, "Updated Cluster");
}

/// Tests the soft deletion of an agent.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Soft deletes the agent.
/// 3. Verifies that the agent is still retrievable after soft deletion.
#[test]
fn test_soft_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.insert_test_agent();

    fixture.dal.agents().soft_delete(created_agent.uuid)
        .expect("Failed to soft delete agent");

    let retrieved_agent = fixture.dal.agents().get(created_agent.uuid);
    assert!(retrieved_agent.is_ok());
}

/// Tests updating an agent's heartbeat.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Updates the agent's heartbeat.
/// 3. Verifies that the last_heartbeat field is updated and more recent than the original.
#[test]
fn test_update_heartbeat() {
    let fixture = TestFixture::new();
    let created_agent = fixture.insert_test_agent();

    let updated_agent = fixture.dal.agents().update_heartbeat(created_agent.uuid)
        .expect("Failed to update heartbeat");

    assert!(updated_agent.last_heartbeat.is_some());
    assert!(updated_agent.last_heartbeat.unwrap() > created_agent.last_heartbeat.unwrap_or_default());
}

/// Tests updating an agent's status.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Updates the agent's status to "active".
/// 3. Verifies that the status field is updated correctly.
#[test]
fn test_update_status() {
    let fixture = TestFixture::new();
    let created_agent = fixture.insert_test_agent();

    let updated_agent = fixture.dal.agents().update_status(created_agent.uuid, "active")
        .expect("Failed to update status");

    assert_eq!(updated_agent.status, "active");
}