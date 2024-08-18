use brokkr_models::models::deployment_objects::NewDeploymentObject;
use diesel::result::Error as DieselError;
use crate::fixtures::TestFixture;


#[test]
fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();

    let new_deployment_object = NewDeploymentObject::new(
        stack_id,
        "key: value".to_string(),
        "checksum123".to_string(),
        1,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let created_object = fixture.dal.deployment_objects().create(&new_deployment_object)
        .expect("Failed to create deployment object");

    assert_eq!(created_object.stack_id, stack_id);
    assert_eq!(created_object.yaml_content, "key: value");
    assert_eq!(created_object.yaml_checksum, "checksum123");
    assert_eq!(created_object.sequence_id, 1);
    assert_eq!(created_object.is_deletion_marker, false);
}

#[test]
fn test_get_deployment_object_by_id() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();

    let new_deployment_object = NewDeploymentObject::new(
        stack_id,
        "key: value".to_string(),
        "checksum123".to_string(),
        1,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let created_object = fixture.dal.deployment_objects().create(&new_deployment_object)
        .expect("Failed to create deployment object");

    let retrieved_object = fixture.dal.deployment_objects().get_by_id(created_object.uuid)
        .expect("Failed to get deployment object");

    assert_eq!(retrieved_object.uuid, created_object.uuid);
    assert_eq!(retrieved_object.stack_id, stack_id);
}

#[test]
fn test_get_deployment_objects_by_stack_id() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();

    let new_deployment_object1 = NewDeploymentObject::new(
        stack_id,
        "key1: value1".to_string(),
        "checksum1".to_string(),
        1,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let new_deployment_object2 = NewDeploymentObject::new(
        stack_id,
        "key2: value2".to_string(),
        "checksum2".to_string(),
        2,
        false,
    ).expect("Failed to create NewDeploymentObject");

    fixture.dal.deployment_objects().create(&new_deployment_object1)
        .expect("Failed to create deployment object 1");
    fixture.dal.deployment_objects().create(&new_deployment_object2)
        .expect("Failed to create deployment object 2");

    let retrieved_objects = fixture.dal.deployment_objects().get_by_stack_id(stack_id)
        .expect("Failed to get deployment objects by stack ID");

    assert_eq!(retrieved_objects.len(), 2);
    assert!(retrieved_objects.iter().any(|obj| obj.sequence_id == 1));
    assert!(retrieved_objects.iter().any(|obj| obj.sequence_id == 2));
}



#[test]
fn test_update_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();
    let created_object = fixture.create_test_deployment_object(stack_id);

    let mut updated_object = created_object.clone();
    updated_object.yaml_content = "updated_key: updated_value".to_string();
    updated_object.yaml_checksum = "updated_checksum".to_string();

    let result = fixture.dal.deployment_objects().update(created_object.uuid, &updated_object);

    assert!(result.is_err());
    match result.unwrap_err() {
        DieselError::DatabaseError(_, error_info) => {
            assert_eq!(
                error_info.message(),
                "Deployment objects cannot be modified except for soft deletion or updating deletion markers"
            );
        }
        _ => panic!("Unexpected error type"),
    }
}

#[test]
fn test_soft_delete_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();

    let new_deployment_object = NewDeploymentObject::new(
        stack_id,
        "key: value".to_string(),
        "checksum123".to_string(),
        1,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let created_object = fixture.dal.deployment_objects().create(&new_deployment_object)
        .expect("Failed to create deployment object");

    let deleted_object = fixture.dal.deployment_objects().soft_delete(created_object.uuid)
        .expect("Failed to soft delete deployment object");

    assert!(deleted_object.deleted_at.is_some());

    let active_objects = fixture.dal.deployment_objects().get_active()
        .expect("Failed to get active deployment objects");
    assert!(!active_objects.iter().any(|obj| obj.uuid == created_object.uuid));
}

#[test]
fn test_get_active_deployment_objects() {
    let fixture = TestFixture::new();
    let stack_id = fixture.create_test_stack();

    let new_deployment_object1 = NewDeploymentObject::new(
        stack_id,
        "key1: value1".to_string(),
        "checksum1".to_string(),
        1,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let new_deployment_object2 = NewDeploymentObject::new(
        stack_id,
        "key2: value2".to_string(),
        "checksum2".to_string(),
        2,
        false,
    ).expect("Failed to create NewDeploymentObject");

    let created_object1 = fixture.dal.deployment_objects().create(&new_deployment_object1)
        .expect("Failed to create deployment object 1");
    fixture.dal.deployment_objects().create(&new_deployment_object2)
        .expect("Failed to create deployment object 2");

    fixture.dal.deployment_objects().soft_delete(created_object1.uuid)
        .expect("Failed to soft delete deployment object");

    let active_objects = fixture.dal.deployment_objects().get_active()
        .expect("Failed to get active deployment objects");

    assert_eq!(active_objects.len(), 1);
    assert_eq!(active_objects[0].sequence_id, 2);
}