use brokkr_models::models::stacks::{Stack, NewStack};
use uuid::Uuid;

use crate::fixtures::TestFixture;

/// Tests the creation of a stack.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates a new stack using the NewStack struct with various fields.
/// 3. Calls the create method of StacksDAL.
/// 4. Verifies that the created stack matches the input data for all fields.
#[test]
fn test_create_stack() {
    let fixture = TestFixture::new();
    
    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        Some(vec!["test".to_string()]),
        Some(vec![("key".to_string(), "value".to_string())]),
        Some(vec!["agent1".to_string()]),
    ).expect("Failed to create NewStack");

    let created_stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");
    
    assert_eq!(created_stack.name, new_stack.name);
    assert_eq!(created_stack.description, new_stack.description);
    assert_eq!(created_stack.labels, new_stack.labels);
    assert_eq!(created_stack.annotations, new_stack.annotations);
    assert_eq!(created_stack.agent_target, new_stack.agent_target);
}

/// Tests retrieving a single stack by its ID.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates a new stack.
/// 3. Retrieves the stack using its ID.
/// 4. Verifies that the retrieved stack matches the created stack.
#[test]
fn test_get_stack_by_id() {
    let fixture = TestFixture::new();
    
    let new_stack = NewStack::new("Test Stack".to_string(), None, None, None, None).expect("Failed to create NewStack");
    let created_stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    let retrieved_stack = fixture.dal.stacks().get_by_id(created_stack.id).expect("Failed to get stack");
    
    assert_eq!(retrieved_stack.id, created_stack.id);
    assert_eq!(retrieved_stack.name, created_stack.name);
}

/// Tests the behavior when trying to retrieve a non-existent stack.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Attempts to retrieve a stack with a randomly generated UUID.
/// 3. Verifies that the operation results in an error.
#[test]
fn test_get_stack_by_id_not_found() {
    let fixture = TestFixture::new();
    
    let non_existent_id = Uuid::new_v4();
    let result = fixture.dal.stacks().get_by_id(non_existent_id);
    
    assert!(result.is_err());
}

/// Tests retrieving all stacks.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates two stacks.
/// 3. Retrieves all stacks.
/// 4. Verifies that both created stacks are in the retrieved list.
#[test]
fn test_get_all_stacks() {
    let fixture = TestFixture::new();

    let stack1 = NewStack::new("Stack 1".to_string(), None, None, None, None).expect("Failed to create NewStack");
    let stack2 = NewStack::new("Stack 2".to_string(), None, None, None, None).expect("Failed to create NewStack");

    fixture.dal.stacks().create(&stack1).expect("Failed to create stack1");
    fixture.dal.stacks().create(&stack2).expect("Failed to create stack2");

    let all_stacks = fixture.dal.stacks().get_all().expect("Failed to get all stacks");
    
    assert_eq!(all_stacks.len(), 2);
    assert!(all_stacks.iter().any(|s| s.name == "Stack 1"));
    assert!(all_stacks.iter().any(|s| s.name == "Stack 2"));
}

/// Tests updating a stack.
///
/// This test:
/// 1. Sets up a test fixture and creates a new stack.
/// 2. Updates the stack's name and description.
/// 3. Verifies that the update operation succeeds and the stack's fields are updated correctly.
#[test]
fn test_update_stack() {
    let fixture = TestFixture::new();

    let new_stack = NewStack::new("Original Stack".to_string(), None, None, None, None).expect("Failed to create NewStack");
    let created_stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    let mut updated_stack = created_stack.clone();
    updated_stack.name = "Updated Stack".to_string();
    updated_stack.description = Some("Updated description".to_string());

    let result = fixture.dal.stacks().update(created_stack.id, &updated_stack).expect("Failed to update stack");

    assert_eq!(result.name, "Updated Stack");
    assert_eq!(result.description, Some("Updated description".to_string()));
}

/// Tests the behavior when trying to update a non-existent stack.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Attempts to update a stack with a randomly generated UUID.
/// 3. Verifies that the operation results in an error.
#[test]
fn test_update_non_existent_stack() {
    let fixture = TestFixture::new();

    let non_existent_id = Uuid::new_v4();
    let dummy_stack = Stack {
        id: non_existent_id,
        name: "Dummy Stack".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        labels: None,
        annotations: None,
        agent_target: None,
    };

    let result = fixture.dal.stacks().update(non_existent_id, &dummy_stack);
    assert!(result.is_err());
}

/// Tests the soft deletion of a stack.
///
/// This test:
/// 1. Sets up a test fixture and creates a new stack.
/// 2. Soft deletes the stack.
/// 3. Verifies that the stack has a deletion timestamp.
/// 4. Checks that the deleted stack doesn't appear in the list of active stacks.
#[test]
fn test_soft_delete_stack() {
    let fixture = TestFixture::new();

    let new_stack = NewStack::new("To Be Deleted".to_string(), None, None, None, None).expect("Failed to create NewStack");
    let created_stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    let deleted_stack = fixture.dal.stacks().soft_delete(created_stack.id).expect("Failed to soft delete stack");

    assert!(deleted_stack.deleted_at.is_some());

    let active_stacks = fixture.dal.stacks().get_active().expect("Failed to get active stacks");
    assert!(!active_stacks.iter().any(|s| s.id == created_stack.id));
}

/// Tests the behavior when trying to soft delete a non-existent stack.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Attempts to soft delete a stack with a randomly generated UUID.
/// 3. Verifies that the operation results in an error.
#[test]
fn test_soft_delete_non_existent_stack() {
    let fixture = TestFixture::new();

    let non_existent_id = Uuid::new_v4();
    let result = fixture.dal.stacks().soft_delete(non_existent_id);
    
    assert!(result.is_err());
}

/// Tests retrieving only active (non-deleted) stacks.
///
/// This test:
/// 1. Sets up a test fixture.
/// 2. Creates two stacks: one active and one to be deleted.
/// 3. Soft deletes one of the stacks.
/// 4. Retrieves active stacks.
/// 5. Verifies that only the non-deleted stack is returned.
#[test]
fn test_get_active_stacks() {
    let fixture = TestFixture::new();

    let active_stack = NewStack::new("Active Stack".to_string(), None, None, None, None).expect("Failed to create NewStack");
    let to_delete_stack = NewStack::new("To Delete Stack".to_string(), None, None, None, None).expect("Failed to create NewStack");

    let created_active = fixture.dal.stacks().create(&active_stack).expect("Failed to create active stack");
    let created_to_delete = fixture.dal.stacks().create(&to_delete_stack).expect("Failed to create to-delete stack");

    fixture.dal.stacks().soft_delete(created_to_delete.id).expect("Failed to soft delete stack");

    let active_stacks = fixture.dal.stacks().get_active().expect("Failed to get active stacks");

    assert_eq!(active_stacks.len(), 1);
    assert_eq!(active_stacks[0].id, created_active.id);
    assert_eq!(active_stacks[0].name, "Active Stack");
}