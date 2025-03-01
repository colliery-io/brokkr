use crate::fixtures::TestFixture;
use brokkr_models::models::deployment_objects::NewDeploymentObject;

#[test]
fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_deployment_object =
        NewDeploymentObject::new(stack.id, "test yaml content".to_string(), false)
            .expect("Failed to create NewDeploymentObject");

    let created_deployment_object = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object)
        .expect("Failed to create deployment object");

    assert_eq!(created_deployment_object.stack_id, stack.id);
    assert_eq!(created_deployment_object.yaml_content, "test yaml content");
    assert!(!created_deployment_object.is_deletion_marker);
}

#[test]
fn test_get_deployment_object() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    let retrieved_deployment_object = fixture
        .dal
        .deployment_objects()
        .get(deployment_object.id)
        .expect("Failed to get deployment object")
        .unwrap();

    assert_eq!(retrieved_deployment_object.id, deployment_object.id);
    assert_eq!(
        retrieved_deployment_object.yaml_content,
        "test yaml content"
    );
}

#[test]
fn test_get_deleted_deployment_object() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    fixture
        .dal
        .deployment_objects()
        .soft_delete(deployment_object.id)
        .expect("Failed to soft delete deployment object");

    let retrieved_deployment_object = fixture
        .dal
        .deployment_objects()
        .get(deployment_object.id)
        .expect("Failed to get deployment object");
    assert!(retrieved_deployment_object.is_none());

    let retrieved_deleted_deployment_object = fixture
        .dal
        .deployment_objects()
        .get_including_deleted(deployment_object.id)
        .expect("Failed to get deleted deployment object")
        .unwrap();
    assert_eq!(retrieved_deleted_deployment_object.id, deployment_object.id);
    assert!(retrieved_deleted_deployment_object.deleted_at.is_some());
}

#[test]
fn test_list_deployment_objects_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_deployment_object(stack.id, "yaml content 1".to_string(), false);
    fixture.create_test_deployment_object(stack.id, "yaml content 2".to_string(), false);
    let deleted_object =
        fixture.create_test_deployment_object(stack.id, "yaml content 3".to_string(), false);
    fixture
        .dal
        .deployment_objects()
        .soft_delete(deleted_object.id)
        .expect("Failed to soft delete deployment object");

    let active_deployment_objects = fixture
        .dal
        .deployment_objects()
        .list_for_stack(stack.id)
        .expect("Failed to list deployment objects");
    assert_eq!(active_deployment_objects.len(), 2);

    let all_deployment_objects = fixture
        .dal
        .deployment_objects()
        .list_all_for_stack(stack.id)
        .expect("Failed to list all deployment objects");
    assert_eq!(all_deployment_objects.len(), 3);
}

#[test]
fn test_soft_delete_deployment_object() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let deployment_object =
        fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    let affected_rows = fixture
        .dal
        .deployment_objects()
        .soft_delete(deployment_object.id)
        .expect("Failed to soft delete deployment object");
    assert_eq!(affected_rows, 1);

    let deleted_deployment_object = fixture
        .dal
        .deployment_objects()
        .get_including_deleted(deployment_object.id)
        .expect("Failed to get deleted deployment object")
        .unwrap();
    assert!(deleted_deployment_object.deleted_at.is_some());
}

#[test]
fn test_get_latest_deployment_object_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_deployment_object(stack.id, "yaml content 1".to_string(), false);
    fixture.create_test_deployment_object(stack.id, "yaml content 2".to_string(), false);
    let latest_object =
        fixture.create_test_deployment_object(stack.id, "yaml content 3".to_string(), false);

    let retrieved_latest_object = fixture
        .dal
        .deployment_objects()
        .get_latest_for_stack(stack.id)
        .expect("Failed to get latest deployment object")
        .unwrap();

    assert_eq!(retrieved_latest_object.id, latest_object.id);
    assert_eq!(retrieved_latest_object.yaml_content, "yaml content 3");
}

#[test]
fn test_get_target_state_for_agent_incremental() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create two stacks
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    // Associate the agent with both stacks
    fixture.create_test_agent_target(agent.id, stack1.id);
    fixture.create_test_agent_target(agent.id, stack2.id);

    // Create deployment objects for both stacks
    let object1 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object1".to_string(),
        false,
    );
    let object2 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object2".to_string(),
        false,
    );
    let object3 = fixture.create_test_deployment_object(
        stack2.id,
        "yaml_content: object3".to_string(),
        false,
    );

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - incremental mode (default)
    let target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, false)
        .expect("Failed to get target state for agent");

    // Verify the results
    assert_eq!(target_state.len(), 2, "Expected 2 objects in target state");

    // Check that object2 and object3 are in the target state
    let target_state_ids: Vec<uuid::Uuid> = target_state.iter().map(|obj| obj.id).collect();
    assert!(
        target_state_ids.contains(&object2.id),
        "object2 should be in target state"
    );
    assert!(
        target_state_ids.contains(&object3.id),
        "object3 should be in target state"
    );

    assert!(
        !target_state_ids.contains(&object1.id),
        "object1 should not be in target state (already deployed)"
    );
}

#[test]
fn test_get_target_state_for_agent_full() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create two stacks
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    // Associate the agent with both stacks
    fixture.create_test_agent_target(agent.id, stack1.id);
    fixture.create_test_agent_target(agent.id, stack2.id);

    // Create deployment objects for both stacks
    let object1 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object1".to_string(),
        false,
    );
    let object2 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object2".to_string(),
        false,
    );
    let object3 = fixture.create_test_deployment_object(
        stack2.id,
        "yaml_content: object3".to_string(),
        false,
    );

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - full mode
    let full_target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, true)
        .expect("Failed to get full target state for agent");

    // Verify the results - now expecting only the latest objects per stack
    assert_eq!(
        full_target_state.len(),
        2,
        "Expected 2 objects in full target state (latest from each stack)"
    );

    // Check that the latest objects from each stack are in the full target state
    let full_target_state_ids: Vec<uuid::Uuid> =
        full_target_state.iter().map(|obj| obj.id).collect();
    assert!(
        full_target_state_ids.contains(&object2.id),
        "object2 (latest from stack1) should be in full target state"
    );
    assert!(
        full_target_state_ids.contains(&object3.id),
        "object3 (latest from stack2) should be in full target state"
    );
    assert!(
        !full_target_state_ids.contains(&object1.id),
        "object1 should not be in full target state as it's not the latest"
    );
}

#[test]
fn test_get_target_state_for_agent_with_no_targets() {
    let fixture = TestFixture::new();
    // Create an agent with no targets
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Get target state for the agent
    let target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, false)
        .expect("Failed to get target state for agent");

    // Verify the results
    assert_eq!(target_state.len(), 0, "Expected 0 objects in target state");
}

#[test]

fn test_get_target_state_for_agent_with_all_deployed_incremental() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );

    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a stack
    let stack = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);

    // Associate the agent with the stack
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects for the stack
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);

    // Create agent events for all objects (simulating all deployed)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);
    fixture.create_test_agent_event(&agent, &object2, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - incremental mode
    let target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, false)
        .expect("Failed to get target state for agent");

    // Verify the results
    assert_eq!(target_state.len(), 0, "Expected 0 objects in target state");
}

#[test]
fn test_get_target_state_for_agent_with_all_deployed_full() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create a stack
    let stack = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);

    // Associate the agent with the stack
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects for the stack
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);

    // Create agent events for all objects (simulating all deployed)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);
    fixture.create_test_agent_event(&agent, &object2, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - full mode
    let full_target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, true)
        .expect("Failed to get full target state for agent");

    // Verify the results - now expecting only the latest object
    assert_eq!(
        full_target_state.len(),
        1,
        "Expected 1 object in full target state (only the latest)"
    );

    // Check that only the latest object is in the full target state
    let full_target_state_ids: Vec<uuid::Uuid> =
        full_target_state.iter().map(|obj| obj.id).collect();
    assert!(
        full_target_state_ids.contains(&object2.id),
        "object2 (the latest) should be in full target state"
    );
    assert!(
        !full_target_state_ids.contains(&object1.id),
        "object1 should not be in full target state as it's not the latest"
    );
}

#[test]
fn test_get_target_state_for_agent_with_deletion_markers_incremental() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent(
        "Agent Deletion Markers".to_string(),
        "Test Cluster".to_string(),
    );

    // Create a stack and associate the agent with it
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Stack Deletion Markers".to_string(), None, generator.id);
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects, including a deletion marker
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);
    let deletion_marker = fixture.create_test_deployment_object(
        stack.id,
        "yaml_content: deletion_marker".to_string(),
        true,
    );

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - incremental mode
    let target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, false)
        .expect("Failed to get target state for agent");

    // Verify the results - now expecting only the latest object (which is the deletion marker)
    assert_eq!(
        target_state.len(),
        1,
        "Expected 1 object in target state (only the latest, which is the deletion marker)"
    );

    let target_state_ids: Vec<uuid::Uuid> = target_state.iter().map(|obj| obj.id).collect();
    assert!(
        !target_state_ids.contains(&object1.id),
        "object1 should not be in target state"
    );
    assert!(
        !target_state_ids.contains(&object2.id),
        "object2 should not be in target state as it's not the latest"
    );
    assert!(
        target_state_ids.contains(&deletion_marker.id),
        "deletion marker (the latest) should be included"
    );

    // Verify that the deletion marker is included and has the correct flag
    let deletion_marker_result = target_state
        .iter()
        .find(|obj| obj.id == deletion_marker.id)
        .unwrap();
    assert!(
        deletion_marker_result.is_deletion_marker,
        "Deletion marker should have is_deletion_marker set to true"
    );
}

#[test]
fn test_get_target_state_for_agent_with_deletion_markers_full() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent(
        "Agent Deletion Markers".to_string(),
        "Test Cluster".to_string(),
    );

    // Create a stack and associate the agent with it
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Stack Deletion Markers".to_string(), None, generator.id);
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects, including a deletion marker
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);
    let deletion_marker = fixture.create_test_deployment_object(
        stack.id,
        "yaml_content: deletion_marker".to_string(),
        true,
    );

    // Create an agent event for object1 (simulating a deployed object)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);

    // Get target state for the agent - full mode
    let full_target_state = fixture
        .dal
        .deployment_objects()
        .get_target_state_for_agent(agent.id, true)
        .expect("Failed to get full target state for agent");

    // Verify the results - now expecting only the latest object (the deletion marker)
    assert_eq!(
        full_target_state.len(),
        1,
        "Expected 1 object in full target state (only the latest, which is the deletion marker)"
    );

    // Check that only the latest object is in the full target state
    let full_target_state_ids: Vec<uuid::Uuid> =
        full_target_state.iter().map(|obj| obj.id).collect();
    assert!(
        !full_target_state_ids.contains(&object1.id),
        "object1 should not be in full target state as it's not the latest"
    );
    assert!(
        !full_target_state_ids.contains(&object2.id),
        "object2 should not be in full target state as it's not the latest"
    );
    assert!(
        full_target_state_ids.contains(&deletion_marker.id),
        "deletion marker (the latest) should be included in full target state"
    );

    // Verify that the deletion marker has the correct flag
    let deletion_marker_result = full_target_state
        .iter()
        .find(|obj| obj.id == deletion_marker.id)
        .unwrap();
    assert!(
        deletion_marker_result.is_deletion_marker,
        "Deletion marker should have is_deletion_marker set to true"
    );
}

#[test]
fn test_search_deployment_objects_by_checksum() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    // Create deployment objects with different checksums
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);
    let object3 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);

    // Search for objects with the same checksum as object1
    let search_results = fixture
        .dal
        .deployment_objects()
        .search(&object1.yaml_checksum)
        .expect("Failed to search deployment objects");

    // Verify the results
    assert_eq!(
        search_results.len(),
        2,
        "Expected 2 objects with the same checksum"
    );
    assert!(
        search_results.iter().any(|obj| obj.id == object1.id),
        "object1 should be in the search results"
    );
    assert!(
        search_results.iter().any(|obj| obj.id == object3.id),
        "object3 should be in the search results"
    );
    assert!(
        !search_results.iter().any(|obj| obj.id == object2.id),
        "object2 should not be in the search results"
    );

    // Verify that the objects are sorted by sequence_id in descending order
    assert!(
        search_results[0].sequence_id > search_results[1].sequence_id,
        "Search results should be sorted by sequence_id in descending order"
    );

    // Search for a non-existent checksum
    let non_existent_search = fixture
        .dal
        .deployment_objects()
        .search("non_existent_checksum")
        .expect("Failed to search for non-existent checksum");
    assert!(
        non_existent_search.is_empty(),
        "Search for non-existent checksum should return empty results"
    );
}

#[test]
fn test_get_desired_state_for_agent() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create two stacks
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None, generator.id);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None, generator.id);

    // Associate the agent with both stacks
    fixture.create_test_agent_target(agent.id, stack1.id);
    fixture.create_test_agent_target(agent.id, stack2.id);

    // Create deployment objects for both stacks
    let object1 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object1".to_string(),
        false,
    );
    let object2 = fixture.create_test_deployment_object(
        stack1.id,
        "yaml_content: object2".to_string(),
        false,
    );
    let object3 = fixture.create_test_deployment_object(
        stack2.id,
        "yaml_content: object3".to_string(),
        false,
    );

    // Get desired state for the agent
    let desired_state = fixture
        .dal
        .deployment_objects()
        .get_desired_state_for_agent(agent.id)
        .expect("Failed to get desired state for agent");

    // Verify the results
    assert_eq!(
        desired_state.len(),
        3,
        "Expected 3 objects in desired state"
    );

    let object_ids: Vec<uuid::Uuid> = desired_state.iter().map(|obj| obj.id).collect();
    assert!(
        object_ids.contains(&object1.id),
        "object1 should be in desired state"
    );
    assert!(
        object_ids.contains(&object2.id),
        "object2 should be in desired state"
    );
    assert!(
        object_ids.contains(&object3.id),
        "object3 should be in desired state"
    );

    // Verify that the objects are sorted by sequence_id in descending order
    assert!(
        desired_state[0].sequence_id >= desired_state[1].sequence_id,
        "Objects should be sorted by sequence_id in descending order"
    );
}
