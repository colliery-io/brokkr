use brokkr_models::models::agents:: NewAgent;
use crate::fixtures::TestFixture;

#[test]
fn test_create_agent() {
    let fixture = TestFixture::new();

    let new_agent = NewAgent::new(
        "Test Agent".to_string(),
        "Test Cluster".to_string()
    ).expect("Failed to create NewAgent");

    let created_agent = fixture.dal.agents().create(&new_agent).expect("Failed to create agent");

    assert_eq!(created_agent.name, new_agent.name);
    assert_eq!(created_agent.cluster_name, new_agent.cluster_name);
    assert!(created_agent.last_heartbeat.is_none());
    assert_eq!(created_agent.status, "INACTIVE");
}

#[test]
fn test_get_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let retrieved_agent = fixture.dal.agents().get(created_agent.id).expect("Failed to get agent").unwrap();
    assert_eq!(retrieved_agent.id, created_agent.id);
    assert_eq!(retrieved_agent.name, created_agent.name);
}

#[test]
fn test_get_deleted_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    
    fixture.dal.agents().soft_delete(created_agent.id).expect("Failed to soft delete agent");

    let retrieved_agent = fixture.dal.agents().get(created_agent.id).expect("Failed to get agent");
    assert!(retrieved_agent.is_none());

    let retrieved_deleted_agent = fixture.dal.agents().get_including_deleted(created_agent.id).expect("Failed to get deleted agent").unwrap();
    assert_eq!(retrieved_deleted_agent.id, created_agent.id);
    assert!(retrieved_deleted_agent.deleted_at.is_some());
}

#[test]
fn test_list_agents() {
    let fixture = TestFixture::new();
    fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let deleted_agent = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    fixture.dal.agents().soft_delete(deleted_agent.id).expect("Failed to soft delete agent");

    let active_agents = fixture.dal.agents().list().expect("Failed to list agents");
    assert_eq!(active_agents.len(), 1);
    assert_eq!(active_agents[0].name, "Agent 1");

    let all_agents = fixture.dal.agents().list_all().expect("Failed to list all agents");
    assert_eq!(all_agents.len(), 2);
}

#[test]
fn test_update_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent("Original Agent".to_string(), "Original Cluster".to_string());

    let mut updated_agent = created_agent.clone();
    updated_agent.name = "Updated Agent".to_string();
    updated_agent.cluster_name = "Updated Cluster".to_string();

    let result = fixture.dal.agents().update(created_agent.id, &updated_agent).expect("Failed to update agent");

    assert_eq!(result.name, "Updated Agent");
    assert_eq!(result.cluster_name, "Updated Cluster");
}

#[test]
fn test_soft_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent("To Be Deleted".to_string(), "Test Cluster".to_string());

    let affected_rows = fixture.dal.agents().soft_delete(created_agent.id).expect("Failed to soft delete agent");
    assert_eq!(affected_rows, 1);

    let deleted_agent = fixture.dal.agents().get_including_deleted(created_agent.id).expect("Failed to get deleted agent").unwrap();
    assert!(deleted_agent.deleted_at.is_some());
}

#[test]
fn test_hard_delete_agent() {
    let fixture = TestFixture::new();
    let created_agent = fixture.create_test_agent("To Be Hard Deleted".to_string(), "Test Cluster".to_string());

    let affected_rows = fixture.dal.agents().hard_delete(created_agent.id).expect("Failed to hard delete agent");
    assert_eq!(affected_rows, 1);

    let hard_deleted_agent = fixture.dal.agents().get_including_deleted(created_agent.id).expect("Failed to attempt retrieval of hard-deleted agent");
    assert!(hard_deleted_agent.is_none());
}

#[test]
fn test_search_agents() {
    let fixture = TestFixture::new();
    fixture.create_test_agent("Alpha Agent".to_string(), "Cluster 1".to_string());
    fixture.create_test_agent("Beta Agent".to_string(), "Alpha Cluster".to_string());
    let deleted_agent = fixture.create_test_agent("Gamma Agent".to_string(), "Cluster 3".to_string());
    fixture.dal.agents().soft_delete(deleted_agent.id).expect("Failed to soft delete agent");

    let active_results = fixture.dal.agents().search("Alpha").expect("Failed to search agents");
    assert_eq!(active_results.len(), 2);
    assert!(active_results.iter().any(|a| a.name == "Alpha Agent"));
    assert!(active_results.iter().any(|a| a.cluster_name == "Alpha Cluster"));

    let all_results = fixture.dal.agents().search_all("Agent").expect("Failed to search all agents");
    assert_eq!(all_results.len(), 3);
}