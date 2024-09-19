use crate::fixtures::TestFixture;
use brokkr_models::models::agent_labels::NewAgentLabel;

#[test]
fn test_create_agent_label() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_label = NewAgentLabel::new(agent.id, "test-label".to_string())
        .expect("Failed to create NewAgentLabel");
    let created_label = fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    assert_eq!(created_label.agent_id, agent.id);
    assert_eq!(created_label.label, "test-label");
}

#[test]
fn test_get_agent_label() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let new_label = NewAgentLabel::new(agent.id, "test-label".to_string())
        .expect("Failed to create NewAgentLabel");
    let created_label = fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    let retrieved_label = fixture
        .dal
        .agent_labels()
        .get(created_label.id)
        .expect("Failed to get agent label")
        .unwrap();
    assert_eq!(retrieved_label.id, created_label.id);
    assert_eq!(retrieved_label.label, "test-label");
}

#[test]
fn test_list_labels_for_agent() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let label1 =
        NewAgentLabel::new(agent.id, "label1".to_string()).expect("Failed to create NewAgentLabel");
    let label2 =
        NewAgentLabel::new(agent.id, "label2".to_string()).expect("Failed to create NewAgentLabel");
    fixture
        .dal
        .agent_labels()
        .create(&label1)
        .expect("Failed to create agent label");
    fixture
        .dal
        .agent_labels()
        .create(&label2)
        .expect("Failed to create agent label");

    let labels = fixture
        .dal
        .agent_labels()
        .list_for_agent(agent.id)
        .expect("Failed to list agent labels");
    assert_eq!(labels.len(), 2);
    assert!(labels.iter().any(|l| l.label == "label1"));
    assert!(labels.iter().any(|l| l.label == "label2"));
}

#[test]
fn test_delete_agent_label() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let new_label = NewAgentLabel::new(agent.id, "test-label".to_string())
        .expect("Failed to create NewAgentLabel");
    let created_label = fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    let deleted_count = fixture
        .dal
        .agent_labels()
        .delete(created_label.id)
        .expect("Failed to delete agent label");
    assert_eq!(deleted_count, 1);

    let retrieved_label = fixture
        .dal
        .agent_labels()
        .get(created_label.id)
        .expect("Failed to attempt retrieval of deleted label");
    assert!(retrieved_label.is_none());
}

#[test]
fn test_delete_all_labels_for_agent() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let label1 =
        NewAgentLabel::new(agent.id, "label1".to_string()).expect("Failed to create NewAgentLabel");
    let label2 =
        NewAgentLabel::new(agent.id, "label2".to_string()).expect("Failed to create NewAgentLabel");
    fixture
        .dal
        .agent_labels()
        .create(&label1)
        .expect("Failed to create agent label");
    fixture
        .dal
        .agent_labels()
        .create(&label2)
        .expect("Failed to create agent label");

    let deleted_count = fixture
        .dal
        .agent_labels()
        .delete_all_for_agent(agent.id)
        .expect("Failed to delete all agent labels");
    assert_eq!(deleted_count, 2);

    let remaining_labels = fixture
        .dal
        .agent_labels()
        .list_for_agent(agent.id)
        .expect("Failed to list agent labels");
    assert!(remaining_labels.is_empty());
}

#[test]
fn test_label_exists() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_label = NewAgentLabel::new(agent.id, "existing-label".to_string())
        .expect("Failed to create NewAgentLabel");
    fixture
        .dal
        .agent_labels()
        .create(&new_label)
        .expect("Failed to create agent label");

    let exists = fixture
        .dal
        .agent_labels()
        .label_exists(agent.id, "existing-label")
        .expect("Failed to check if label exists");
    assert!(exists);

    let not_exists = fixture
        .dal
        .agent_labels()
        .label_exists(agent.id, "non-existing-label")
        .expect("Failed to check if label exists");
    assert!(!not_exists);
}
