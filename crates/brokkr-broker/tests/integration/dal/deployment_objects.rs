use crate::fixtures::TestFixture;
use brokkr_models::models::deployment_objects::NewDeploymentObject;
use diesel::result::Error as DieselError;

/// Tests the creation of a deployment object.
///
/// This test:
/// 1. Sets up a test fixture and creates a test stack.
/// 2. Creates a new deployment object using the NewDeploymentObject struct.
/// 3. Calls the create method of DeploymentObjectsDAL.
/// 4. Verifies that the created object matches the input data and has the correct sequence ID.
#[test]
fn test_create_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let new_deployment_object = NewDeploymentObject::new(stack_id, "key: value".to_string(), false)
        .expect("Failed to create NewDeploymentObject");

    let created_object = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object)
        .expect("Failed to create deployment object");

    assert_eq!(created_object.stack_id, stack_id);
    assert_eq!(created_object.yaml_content, "key: value");
    assert_eq!(
        created_object.yaml_checksum,
        "b701870861d6ff0565b7078ee799ae7362323298a814d7af4d2dce6cb8d8b674"
    );
    assert_eq!(created_object.sequence_id, 1);
    assert_eq!(created_object.is_deletion_marker, false);
}

/// Tests retrieving a single deployment object by its id.
///
/// This test:
/// 1. Sets up a test fixture and creates a test stack.
/// 2. Creates a new deployment object.
/// 3. Retrieves the object using its id.
/// 4. Verifies that the retrieved object matches the created object.
#[test]
fn test_get_deployment_object_by_id() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let new_deployment_object = NewDeploymentObject::new(stack_id, "key: value".to_string(), false)
        .expect("Failed to create NewDeploymentObject");

    let created_object = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object)
        .expect("Failed to create deployment object");

    let retrieved_object = fixture
        .dal
        .deployment_objects()
        .get_by_id(created_object.id)
        .expect("Failed to get deployment object");

    assert_eq!(retrieved_object.id, created_object.id);
    assert_eq!(retrieved_object.stack_id, stack_id);
}

/// Tests retrieving all deployment objects for a specific stack.
///
/// This test:
/// 1. Sets up a test fixture and creates a test stack.
/// 2. Creates two deployment objects for the stack.
/// 3. Retrieves all objects for the stack.
/// 4. Verifies that both objects are retrieved and have correct sequence IDs.
#[test]
fn test_get_deployment_objects_by_stack_id() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let new_deployment_object1 =
        NewDeploymentObject::new(stack_id, "key1: value1".to_string(), false)
            .expect("Failed to create NewDeploymentObject");

    let new_deployment_object2 =
        NewDeploymentObject::new(stack_id, "key2: value2".to_string(), false)
            .expect("Failed to create NewDeploymentObject");

    fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object1)
        .expect("Failed to create deployment object 1");
    fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object2)
        .expect("Failed to create deployment object 2");

    let retrieved_objects = fixture
        .dal
        .deployment_objects()
        .get_by_stack_id(stack_id)
        .expect("Failed to get deployment objects by stack ID");

    assert_eq!(retrieved_objects.len(), 2);
    assert!(retrieved_objects.iter().any(|obj| obj.sequence_id == 1));
    assert!(retrieved_objects.iter().any(|obj| obj.sequence_id == 2));
}

/// Tests that updating a deployment object is not allowed.
///
/// This test:
/// 1. Sets up a test fixture and creates a test deployment object.
/// 2. Attempts to update the object's content and checksum.
/// 3. Verifies that the update operation fails with the expected error message.
#[test]
fn test_update_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();
    let created_object = fixture.insert_test_deployment_object(stack_id);

    let mut updated_object = created_object.clone();
    updated_object.yaml_content = "updated_key: updated_value".to_string();
    updated_object.yaml_checksum = "updated_checksum".to_string();

    let result = fixture
        .dal
        .deployment_objects()
        .update(created_object.id, &updated_object);

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

/// Tests the soft deletion of a deployment object.
///
/// This test:
/// 1. Sets up a test fixture and creates a test deployment object.
/// 2. Soft deletes the object.
/// 3. Verifies that the object has a deletion timestamp.
/// 4. Checks that the deleted object doesn't appear in the list of active objects.
#[test]
fn test_soft_delete_deployment_object() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let new_deployment_object = NewDeploymentObject::new(stack_id, "key: value".to_string(), false)
        .expect("Failed to create NewDeploymentObject");

    let created_object = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object)
        .expect("Failed to create deployment object");

    let deleted_object = fixture
        .dal
        .deployment_objects()
        .soft_delete(created_object.id)
        .expect("Failed to soft delete deployment object");

    assert!(deleted_object.deleted_at.is_some());

    let active_objects = fixture
        .dal
        .deployment_objects()
        .get_active()
        .expect("Failed to get active deployment objects");
    assert!(!active_objects.iter().any(|obj| obj.id == created_object.id));
}

/// Tests retrieving only active (non-deleted) deployment objects.
///
/// This test:
/// 1. Sets up a test fixture and creates a test stack.
/// 2. Creates two deployment objects.
/// 3. Soft deletes one of the objects.
/// 4. Retrieves active objects.
/// 5. Verifies that only the non-deleted object is returned.
#[test]
fn test_get_active_deployment_objects() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let new_deployment_object1 =
        NewDeploymentObject::new(stack_id, "key1: value1".to_string(), false)
            .expect("Failed to create NewDeploymentObject");

    let new_deployment_object2 =
        NewDeploymentObject::new(stack_id, "key2: value2".to_string(), false)
            .expect("Failed to create NewDeploymentObject");

    let created_object1 = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object1)
        .expect("Failed to create deployment object 1");
    fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object2)
        .expect("Failed to create deployment object 2");

    fixture
        .dal
        .deployment_objects()
        .soft_delete(created_object1.id)
        .expect("Failed to soft delete deployment object");

    let active_objects = fixture
        .dal
        .deployment_objects()
        .get_active()
        .expect("Failed to get active deployment objects");

    assert_eq!(active_objects.len(), 1);
    assert_eq!(active_objects[0].sequence_id, 2);
}

/// Tests creating a deployment object with a duplicate YAML checksum.
///
/// This test:
/// 1. Sets up a test fixture and creates a test stack.
/// 2. Creates a new deployment object with a specific YAML content and checksum.
/// 3. Attempts to create another deployment object with the same checksum.
/// 4. Verifies that the second creation attempt results in an error.
#[test]
fn test_create_deployment_object_with_duplicate_checksum() {
    let fixture = TestFixture::new();
    let stack_id = fixture.insert_test_stack();

    let yaml_content = "key: value".to_string();

    let new_deployment_object1 = NewDeploymentObject::new(stack_id, yaml_content.clone(), false)
        .expect("Failed to create NewDeploymentObject");

    let new_deployment_object2 = NewDeploymentObject::new(stack_id, yaml_content.clone(), false)
        .expect("Failed to create NewDeploymentObject");

    fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object1)
        .expect("Failed to create first deployment object");

    let result = fixture
        .dal
        .deployment_objects()
        .create(&new_deployment_object2);
    assert!(result.is_err());
    // You may want to check for a specific error message or type here
}
