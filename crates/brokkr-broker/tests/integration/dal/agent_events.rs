use uuid::Uuid;
use brokkr_models::models::agent_events::NewAgentEvent;
use crate::fixtures::TestFixture;

#[test]
fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);

    let new_agent_event = NewAgentEvent::new(
        agent.uuid,
        deployment_object.uuid,
        "test_event".to_string(),
        "success".to_string(),
        Some("Test message".to_string()),
    );

    let created_event = fixture.dal.agent_events().create(&new_agent_event)
        .expect("Failed to create agent event");

    assert_eq!(created_event.agent_id, agent.uuid);
    assert_eq!(created_event.deployment_object_id, deployment_object.uuid);
    assert_eq!(created_event.event_type, "test_event");
    assert_eq!(created_event.status, "success");
    assert_eq!(created_event.message, Some("Test message".to_string()));
}

#[test]
fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.uuid, deployment_object.uuid);

    let retrieved_event = fixture.dal.agent_events().get(created_event.uuid)
        .expect("Failed to get agent event")
        .expect("Agent event not found");

    assert_eq!(retrieved_event.uuid, created_event.uuid);
    assert_eq!(retrieved_event.agent_id, agent.uuid);
    assert_eq!(retrieved_event.deployment_object_id, deployment_object.uuid);
}

#[test]
fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);

    fixture.insert_test_agent_event(agent.uuid, deployment_object.uuid);
    fixture.insert_test_agent_event(agent.uuid, deployment_object.uuid);

    let events = fixture.dal.agent_events().list()
        .expect("Failed to list agent events");

    assert_eq!(events.len(), 2);
}

#[test]
fn test_get_events() {
    let fixture = TestFixture::new();
    let agent1 = fixture.insert_test_agent();
    let agent2 = fixture.insert_test_agent();
    let stack1 = fixture.insert_test_stack();
    let stack2 = fixture.insert_test_stack();
    let deployment_object1 = fixture.insert_test_deployment_object(stack1);
    let deployment_object2 = fixture.insert_test_deployment_object(stack2);

    fixture.insert_test_agent_event(agent1.uuid, deployment_object1.uuid);
    fixture.insert_test_agent_event(agent1.uuid, deployment_object2.uuid);
    fixture.insert_test_agent_event(agent2.uuid, deployment_object1.uuid);

    // Test getting all events
    let all_events = fixture.dal.agent_events().get_events(None, None)
        .expect("Failed to get all agent events");
    assert_eq!(all_events.len(), 3);

    // Test getting events for a specific stack
    let stack1_events = fixture.dal.agent_events().get_events(Some(stack1), None)
        .expect("Failed to get stack1 agent events");
    assert_eq!(stack1_events.len(), 2);

    // Test getting events for a specific agent
    let agent1_events = fixture.dal.agent_events().get_events(None, Some(agent1.uuid))
        .expect("Failed to get agent1 events");
    assert_eq!(agent1_events.len(), 2);

    // Test getting events for a specific stack and agent
    let stack1_agent1_events = fixture.dal.agent_events().get_events(Some(stack1), Some(agent1.uuid))
        .expect("Failed to get stack1 and agent1 events");
    assert_eq!(stack1_agent1_events.len(), 1);

    // Test getting events for a non-existent stack
    let non_existent_events = fixture.dal.agent_events().get_events(Some(Uuid::new_v4()), None)
        .expect("Failed to get non-existent events");
    assert_eq!(non_existent_events.len(), 0);
}

#[test]
fn test_soft_delete_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.uuid, deployment_object.uuid);

    fixture.dal.agent_events().soft_delete(created_event.uuid)
        .expect("Failed to soft delete agent event");

    // The event should not be retrievable with the normal get method
    let retrieved_event = fixture.dal.agent_events().get(created_event.uuid)
        .expect("Failed to get agent event");
    assert!(retrieved_event.is_none());

    // But it should be retrievable with the method that includes deleted events
    let retrieved_deleted_event = fixture.dal.agent_events().get_including_deleted(created_event.uuid)
        .expect("Failed to get deleted agent event")
        .expect("Deleted event not found");

    assert_eq!(retrieved_deleted_event.uuid, created_event.uuid);
    assert!(retrieved_deleted_event.deleted_at.is_some());

    // Soft-deleted events should not appear in get_events results
    let events = fixture.dal.agent_events().get_events(None, None)
        .expect("Failed to get events");
    assert!(!events.iter().any(|e| e.uuid == created_event.uuid));
}

#[test]
fn test_get_including_deleted() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.uuid, deployment_object.uuid);

    // Soft delete the event
    fixture.dal.agent_events().soft_delete(created_event.uuid)
        .expect("Failed to soft delete agent event");

    // Retrieve the deleted event
    let retrieved_event = fixture.dal.agent_events().get_including_deleted(created_event.uuid)
        .expect("Failed to get including deleted agent event")
        .expect("Deleted agent event not found");

    assert_eq!(retrieved_event.uuid, created_event.uuid);
    assert!(retrieved_event.deleted_at.is_some());
}