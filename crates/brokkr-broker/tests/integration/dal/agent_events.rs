use crate::fixtures::TestFixture;
use brokkr_models::models::agent_events::NewAgentEvent;
use uuid::Uuid;

/// Tests the creation of an agent event.
///
/// This test:
/// 1. Sets up a test fixture with an agent, stack, and deployment object.
/// 2. Creates a new agent event using the NewAgentEvent struct.
/// 3. Calls the create method of AgentEventsDAL.
/// 4. Verifies that the created event matches the input data.
#[test]
fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);

    let new_agent_event = NewAgentEvent::new(
        agent.id,
        deployment_object.id,
        "test_event".to_string(),
        "success".to_string(),
        Some("Test message".to_string()),
    )
    .unwrap();

    let created_event = fixture
        .dal
        .agent_events()
        .create(&new_agent_event)
        .expect("Failed to create agent event");

    assert_eq!(created_event.agent_id, agent.id);
    assert_eq!(created_event.deployment_object_id, deployment_object.id);
    assert_eq!(created_event.event_type, "test_event");
    assert_eq!(created_event.status, "success");
    assert_eq!(created_event.message, Some("Test message".to_string()));
}

/// Tests retrieving a single agent event by its id.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent event.
/// 2. Retrieves the event using its id.
/// 3. Verifies that the retrieved event matches the inserted event.
#[test]
fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.id, deployment_object.id);

    let retrieved_event = fixture
        .dal
        .agent_events()
        .get(created_event.id)
        .expect("Failed to get agent event")
        .expect("Agent event not found");

    assert_eq!(retrieved_event.id, created_event.id);
    assert_eq!(retrieved_event.agent_id, agent.id);
    assert_eq!(retrieved_event.deployment_object_id, deployment_object.id);
}

/// Tests listing all agent events.
///
/// This test:
/// 1. Sets up a test fixture and inserts two test agent events.
/// 2. Calls the list method of AgentEventsDAL.
/// 3. Verifies that the correct number of events are returned.
#[test]
fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);

    fixture.insert_test_agent_event(agent.id, deployment_object.id);
    fixture.insert_test_agent_event(agent.id, deployment_object.id);

    let events = fixture
        .dal
        .agent_events()
        .list()
        .expect("Failed to list agent events");

    assert_eq!(events.len(), 2);
}

/// Tests retrieving agent events with various filters.
///
/// This test:
/// 1. Sets up a test fixture with multiple agents, stacks, and events.
/// 2. Tests retrieving all events.
/// 3. Tests filtering events by stack.
/// 4. Tests filtering events by agent.
/// 5. Tests filtering events by both stack and agent.
/// 6. Tests filtering with a non-existent stack.
#[test]
fn test_get_events() {
    let fixture = TestFixture::new();
    let agent1 = fixture.insert_test_agent();
    let agent2 = fixture.insert_test_agent();
    let stack1 = fixture.insert_test_stack();
    let stack2 = fixture.insert_test_stack();
    let deployment_object1 = fixture.insert_test_deployment_object(stack1);
    let deployment_object2 = fixture.insert_test_deployment_object(stack2);

    fixture.insert_test_agent_event(agent1.id, deployment_object1.id);
    fixture.insert_test_agent_event(agent1.id, deployment_object2.id);
    fixture.insert_test_agent_event(agent2.id, deployment_object1.id);

    // Test getting all events
    let all_events = fixture
        .dal
        .agent_events()
        .get_events(None, None)
        .expect("Failed to get all agent events");
    assert_eq!(all_events.len(), 3);

    // Test getting events for a specific stack
    let stack1_events = fixture
        .dal
        .agent_events()
        .get_events(Some(stack1), None)
        .expect("Failed to get stack1 agent events");
    assert_eq!(stack1_events.len(), 2);

    // Test getting events for a specific agent
    let agent1_events = fixture
        .dal
        .agent_events()
        .get_events(None, Some(agent1.id))
        .expect("Failed to get agent1 events");
    assert_eq!(agent1_events.len(), 2);

    // Test getting events for a specific stack and agent
    let stack1_agent1_events = fixture
        .dal
        .agent_events()
        .get_events(Some(stack1), Some(agent1.id))
        .expect("Failed to get stack1 and agent1 events");
    assert_eq!(stack1_agent1_events.len(), 1);

    // Test getting events for a non-existent stack
    let non_existent_events = fixture
        .dal
        .agent_events()
        .get_events(Some(Uuid::new_v4()), None)
        .expect("Failed to get non-existent events");
    assert_eq!(non_existent_events.len(), 0);
}

/// Tests the soft deletion of an agent event.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent event.
/// 2. Soft deletes the event.
/// 3. Verifies that the event is not retrievable with the normal get method.
/// 4. Verifies that the event is retrievable with the method that includes deleted events.
/// 5. Checks that soft-deleted events do not appear in get_events results.
#[test]
fn test_soft_delete_agent_event() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.id, deployment_object.id);

    fixture
        .dal
        .agent_events()
        .soft_delete(created_event.id)
        .expect("Failed to soft delete agent event");

    // The event should not be retrievable with the normal get method
    let retrieved_event = fixture
        .dal
        .agent_events()
        .get(created_event.id)
        .expect("Failed to get agent event");
    assert!(retrieved_event.is_none());

    // But it should be retrievable with the method that includes deleted events
    let retrieved_deleted_event = fixture
        .dal
        .agent_events()
        .get_including_deleted(created_event.id)
        .expect("Failed to get deleted agent event")
        .expect("Deleted event not found");

    assert_eq!(retrieved_deleted_event.id, created_event.id);
    assert!(retrieved_deleted_event.deleted_at.is_some());

    // Soft-deleted events should not appear in get_events results
    let events = fixture
        .dal
        .agent_events()
        .get_events(None, None)
        .expect("Failed to get events");
    assert!(!events.iter().any(|e| e.id == created_event.id));
}

/// Tests retrieving a soft-deleted agent event.
///
/// This test:
/// 1. Sets up a test fixture and inserts a test agent event.
/// 2. Soft deletes the event.
/// 3. Retrieves the deleted event using the get_including_deleted method.
/// 4. Verifies that the retrieved event matches the original and has a deletion timestamp.
#[test]
fn test_get_including_deleted() {
    let fixture = TestFixture::new();
    let agent = fixture.insert_test_agent();
    let stack = fixture.insert_test_stack();
    let deployment_object = fixture.insert_test_deployment_object(stack);
    let created_event = fixture.insert_test_agent_event(agent.id, deployment_object.id);

    // Soft delete the event
    fixture
        .dal
        .agent_events()
        .soft_delete(created_event.id)
        .expect("Failed to soft delete agent event");

    // Retrieve the deleted event
    let retrieved_event = fixture
        .dal
        .agent_events()
        .get_including_deleted(created_event.id)
        .expect("Failed to get including deleted agent event")
        .expect("Deleted agent event not found");

    assert_eq!(retrieved_event.id, created_event.id);
    assert!(retrieved_event.deleted_at.is_some());
}
