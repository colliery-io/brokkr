use brokkr_models::models::agent_events::NewAgentEvent;
use crate::fixtures::TestFixture;

#[test]
fn test_create_agent_event() {
    let fixture = TestFixture::new();
    let agent_id = fixture.insert_test_agent();
    let deployment_object_id = fixture.insert_test_deployment_object(fixture.insert_test_stack());

    let new_agent_event = NewAgentEvent::new(
        agent_id.uuid,
        deployment_object_id.uuid,
        "test_event".to_string(),
        "success".to_string(),
        Some("Test message".to_string()),
    );

    let created_event = fixture.dal.agent_events().create(&new_agent_event)
        .expect("Failed to create agent event");

    assert_eq!(created_event.agent_id, agent_id.uuid);
    assert_eq!(created_event.deployment_object_id, deployment_object_id.uuid);
    assert_eq!(created_event.event_type, "test_event");
    assert_eq!(created_event.status, "success");
    assert_eq!(created_event.message, Some("Test message".to_string()));
}

#[test]
fn test_get_agent_event() {
    let fixture = TestFixture::new();
    let agent_id = fixture.insert_test_agent();
    let deployment_object_id = fixture.insert_test_deployment_object(fixture.insert_test_stack());
    let created_event = fixture.insert_test_agent_event(agent_id.uuid, deployment_object_id.uuid);

    let retrieved_event = fixture.dal.agent_events().get(created_event.uuid)
        .expect("Failed to get agent event")
        .expect("Agent event not found");

    assert_eq!(retrieved_event.uuid, created_event.uuid);
    assert_eq!(retrieved_event.agent_id, agent_id.uuid);
    assert_eq!(retrieved_event.deployment_object_id, deployment_object_id.uuid);
}

#[test]
fn test_list_agent_events() {
    let fixture = TestFixture::new();
    let agent_id = fixture.insert_test_agent();
    let deployment_object_id = fixture.insert_test_deployment_object(fixture.insert_test_stack());

    fixture.insert_test_agent_event(agent_id.uuid, deployment_object_id.uuid);
    fixture.insert_test_agent_event(agent_id.uuid, deployment_object_id.uuid);

    let events = fixture.dal.agent_events().list()
        .expect("Failed to list agent events");

    assert_eq!(events.len(), 2);
}


#[test]
fn test_soft_delete_agent_event() {
    let fixture = TestFixture::new();
    let agent_id = fixture.insert_test_agent().uuid;
    let deployment_object_id = fixture.insert_test_deployment_object(fixture.insert_test_stack()).uuid;
    let created_event = fixture.insert_test_agent_event(agent_id, deployment_object_id);

    fixture.dal.agent_events().soft_delete(created_event.uuid)
        .expect("Failed to soft delete agent event");

    // The event should not be retrievable with the normal get method
    let retrieved_event = fixture.dal.agent_events().get(created_event.uuid)
        .expect("Failed to get agent event");
    assert!(retrieved_event.is_none());

    // But it should be retrievable with a method that includes deleted events
    let retrieved_deleted_event = fixture.dal.agent_events().get_including_deleted(created_event.uuid)
        .expect("Failed to get deleted agent event")
        .expect("Deleted event not found");

    assert_eq!(retrieved_deleted_event.uuid, created_event.uuid);
    assert!(retrieved_deleted_event.deleted_at.is_some());
}