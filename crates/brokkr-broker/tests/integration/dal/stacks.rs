use brokkr_models::models::stacks::NewStack;
use uuid::Uuid;

use crate::fixtures::TestFixture;
use brokkr_broker::dal::FilterType;

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
    let search_stack_id = vec![created_stack.id];
    let retrieved_stack = fixture.dal.stacks().get(search_stack_id).expect("Failed to get stack");
    assert_eq!(retrieved_stack.len(), 1);
    assert_eq!(retrieved_stack[0].id, created_stack.id);
    assert_eq!(retrieved_stack[0].name, created_stack.name);
}

#[test]
fn test_get_deleted_stack() {
    let fixture = TestFixture::new();
    let created_stack = fixture.create_test_stack("Test Stack".to_string(), None);

    
    fixture.dal.stacks().soft_delete(created_stack.id).expect("Failed to soft delete stack");

    let search_stack_id = vec![created_stack.id];
    let retrieved_stack = fixture.dal.stacks().get(search_stack_id).expect("Failed to get stack");
    assert_eq!(retrieved_stack.len(), 0);

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
    // ... (rest of the test remains the same)
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

#[test]
fn test_filter_by_labels_or() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    
    fixture.create_test_stack_label(stack1.id, "label1".to_string());
    fixture.create_test_stack_label(stack1.id, "label2".to_string());
    fixture.create_test_stack_label(stack2.id, "label2".to_string());

    let or_filtered = fixture.dal.stacks().filter_by_labels(vec!["label1".to_string(), "label2".to_string()], FilterType::Or)
        .expect("Failed to filter stacks by labels (OR)");
    assert_eq!(or_filtered.len(), 2);
    assert!(or_filtered.iter().any(|s| s.id == stack1.id));
    assert!(or_filtered.iter().any(|s| s.id == stack2.id));
}

#[test]
fn test_filter_by_labels_and() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    
    fixture.create_test_stack_label(stack1.id, "label1".to_string());
    fixture.create_test_stack_label(stack1.id, "label2".to_string());
    fixture.create_test_stack_label(stack2.id, "label2".to_string());

    let and_filtered = fixture.dal.stacks().filter_by_labels(vec!["label1".to_string(), "label2".to_string()], FilterType::And)
        .expect("Failed to filter stacks by labels (AND)");
    assert_eq!(and_filtered.len(), 1);
    assert_eq!(and_filtered[0].id, stack1.id);
}

#[test]
fn test_filter_by_labels_no_match() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    fixture.create_test_stack_label(stack1.id, "label1".to_string());

    let and_filtered_no_match = fixture.dal.stacks().filter_by_labels(vec!["label1".to_string(), "label3".to_string()], FilterType::And)
        .expect("Failed to filter stacks by non-matching labels (AND)");
    assert_eq!(and_filtered_no_match.len(), 0);
}

#[test]
fn test_filter_by_labels_empty_input() {
    let fixture = TestFixture::new();
    
    let empty_filter = fixture.dal.stacks().filter_by_labels(vec![], FilterType::Or)
        .expect("Failed to filter stacks with empty label list");
    assert_eq!(empty_filter.len(), 0);
}

#[test]
fn test_filter_by_labels_non_existent() {
    let fixture = TestFixture::new();
    
    let non_matching = fixture.dal.stacks().filter_by_labels(vec!["non_existent_label".to_string()], FilterType::Or)
        .expect("Failed to filter stacks with non-matching label");
    assert_eq!(non_matching.len(), 0);
}

#[test]
fn test_filter_by_labels_duplicate() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    
    fixture.create_test_stack_label(stack1.id, "label2".to_string());
    fixture.create_test_stack_label(stack2.id, "label2".to_string());

    let filtered = fixture.dal.stacks().filter_by_labels(vec!["label2".to_string(), "label2".to_string()], FilterType::Or)
        .expect("Failed to filter stacks by duplicate labels");
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().any(|s| s.id == stack1.id));
    assert!(filtered.iter().any(|s| s.id == stack2.id));
}

#[test]
fn test_filter_by_labels_mixed_existing_and_non_existent() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    fixture.create_test_stack_label(stack1.id, "label1".to_string());

    let non_existent_label = "non_existent_label".to_string();
    
    let or_filtered = fixture.dal.stacks().filter_by_labels(vec!["label1".to_string(), non_existent_label.clone()], FilterType::Or)
        .expect("Failed to filter stacks with mix of existing and non-existent labels (OR)");
    assert_eq!(or_filtered.len(), 1, "OR filtering with mix of labels should return results for existing labels");
    assert_eq!(or_filtered[0].id, stack1.id);

    let and_filtered = fixture.dal.stacks().filter_by_labels(vec!["label1".to_string(), non_existent_label], FilterType::And)
        .expect("Failed to filter stacks with mix of existing and non-existent labels (AND)");
    assert_eq!(and_filtered.len(), 0, "AND filtering with mix of labels including non-existent should return empty result");
}

#[test]
fn test_filter_by_annotations() {
    let fixture = TestFixture::new();
    
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    let stack3 = fixture.create_test_stack("Stack 3".to_string(), None);
    
    fixture.create_test_stack_annotation(stack1.id, "key1", "value1");
    fixture.create_test_stack_annotation(stack1.id, "key2", "value2");
    fixture.create_test_stack_annotation(stack2.id, "key2", "value2");
    fixture.create_test_stack_annotation(stack3.id, "key3", "value3");

    // Test OR filter
    let or_filtered = fixture.dal.stacks().filter_by_annotations(
        vec![("key1".to_string(), "value1".to_string()), ("key2".to_string(), "value2".to_string())],
        FilterType::Or
    ).expect("Failed to filter stacks by annotations (OR)");
    assert_eq!(or_filtered.len(), 2);
    assert!(or_filtered.iter().any(|s| s.id == stack1.id));
    assert!(or_filtered.iter().any(|s| s.id == stack2.id));

    // Test AND filter
    let and_filtered = fixture.dal.stacks().filter_by_annotations(
        vec![("key1".to_string(), "value1".to_string()), ("key2".to_string(), "value2".to_string())],
        FilterType::And
    ).expect("Failed to filter stacks by annotations (AND)");
    assert_eq!(and_filtered.len(), 1);
    assert_eq!(and_filtered[0].id, stack1.id);

    // Test empty input
    let empty_filter = fixture.dal.stacks().filter_by_annotations(vec![], FilterType::Or)
        .expect("Failed to filter stacks with empty annotation list");
    assert_eq!(empty_filter.len(), 0);

    // Test non-matching filter
    let non_matching = fixture.dal.stacks().filter_by_annotations(
        vec![("non_existent_key".to_string(), "non_existent_value".to_string())],
        FilterType::Or
    ).expect("Failed to filter stacks with non-matching annotation");
    assert_eq!(non_matching.len(), 0);
}

#[test]
fn test_get_associated_stacks() {

    println!("Starting test_get_associated_stacks");
    let fixture = TestFixture::new();
    println!("Created TestFixture");
        
     // Try to list all stacks
     let stacks = fixture.dal.stacks().list().expect("Failed to list stacks");
     println!("Number of stacks: {}", stacks.len());
    

    // Create agents
    let agent1 = fixture.create_test_agent("Agent 1".to_string(), "Cluster 1".to_string());
    let agent2 = fixture.create_test_agent("Agent 2".to_string(), "Cluster 2".to_string());
    
    // Create stacks
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    let stack3 = fixture.create_test_stack("Stack 3".to_string(), None);
    let stack4 = fixture.create_test_stack("Stack 4".to_string(), None);
    
    // Add labels
    println!("Adding labels to agent and stacks");
    fixture.create_test_agent_label(agent1.id, "label1".to_string());
    fixture.create_test_agent_label(agent1.id, "label2".to_string());
    fixture.create_test_stack_label(stack1.id, "label1".to_string());
    fixture.create_test_stack_label(stack2.id, "label2".to_string());
    
    // Verify labels
    let agent_labels = fixture.dal.agent_labels().list_for_agent(agent1.id).expect("Failed to list agent labels");
    println!("Agent labels: {:?}", agent_labels);
    let stack1_labels = fixture.dal.stack_labels().list_for_stack(stack1.id).expect("Failed to list stack1 labels");
    println!("Stack1 labels: {:?}", stack1_labels);
    let stack2_labels = fixture.dal.stack_labels().list_for_stack(stack2.id).expect("Failed to list stack2 labels");
    println!("Stack2 labels: {:?}", stack2_labels);
    
    // Add annotations
    fixture.create_test_agent_annotation(agent1.id, "key1".to_string(), "value1".to_string());
    fixture.create_test_agent_annotation(agent1.id, "key2".to_string(), "value2".to_string());
    fixture.create_test_stack_annotation(stack2.id, "key1", "value1");
    fixture.create_test_stack_annotation(stack3.id, "key2", "value2");
    
    // Add targets
    fixture.create_test_agent_target(agent1.id, stack3.id);
    fixture.create_test_agent_target(agent1.id, stack4.id);

    // Test get_associated_stacks
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent1.id).unwrap();
    
    assert_eq!(associated_stacks.len(), 4);
    assert!(associated_stacks.iter().any(|s| s.id == stack1.id));
    assert!(associated_stacks.iter().any(|s| s.id == stack2.id));
    assert!(associated_stacks.iter().any(|s| s.id == stack3.id));
    assert!(associated_stacks.iter().any(|s| s.id == stack4.id));

    // Test with agent having no associations
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent2.id).unwrap();
    assert!(associated_stacks.is_empty());

    // Test with non-existent agent
    let non_existent_uuid = Uuid::new_v4();
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(non_existent_uuid).unwrap();
    assert!(associated_stacks.is_empty());

    // Test with deleted stack
    fixture.dal.stacks().soft_delete(stack1.id).unwrap();
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent1.id).unwrap();
    assert_eq!(associated_stacks.len(), 3);
    assert!(!associated_stacks.iter().any(|s| s.id == stack1.id));

    // Test with only labels
    let agent3 = fixture.create_test_agent("Agent 3".to_string(), "Cluster 3".to_string());
    fixture.create_test_agent_label(agent3.id, "label1".to_string());
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent3.id).unwrap();
    assert_eq!(associated_stacks.len(), 0); // stack1 is deleted

    // Test with only annotations
    let agent4 = fixture.create_test_agent("Agent 4".to_string(), "Cluster 4".to_string());
    fixture.create_test_agent_annotation(agent4.id, "key1".to_string(), "value1".to_string());
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent4.id).unwrap();
    assert_eq!(associated_stacks.len(), 1);
    assert!(associated_stacks.iter().any(|s| s.id == stack2.id));

    // Test with only targets
    let agent5 = fixture.create_test_agent("Agent 5".to_string(), "Cluster 5".to_string());
    fixture.create_test_agent_target(agent5.id, stack4.id);
    let associated_stacks = fixture.dal.stacks().get_associated_stacks(agent5.id).unwrap();
    assert_eq!(associated_stacks.len(), 1);
    assert!(associated_stacks.iter().any(|s| s.id == stack4.id));
}

