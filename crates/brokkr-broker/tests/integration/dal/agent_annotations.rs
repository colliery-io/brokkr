use brokkr_models::models::agent_annotations::NewAgentAnnotation;
use crate::fixtures::TestFixture;

#[test]
fn test_create_agent_annotation() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    let new_annotation = NewAgentAnnotation::new(agent.id, "key1".to_string(), "value1".to_string())
        .expect("Failed to create NewAgentAnnotation");

    let created_annotation = fixture.dal.agent_annotations().create(&new_annotation)
        .expect("Failed to create agent annotation");

    assert_eq!(created_annotation.agent_id, agent.id);
    assert_eq!(created_annotation.key, "key1");
    assert_eq!(created_annotation.value, "value1");
}

#[test]
fn test_get_agent_annotation() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let annotation = fixture.create_test_agent_annotation(agent.id, "key1".to_string(), "value1".to_string());

    let retrieved_annotation = fixture.dal.agent_annotations().get(annotation.id)
        .expect("Failed to get agent annotation")
        .unwrap();

    assert_eq!(retrieved_annotation.id, annotation.id);
    assert_eq!(retrieved_annotation.agent_id, agent.id);
    assert_eq!(retrieved_annotation.key, "key1");
    assert_eq!(retrieved_annotation.value, "value1");
}

#[test]
fn test_list_agent_annotations() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    fixture.create_test_agent_annotation(agent.id, "key1".to_string(), "value1".to_string());
    fixture.create_test_agent_annotation(agent.id, "key2".to_string(), "value2".to_string());

    let annotations = fixture.dal.agent_annotations().list_for_agent(agent.id)
        .expect("Failed to list agent annotations");

    assert_eq!(annotations.len(), 2);
    assert!(annotations.iter().any(|a| a.key == "key1" && a.value == "value1"));
    assert!(annotations.iter().any(|a| a.key == "key2" && a.value == "value2"));
}

#[test]
fn test_update_agent_annotation() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let annotation = fixture.create_test_agent_annotation(agent.id, "key1".to_string(), "value1".to_string());

    let mut updated_annotation = annotation.clone();
    updated_annotation.value = "updated_value".to_string();

    let result = fixture.dal.agent_annotations().update(annotation.id, &updated_annotation)
        .expect("Failed to update agent annotation");

    assert_eq!(result.value, "updated_value");
}

#[test]
fn test_delete_agent_annotation() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let annotation = fixture.create_test_agent_annotation(agent.id, "key1".to_string(), "value1".to_string());

    let affected_rows = fixture.dal.agent_annotations().delete(annotation.id)
        .expect("Failed to delete agent annotation");

    assert_eq!(affected_rows, 1);

    let deleted_annotation = fixture.dal.agent_annotations().get(annotation.id)
        .expect("Failed to attempt retrieval of deleted annotation");
    assert!(deleted_annotation.is_none());
}

#[test]
fn test_delete_all_agent_annotations() {
    let fixture = TestFixture::new();
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    fixture.create_test_agent_annotation(agent.id, "key1".to_string(), "value1".to_string());
    fixture.create_test_agent_annotation(agent.id, "key2".to_string(), "value2".to_string());

    let affected_rows = fixture.dal.agent_annotations().delete_all_for_agent(agent.id)
        .expect("Failed to delete all agent annotations");

    assert_eq!(affected_rows, 2);

    let remaining_annotations = fixture.dal.agent_annotations().list_for_agent(agent.id)
        .expect("Failed to list agent annotations after deletion");
    assert!(remaining_annotations.is_empty());
}