use brokkr_models::models::stacks::NewStack;
use uuid::Uuid;

use crate::fixtures::TestFixture;

#[test]
fn test_create_stack() {
    let fixture = TestFixture::new();

    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
    )
    .expect("Failed to create NewStack");

    let created_stack = fixture.dal.stacks().create(&new_stack).expect("Failed to create stack");

    assert_eq!(created_stack.name, new_stack.name);
    assert_eq!(created_stack.description, new_stack.description);
}
#[test]
fn test_get_stack() {
    let fixture = TestFixture::new();
    let created_stack = fixture.create_test_stack("Test Stack".to_string(), None);

    let retrieved_stack = fixture.dal.stacks().get(created_stack.id).expect("Failed to get stack").unwrap();
    assert_eq!(retrieved_stack.id, created_stack.id);
    assert_eq!(retrieved_stack.name, created_stack.name);
}

#[test]
fn test_get_deleted_stack() {
    let fixture = TestFixture::new();
    let created_stack = fixture.create_test_stack("Test Stack".to_string(), None);
    
    fixture.dal.stacks().soft_delete(created_stack.id).expect("Failed to soft delete stack");

    let retrieved_stack = fixture.dal.stacks().get(created_stack.id).expect("Failed to get stack");
    assert!(retrieved_stack.is_none());

    let retrieved_deleted_stack = fixture.dal.stacks().get_including_deleted(created_stack.id).expect("Failed to get deleted stack").unwrap();
    assert_eq!(retrieved_deleted_stack.id, created_stack.id);
    assert!(retrieved_deleted_stack.deleted_at.is_some());
}

#[test]
fn test_list_stacks() {
    let fixture = TestFixture::new();
    fixture.create_test_stack("Stack 1".to_string(), None);
    let deleted_stack = fixture.create_test_stack("Stack 2".to_string(), None);
    fixture.dal.stacks().soft_delete(deleted_stack.id).expect("Failed to soft delete stack");

    let active_stacks = fixture.dal.stacks().list().expect("Failed to list stacks");
    assert_eq!(active_stacks.len(), 1);
    assert_eq!(active_stacks[0].name, "Stack 1");

    let all_stacks = fixture.dal.stacks().list_all().expect("Failed to list all stacks");
    assert_eq!(all_stacks.len(), 2);
}

#[test]
fn test_update_stack() {
    // ... (existing test remains the same)
}

#[test]
fn test_soft_delete_stack() {
    let fixture = TestFixture::new();
    let created_stack = fixture.create_test_stack("To Be Deleted".to_string(), None);

    let affected_rows = fixture.dal.stacks().soft_delete(created_stack.id).expect("Failed to soft delete stack");
    assert_eq!(affected_rows, 1);

    let deleted_stack = fixture.dal.stacks().get_including_deleted(created_stack.id).expect("Failed to get deleted stack").unwrap();
    assert!(deleted_stack.deleted_at.is_some());
}


#[test]
fn test_hard_delete_stack() {
    let fixture = TestFixture::new();
    let created_stack = fixture.create_test_stack("To Be Hard Deleted".to_string(), None);

    // First, let's soft delete the stack
    fixture.dal.stacks().soft_delete(created_stack.id).expect("Failed to soft delete stack");

    // Verify the stack is still retrievable when including deleted items
    let soft_deleted_stack = fixture.dal.stacks().get_including_deleted(created_stack.id)
        .expect("Failed to get soft-deleted stack")
        .expect("Soft-deleted stack not found");
    assert!(soft_deleted_stack.deleted_at.is_some());

    // Now, let's hard delete the stack
    let affected_rows = fixture.dal.stacks().hard_delete(created_stack.id).expect("Failed to hard delete stack");
    assert_eq!(affected_rows, 1);

    // Verify the stack is no longer retrievable, even when including deleted items
    let hard_deleted_stack = fixture.dal.stacks().get_including_deleted(created_stack.id)
        .expect("Failed to attempt retrieval of hard-deleted stack");
    assert!(hard_deleted_stack.is_none());
}

#[test]
fn test_hard_delete_non_existent_stack() {
    let fixture = TestFixture::new();
    let non_existent_id = Uuid::new_v4();

    let affected_rows = fixture.dal.stacks().hard_delete(non_existent_id).expect("Hard delete operation failed");
    assert_eq!(affected_rows, 0, "No rows should be affected when hard deleting a non-existent stack");
}
