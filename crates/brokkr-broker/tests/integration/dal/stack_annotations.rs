use crate::fixtures::TestFixture;
use brokkr_models::models::stack_annotations::NewStackAnnotation;

#[test]
fn test_create_stack_annotation() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);

    let new_annotation = NewStackAnnotation {
        stack_id: stack.id,
        key: "test_key".to_string(),
        value: "test_value".to_string(),
    };

    let created_annotation = fixture
        .dal
        .stack_annotations()
        .create(&new_annotation)
        .expect("Failed to create stack annotation");

    assert_eq!(created_annotation.stack_id, stack.id);
    assert_eq!(created_annotation.key, "test_key");
    assert_eq!(created_annotation.value, "test_value");
}

#[test]
fn test_get_stack_annotation() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let annotation = fixture.create_test_stack_annotation(stack.id, "test_key", "test_value");

    let retrieved_annotation = fixture
        .dal
        .stack_annotations()
        .get(annotation.id)
        .expect("Failed to get stack annotation")
        .expect("Stack annotation not found");

    assert_eq!(retrieved_annotation.id, annotation.id);
    assert_eq!(retrieved_annotation.key, "test_key");
    assert_eq!(retrieved_annotation.value, "test_value");
}

#[test]
fn test_list_annotations_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_annotation(stack.id, "key1", "value1");
    fixture.create_test_stack_annotation(stack.id, "key2", "value2");

    let annotations = fixture
        .dal
        .stack_annotations()
        .list_for_stack(stack.id)
        .expect("Failed to list stack annotations");

    assert_eq!(annotations.len(), 2);
    assert!(annotations
        .iter()
        .any(|a| a.key == "key1" && a.value == "value1"));
    assert!(annotations
        .iter()
        .any(|a| a.key == "key2" && a.value == "value2"));
}

#[test]
fn test_update_stack_annotation() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let annotation = fixture.create_test_stack_annotation(stack.id, "old_key", "old_value");

    let mut updated_annotation = annotation.clone();
    updated_annotation.key = "new_key".to_string();
    updated_annotation.value = "new_value".to_string();

    let result = fixture
        .dal
        .stack_annotations()
        .update(annotation.id, &updated_annotation)
        .expect("Failed to update stack annotation");

    assert_eq!(result.key, "new_key");
    assert_eq!(result.value, "new_value");
}

#[test]
fn test_delete_stack_annotation() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    let annotation = fixture.create_test_stack_annotation(stack.id, "test_key", "test_value");

    let affected_rows = fixture
        .dal
        .stack_annotations()
        .delete(annotation.id)
        .expect("Failed to delete stack annotation");
    assert_eq!(affected_rows, 1);

    let deleted_annotation = fixture
        .dal
        .stack_annotations()
        .get(annotation.id)
        .expect("Failed to attempt retrieval of deleted annotation");
    assert!(deleted_annotation.is_none());
}

#[test]
fn test_delete_all_annotations_for_stack() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_api_key_hash".to_string(),
    );
    let stack = fixture.create_test_stack("Test Stack".to_string(), None, generator.id);
    fixture.create_test_stack_annotation(stack.id, "key1", "value1");
    fixture.create_test_stack_annotation(stack.id, "key2", "value2");

    let affected_rows = fixture
        .dal
        .stack_annotations()
        .delete_all_for_stack(stack.id)
        .expect("Failed to delete all stack annotations");
    assert_eq!(affected_rows, 2);

    let remaining_annotations = fixture
        .dal
        .stack_annotations()
        .list_for_stack(stack.id)
        .expect("Failed to list stack annotations after deletion");
    assert!(remaining_annotations.is_empty());
}
