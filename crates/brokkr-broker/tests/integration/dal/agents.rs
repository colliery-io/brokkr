use brokkr_models::models::agents::NewAgent;
use crate::fixtures::TestFixture;
use uuid::Uuid;

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

#[test]
fn test_get_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent();

    let retrieved_agent = fixture.dal.agents().get(created_agent.uuid)
        .expect("Failed to get agent")
        .expect("Agent not found");

    assert_eq!(retrieved_agent.uuid, created_agent.uuid);
    assert_eq!(retrieved_agent.name, created_agent.name);
    assert_eq!(retrieved_agent.cluster_name, created_agent.cluster_name);
}

#[test]
fn test_list_agents() {
    let fixture = TestFixture::new();
    fixture.create_test_agent();
    fixture.create_test_agent();

    let agents = fixture.dal.agents().list()
        .expect("Failed to list agents");

    assert_eq!(agents.len(), 2);
}

#[test]
fn test_update_agent() {
    let fixture = TestFixture::new();
    let mut agent = fixture.create_test_agent();

    agent.name = "Updated Agent".to_string();
    agent.cluster_name = "Updated Cluster".to_string();

    let updated_agent = fixture.dal.agents().update(agent.uuid, &agent)
        .expect("Failed to update agent");

    assert_eq!(updated_agent.name, "Updated Agent");
    assert_eq!(updated_agent.cluster_name, "Updated Cluster");
}

#[test]
fn test_soft_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent();

    fixture.dal.agents().soft_delete(created_agent.uuid)
        .expect("Failed to soft delete agent");

    let retrieved_agent = fixture.dal.agents().get(created_agent.uuid)
        .expect("Failed to get agent");
    assert!(retrieved_agent.is_none());
}

#[test]
fn test_update_heartbeat() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent();

    let updated_agent = fixture.dal.agents().update_heartbeat(created_agent.uuid)
        .expect("Failed to update heartbeat");

    assert!(updated_agent.last_heartbeat.is_some());
    assert!(updated_agent.last_heartbeat.unwrap() > created_agent.last_heartbeat.unwrap_or_default());
}

#[test]
fn test_update_status() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent();

    let updated_agent = fixture.dal.agents().update_status(created_agent.uuid, "active")
        .expect("Failed to update status");

    assert_eq!(updated_agent.status, "active");
}