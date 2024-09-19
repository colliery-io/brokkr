use crate::fixtures::TestFixture;
use brokkr_models::models::deployment_objects::NewDeploymentObject;

#[test]
fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
fn test_get_undeployed_objects_for_agent() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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

    // Get undeployed objects for the agent
    let undeployed_objects = fixture
        .dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(agent.id)
        .expect("Failed to get undeployed objects for agent");

    // Verify the results
    assert_eq!(undeployed_objects.len(), 2, "Expected 2 undeployed objects");

    // Check that object2 and object3 are in the undeployed objects list
    let undeployed_ids: Vec<uuid::Uuid> = undeployed_objects.iter().map(|obj| obj.id).collect();
    assert!(
        undeployed_ids.contains(&object2.id),
        "object2 should be undeployed"
    );
    assert!(
        undeployed_ids.contains(&object3.id),
        "object3 should be undeployed"
    );

    // Check that object1 is not in the undeployed objects list
    assert!(
        !undeployed_ids.contains(&object1.id),
        "object1 should not be undeployed"
    );

    // Verify that the objects are sorted by sequence_id in descending order
    assert!(
        undeployed_objects[0].sequence_id > undeployed_objects[1].sequence_id,
        "Undeployed objects should be sorted by sequence_id in descending order"
    );
}

#[test]
fn test_get_undeployed_objects_for_agent_with_no_targets() {
    let fixture = TestFixture::new();

    // Create an agent without any targets
    let agent =
        fixture.create_test_agent("Agent No Targets".to_string(), "Test Cluster".to_string());

    // Get undeployed objects for the agent
    let undeployed_objects = fixture
        .dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(agent.id)
        .expect("Failed to get undeployed objects for agent");

    // Verify the results
    assert!(
        undeployed_objects.is_empty(),
        "Expected no undeployed objects for agent without targets"
    );
}

#[test]
fn test_get_undeployed_objects_for_agent_with_all_deployed() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());

    // Create an agent
    let agent =
        fixture.create_test_agent("Agent All Deployed".to_string(), "Test Cluster".to_string());

    // Create a stack and associate the agent with it
    let stack = fixture.create_test_stack("Stack All Deployed".to_string(), None, generator.id);
    fixture.create_test_agent_target(agent.id, stack.id);

    // Create deployment objects
    let object1 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object1".to_string(), false);
    let object2 =
        fixture.create_test_deployment_object(stack.id, "yaml_content: object2".to_string(), false);

    // Create agent events for all objects (simulating all deployed)
    fixture.create_test_agent_event(&agent, &object1, "DEPLOY", "SUCCESS", None);
    fixture.create_test_agent_event(&agent, &object2, "DEPLOY", "SUCCESS", None);

    // Get undeployed objects for the agent
    let undeployed_objects = fixture
        .dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(agent.id)
        .expect("Failed to get undeployed objects for agent");

    // Verify the results
    assert!(
        undeployed_objects.is_empty(),
        "Expected no undeployed objects when all are deployed"
    );
}

#[test]
fn test_get_undeployed_objects_for_agent_with_deletion_markers() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent(
        "Agent Deletion Markers".to_string(),
        "Test Cluster".to_string(),
    );

    // Create a stack and associate the agent with it
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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

    // Get undeployed objects for the agent
    let undeployed_objects = fixture
        .dal
        .deployment_objects()
        .get_undeployed_objects_for_agent(agent.id)
        .expect("Failed to get undeployed objects for agent");

    // Verify the results
    assert_eq!(
        undeployed_objects.len(),
        2,
        "Expected 2 undeployed objects (including deletion marker)"
    );

    let undeployed_ids: Vec<uuid::Uuid> = undeployed_objects.iter().map(|obj| obj.id).collect();
    assert!(
        undeployed_ids.contains(&object2.id),
        "object2 should be undeployed"
    );
    assert!(
        undeployed_ids.contains(&deletion_marker.id),
        "deletion marker should be included"
    );
    assert!(
        !undeployed_ids.contains(&object1.id),
        "object1 should not be undeployed"
    );

    // Verify that the deletion marker is included and has the correct flag
    let deletion_marker_result = undeployed_objects
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
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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
fn test_get_applicable_deployment_objects() {
    let fixture = TestFixture::new();

    // Create an agent
    let agent = fixture.create_test_agent("Test Agent".to_string(), "Test Cluster".to_string());

    // Create two stacks
    let generator = fixture.create_test_generator("Test Generator".to_string(), None, "test_api_key_hash".to_string());
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

    // Get applicable deployment objects for the agent
    let applicable_objects = fixture
        .dal
        .deployment_objects()
        .get_applicable_deployment_objects(agent.id)
        .expect("Failed to get applicable deployment objects for agent");

    // Verify the results
    assert_eq!(applicable_objects.len(), 3, "Expected 3 applicable objects");

    let object_ids: Vec<uuid::Uuid> = applicable_objects.iter().map(|obj| obj.id).collect();
    assert!(
        object_ids.contains(&object1.id),
        "object1 should be applicable"
    );
    assert!(
        object_ids.contains(&object2.id),
        "object2 should be applicable"
    );
    assert!(
        object_ids.contains(&object3.id),
        "object3 should be applicable"
    );

    // Verify that the objects are sorted by sequence_id in descending order
    assert!(
        applicable_objects[0].sequence_id >= applicable_objects[1].sequence_id,
        "Applicable objects should be sorted by sequence_id in descending order"
    );
    assert!(
        applicable_objects[1].sequence_id >= applicable_objects[2].sequence_id,
        "Applicable objects should be sorted by sequence_id in descending order"
    );
}
