use brokkr_models::models::deployment_objects:: NewDeploymentObject;
use crate::fixtures::TestFixture;

#[test]
fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);

    let new_deployment_object = NewDeploymentObject::new(
        stack.id,
        "test yaml content".to_string(),
        false,
    ).expect("Failed to create NewDeploymentObject");

    let created_deployment_object = fixture.dal.deployment_objects().create(&new_deployment_object)
        .expect("Failed to create deployment object");

    assert_eq!(created_deployment_object.stack_id, stack.id);
    assert_eq!(created_deployment_object.yaml_content, "test yaml content");
    assert!(!created_deployment_object.is_deletion_marker);
}

#[test]
fn test_get_deployment_object() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    let retrieved_deployment_object = fixture.dal.deployment_objects().get(deployment_object.id)
        .expect("Failed to get deployment object")
        .unwrap();

    assert_eq!(retrieved_deployment_object.id, deployment_object.id);
    assert_eq!(retrieved_deployment_object.yaml_content, "test yaml content");
}

#[test]
fn test_get_deleted_deployment_object() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    fixture.dal.deployment_objects().soft_delete(deployment_object.id)
        .expect("Failed to soft delete deployment object");

    let retrieved_deployment_object = fixture.dal.deployment_objects().get(deployment_object.id)
        .expect("Failed to get deployment object");
    assert!(retrieved_deployment_object.is_none());

    let retrieved_deleted_deployment_object = fixture.dal.deployment_objects().get_including_deleted(deployment_object.id)
        .expect("Failed to get deleted deployment object")
        .unwrap();
    assert_eq!(retrieved_deleted_deployment_object.id, deployment_object.id);
    assert!(retrieved_deleted_deployment_object.deleted_at.is_some());
}

#[test]
fn test_list_deployment_objects_for_stack() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    fixture.create_test_deployment_object(stack.id, "yaml content 1".to_string(), false);
    fixture.create_test_deployment_object(stack.id, "yaml content 2".to_string(), false);
    let deleted_object = fixture.create_test_deployment_object(stack.id, "yaml content 3".to_string(), false);
    fixture.dal.deployment_objects().soft_delete(deleted_object.id)
        .expect("Failed to soft delete deployment object");

    let active_deployment_objects = fixture.dal.deployment_objects().list_for_stack(stack.id)
        .expect("Failed to list deployment objects");
    assert_eq!(active_deployment_objects.len(), 2);

    let all_deployment_objects = fixture.dal.deployment_objects().list_all_for_stack(stack.id)
        .expect("Failed to list all deployment objects");
    assert_eq!(all_deployment_objects.len(), 3);
}

#[test]
fn test_soft_delete_deployment_object() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    let deployment_object = fixture.create_test_deployment_object(stack.id, "test yaml content".to_string(), false);

    let affected_rows = fixture.dal.deployment_objects().soft_delete(deployment_object.id)
        .expect("Failed to soft delete deployment object");
    assert_eq!(affected_rows, 1);

    let deleted_deployment_object = fixture.dal.deployment_objects().get_including_deleted(deployment_object.id)
        .expect("Failed to get deleted deployment object")
        .unwrap();
    assert!(deleted_deployment_object.deleted_at.is_some());
}

#[test]
fn test_get_latest_deployment_object_for_stack() {
    let fixture = TestFixture::new();
    let stack = fixture.create_test_stack("Test Stack".to_string(), None);
    fixture.create_test_deployment_object(stack.id, "yaml content 1".to_string(), false);
    fixture.create_test_deployment_object(stack.id, "yaml content 2".to_string(), false);
    let latest_object = fixture.create_test_deployment_object(stack.id, "yaml content 3".to_string(), false);

    let retrieved_latest_object = fixture.dal.deployment_objects().get_latest_for_stack(stack.id)
        .expect("Failed to get latest deployment object")
        .unwrap();

    assert_eq!(retrieved_latest_object.id, latest_object.id);
    assert_eq!(retrieved_latest_object.yaml_content, "yaml content 3");
}