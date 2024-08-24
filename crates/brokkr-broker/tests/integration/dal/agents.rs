use brokkr_models::models::agents::NewAgent;
use crate::fixtures::TestFixture;
use serde_json::json;


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

/// Tests retrieving a single agent by its id.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent.
/// 2. Retrieves the agent using its id.
/// 3. Verifies that the retrieved agent matches the inserted agent.
#[test]
fn test_get_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.insert_test_agent();

    let retrieved_agent = fixture.dal.agents().get(created_agent.id, false).unwrap();
    assert_eq!(retrieved_agent.id, created_agent.id);
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

    let updated_agent = fixture.dal.agents().update(agent.id, &agent)
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

    fixture.dal.agents().soft_delete(created_agent.id)
        .expect("Failed to soft delete agent");

    let retrieved_agent = fixture.dal.agents().get(created_agent.id, true);
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

    let updated_agent = fixture.dal.agents().update_heartbeat(created_agent.id)
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

    let updated_agent = fixture.dal.agents().update_status(created_agent.id, "active")
        .expect("Failed to update status");

    assert_eq!(updated_agent.status, "active");
}

/// Tests creating an agent with a conflicting name/cluster pair
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates a new agent with a specific name.
/// 3. Attempts to create another agent with the same name.
/// 4. Verifies that the second creation attempt results in an error.
#[test]
fn test_create_agent_with_conflicting_name() {
    let fixture = TestFixture::new();
    
    let agent_name = "Duplicate Agent Name".to_string();
    
    let new_agent1 = NewAgent::new(
        agent_name.clone(),
        "Cluster1".to_string(),
        None,
        None,
    ).expect("Failed to create NewAgent");

    let new_agent2 = NewAgent::new(
        agent_name.clone(),
        "Cluster1".to_string(),
        None,
        None,
    ).expect("Failed to create NewAgent");

    fixture.dal.agents().create(&new_agent1).expect("Failed to create first agent");

    let result = fixture.dal.agents().create(&new_agent2);
    assert!(result.is_err());
    // You may want to check for a specific error message or type here
}

/// Tests updating an agent's labels and annotations.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates a new agent with initial labels and annotations.
/// 3. Updates the agent with new labels and annotations.
/// 4. Verifies that the agent's labels and annotations are correctly updated.
#[test]
fn test_update_agent_labels_and_annotations() {
    let fixture = TestFixture::new();
    
    let new_agent = NewAgent::new(
        "Test Agent".to_string(),
        "Test Cluster".to_string(),
        Some(vec!["initial_label".to_string()]),
        Some(vec![("initial_key".to_string(), "initial_value".to_string())]),
    ).expect("Failed to create NewAgent");

    let created_agent = fixture.dal.agents().create(&new_agent).expect("Failed to create agent");

    let mut updated_agent = created_agent.clone();
    updated_agent.labels = Some(json!(["updated_label".to_string()]));
    updated_agent.annotations = Some(json![("updated_key".to_string(), "updated_value".to_string())]);

    let result = fixture.dal.agents().update(created_agent.id, &updated_agent).expect("Failed to update agent");

    assert_eq!(result.labels, Some(json!(["updated_label".to_string()])));
    assert_eq!(result.annotations, Some(json![("updated_key".to_string(), "updated_value".to_string())]));
}