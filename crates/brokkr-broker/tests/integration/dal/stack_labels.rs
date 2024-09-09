use brokkr_models::models::stack_labels::NewStackLabel;
use crate::fixtures::TestFixture;

#[test]
fn test_create_stack_label() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);

    let new_label = NewStackLabel::new(stack.id, "test-label".to_string()).expect("Failed to create NewStackLabel");
    let created_label = fixture.dal.stack_labels().create(&new_label).expect("Failed to create stack label");

    assert_eq!(created_label.stack_id, stack.id);
    assert_eq!(created_label.label, "test-label");
}

#[test]
fn test_get_stack_label() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let label = fixture.create_test_stack_label(stack.id, "test-label".to_string());

    let retrieved_label = fixture.dal.stack_labels().get(label.id).expect("Failed to get stack label").unwrap();
    assert_eq!(retrieved_label.id, label.id);
    assert_eq!(retrieved_label.label, "test-label");
}

#[test]
fn test_list_labels_for_stack() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    fixture.create_test_stack_label(stack.id, "label1".to_string());
    fixture.create_test_stack_label(stack.id, "label2".to_string());

    let labels = fixture.dal.stack_labels().list_for_stack(stack.id).expect("Failed to list stack labels");
    assert_eq!(labels.len(), 2);
    assert!(labels.iter().any(|l| l.label == "label1"));
    assert!(labels.iter().any(|l| l.label == "label2"));
}

#[test]
fn test_delete_stack_label() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let label = fixture.create_test_stack_label(stack.id, "test-label".to_string());

    let affected_rows = fixture.dal.stack_labels().delete(label.id).expect("Failed to delete stack label");
    assert_eq!(affected_rows, 1);

    let deleted_label = fixture.dal.stack_labels().get(label.id).expect("Failed to get deleted stack label");
    assert!(deleted_label.is_none());
}

#[test]
fn test_delete_all_labels_for_stack() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    fixture.create_test_stack_label(stack.id, "label1".to_string());
    fixture.create_test_stack_label(stack.id, "label2".to_string());

    let affected_rows = fixture.dal.stack_labels().delete_all_for_stack(stack.id).expect("Failed to delete all stack labels");
    assert_eq!(affected_rows, 2);

    let remaining_labels = fixture.dal.stack_labels().list_for_stack(stack.id).expect("Failed to list stack labels");
    assert!(remaining_labels.is_empty());
}

#[test]
fn test_search_stack_labels() {
    let fixture = TestFixture::new();
    let stack1 = fixture.create_test_stack("Stack 1".to_string(), None);
    let stack2 = fixture.create_test_stack("Stack 2".to_string(), None);
    fixture.create_test_stack_label(stack1.id, "alpha-label".to_string());
    fixture.create_test_stack_label(stack2.id, "beta-label".to_string());

    let search_results = fixture.dal.stack_labels().search("alpha").expect("Failed to search stack labels");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].label, "alpha-label");
}