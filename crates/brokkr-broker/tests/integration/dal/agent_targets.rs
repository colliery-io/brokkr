use crate::fixtures::TestFixture;
use brokkr_models::models::agent_targets::NewAgentTarget;

#[test]
fn test_create_agent_target() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_target =
        NewAgentTarget::new(agent.id, stack.id).expect("Failed to create NewAgentTarget");
    let created_target = fixture
        .dal
        .agent_targets()
        .create(&new_target)
        .expect("Failed to create agent target");

    assert_eq!(created_target.agent_id, agent.id);
    assert_eq!(created_target.stack_id, stack.id);
}

#[test]
fn test_get_agent_target() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let target = fixture.create_test_agent_target(agent.id, stack.id);

    let retrieved_target = fixture
        .dal
        .agent_targets()
        .get(target.id)
        .expect("Failed to get agent target")
        .unwrap();
    assert_eq!(retrieved_target.id, target.id);
    assert_eq!(retrieved_target.agent_id, agent.id);
    assert_eq!(retrieved_target.stack_id, stack.id);
}

#[test]
fn test_list_agent_targets() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    fixture.create_test_agent_target(agent1.id, stack1.id);
    fixture.create_test_agent_target(agent1.id, stack2.id);
    fixture.create_test_agent_target(agent2.id, stack1.id);

    let all_targets = fixture
        .dal
        .agent_targets()
        .list()
        .expect("Failed to list agent targets");
    assert_eq!(all_targets.len(), 3);
}

#[test]
fn test_list_agent_targets_for_agent() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    fixture.create_test_agent_target(agent.id, stack1.id);
    fixture.create_test_agent_target(agent.id, stack2.id);

    let agent_targets = fixture
        .dal
        .agent_targets()
        .list_for_agent(agent.id)
        .expect("Failed to list agent targets for agent");
    assert_eq!(agent_targets.len(), 2);
    assert!(agent_targets.iter().all(|t| t.agent_id == agent.id));
}

#[test]
fn test_list_agent_targets_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    fixture.create_test_agent_target(agent1.id, stack.id);
    fixture.create_test_agent_target(agent2.id, stack.id);

    let stack_targets = fixture
        .dal
        .agent_targets()
        .list_for_stack(stack.id)
        .expect("Failed to list agent targets for stack");
    assert_eq!(stack_targets.len(), 2);
    assert!(stack_targets.iter().all(|t| t.stack_id == stack.id));
}

#[test]
fn test_delete_agent_target() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let target = fixture.create_test_agent_target(agent.id, stack.id);

    let affected_rows = fixture
        .dal
        .agent_targets()
        .delete(target.id)
        .expect("Failed to delete agent target");
    assert_eq!(affected_rows, 1);

    let deleted_target = fixture
        .dal
        .agent_targets()
        .get(target.id)
        .expect("Failed to attempt retrieval of deleted agent target");
    assert!(deleted_target.is_none());
}

#[test]
fn test_delete_agent_targets_for_agent() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    fixture.create_test_agent_target(agent.id, stack1.id);
    fixture.create_test_agent_target(agent.id, stack2.id);

    let affected_rows = fixture
        .dal
        .agent_targets()
        .delete_for_agent(agent.id)
        .expect("Failed to delete agent targets for agent");
    assert_eq!(affected_rows, 2);

    let remaining_targets = fixture
        .dal
        .agent_targets()
        .list_for_agent(agent.id)
        .expect("Failed to list agent targets for agent");
    assert!(remaining_targets.is_empty());
}

#[test]
fn test_delete_agent_targets_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    fixture.create_test_agent_target(agent1.id, stack.id);
    fixture.create_test_agent_target(agent2.id, stack.id);

    let affected_rows = fixture
        .dal
        .agent_targets()
        .delete_for_stack(stack.id)
        .expect("Failed to delete agent targets for stack");
    assert_eq!(affected_rows, 2);

    let remaining_targets = fixture
        .dal
        .agent_targets()
        .list_for_stack(stack.id)
        .expect("Failed to list agent targets for stack");
    assert!(remaining_targets.is_empty());
}
