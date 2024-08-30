use brokkr_models::models::{
    agents::NewAgent,
    deployment_objects::NewDeploymentObject,
    agent_events::NewAgentEvent,
    stacks::NewStack
};
use crate::fixtures::TestFixture;

#[test]
fn test_create_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for create agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for create agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for create event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");

    let created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    assert_eq!(created_event.agent_id, new_event.agent_id);
    assert_eq!(created_event.deployment_object_id, new_event.deployment_object_id);
    assert_eq!(created_event.event_type, new_event.event_type);
    assert_eq!(created_event.status, new_event.status);
    assert_eq!(created_event.message, new_event.message);
}

#[test]
fn test_get_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for get agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for get agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for get event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create an agent event
    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    let retrieved_event = fixture.dal.agent_events().get(created_event.id).expect("Failed to get agent event").unwrap();
    assert_eq!(retrieved_event.id, created_event.id);
    assert_eq!(retrieved_event.event_type, created_event.event_type);
}

#[test]
fn test_get_deleted_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for deleted agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for deleted agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for deleted event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create an agent event
    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    fixture.dal.agent_events().soft_delete(created_event.id).expect("Failed to soft delete agent event");

    let retrieved_event = fixture.dal.agent_events().get(created_event.id).expect("Failed to get agent event");
    assert!(retrieved_event.is_none());

    let retrieved_deleted_event = fixture.dal.agent_events().get_including_deleted(created_event.id).expect("Failed to get deleted agent event").unwrap();
    assert_eq!(retrieved_deleted_event.id, created_event.id);
    assert!(retrieved_deleted_event.deleted_at.is_some());
}

#[test]
fn test_update_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for update agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for update agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for update event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create an agent event
    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let mut created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    created_event.event_type = "UPDATED_EVENT".to_string();
    created_event.status = "FAILED".to_string();

    let updated_event = fixture.dal.agent_events().update(created_event.id, &created_event).expect("Failed to update agent event");

    assert_eq!(updated_event.event_type, "UPDATED_EVENT");
    assert_eq!(updated_event.status, "FAILED");
}

#[test]
fn test_soft_delete_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for soft delete agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for soft delete agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for soft delete event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create an agent event
    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    let affected_rows = fixture.dal.agent_events().soft_delete(created_event.id).expect("Failed to soft delete agent event");
    assert_eq!(affected_rows, 1);

    let deleted_event = fixture.dal.agent_events().get_including_deleted(created_event.id).expect("Failed to get deleted agent event").unwrap();
    assert!(deleted_event.deleted_at.is_some());
}

#[test]
fn test_hard_delete_agent_event() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for hard delete agent event".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for hard delete agent event".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for hard delete event".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create an agent event
    let new_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT".to_string(),
        "SUCCESS".to_string(),
        Some("Test message".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let created_event = fixture.dal.agent_events().create(&new_event).expect("Failed to create agent event");

    let affected_rows = fixture.dal.agent_events().hard_delete(created_event.id).expect("Failed to hard delete agent event");
    assert_eq!(affected_rows, 1);

    let hard_deleted_event = fixture.dal.agent_events().get_including_deleted(created_event.id).expect("Failed to attempt retrieval of hard-deleted agent event");
    assert!(hard_deleted_event.is_none());
}

#[test]
fn test_list_agent_events() {
    let fixture = TestFixture::new();

    // Create a stack
    let stack = fixture.dal.stacks().create(&NewStack::new(
        "Stack for listing events".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack");

    // Create an agent
    let agent = fixture.dal.agents().create(&NewAgent::new(
        "Agent for listing events".to_string(),
        "TestCluster".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent");

    // Create a deployment object
    let deployment_object = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack.id,
        "test: deployment for listing".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object");

    // Create two agent events
    let new_event1 = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT1".to_string(),
        "SUCCESS".to_string(),
        Some("Test message 1".to_string()),
    ).expect("Failed to create NewAgentEvent");
    fixture.dal.agent_events().create(&new_event1).expect("Failed to create agent event 1");

    let new_event2 = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "TEST_EVENT2".to_string(),
        "SUCCESS".to_string(),
        Some("Test message 2".to_string()),
    ).expect("Failed to create NewAgentEvent");
    let event2 = fixture.dal.agent_events().create(&new_event2).expect("Failed to create agent event 2");

    // Soft delete the second event
    fixture.dal.agent_events().soft_delete(event2.id).expect("Failed to soft delete agent event");

    let active_events = fixture.dal.agent_events().list().expect("Failed to list agent events");
    assert_eq!(active_events.len(), 1);

    let all_events = fixture.dal.agent_events().list_all().expect("Failed to list all agent events");
    assert_eq!(all_events.len(), 2);
}

#[test]
fn test_get_events_filtered() {
    let fixture = TestFixture::new();

    // Create stacks with unique names
    let stack1 = fixture.dal.stacks().create(&NewStack::new(
        "Stack1 for filtered events".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack1");

    let stack2 = fixture.dal.stacks().create(&NewStack::new(
        "Stack2 for filtered events".to_string(),
        None
    ).expect("Failed to create NewStack")).expect("Failed to create stack2");

    // Create agents
    let agent1 = fixture.dal.agents().create(&NewAgent::new(
        "Agent1 for filtered events".to_string(),
        "Cluster1".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent1");

    let agent2 = fixture.dal.agents().create(&NewAgent::new(
        "Agent2 for filtered events".to_string(),
        "Cluster2".to_string()
    ).expect("Failed to create NewAgent")).expect("Failed to create agent2");

    // Create deployment objects
    let deployment_object1 = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack1.id,
        "test: deployment1".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object1");

    let deployment_object2 = fixture.dal.deployment_objects().create(&NewDeploymentObject::new(
        stack2.id,
        "test: deployment2".to_string(),
        false
    ).expect("Failed to create NewDeploymentObject")).expect("Failed to create deployment object2");

    // Create agent events
    let new_event1 = NewAgentEvent::new(
        agent1.id,
        deployment_object1.id,
        "TEST_EVENT1".to_string(),
        "SUCCESS".to_string(),
        Some("Test message 1".to_string()),
    ).expect("Failed to create NewAgentEvent");
    fixture.dal.agent_events().create(&new_event1).expect("Failed to create agent event 1");

    let new_event2 = NewAgentEvent::new(
        agent2.id,
        deployment_object2.id,
        "TEST_EVENT2".to_string(),
        "SUCCESS".to_string(),
        Some("Test message 2".to_string()),
    ).expect("Failed to create NewAgentEvent");
    fixture.dal.agent_events().create(&new_event2).expect("Failed to create agent event 2");

    let events_for_agent1 = fixture.dal.agent_events().get_events(None, Some(agent1.id)).expect("Failed to get events for agent1");
    assert_eq!(events_for_agent1.len(), 1);
    assert_eq!(events_for_agent1[0].agent_id, agent1.id);

    let events_for_stack1 = fixture.dal.agent_events().get_events(Some(stack1.id), None).expect("Failed to get events for stack1");
    assert_eq!(events_for_stack1.len(), 1);
    assert_eq!(events_for_stack1[0].deployment_object_id, deployment_object1.id);
}




