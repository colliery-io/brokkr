use brokkr_models::dao::foundations::FoundationDAO;
use brokkr_models::dao::kubernetes_objects::KubernetesObjectDAO;
use brokkr_models::models::{Foundation, NewFoundation, NewKubernetesObject};
use diesel::pg::PgConnection;
use serde_json::json;
use std::sync::{Arc, Mutex};

// Import TestDb
use crate::common::setup_test_db;

fn create_foundation(conn: &Arc<Mutex<PgConnection>>) -> Foundation {
    let foundation_dao = FoundationDAO::new(Arc::clone(conn));
    let new_foundation = NewFoundation::new(
        "Test Foundation".to_string(),
        Some("For testing".to_string()),
    );
    foundation_dao.create(&new_foundation).unwrap()
}

#[test]
fn test_kubernetes_object_crud_operations() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Test Create
    let new_object = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: test-pod".to_string(),
        "Pod".to_string(),
        "test-pod".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test"])),
        Some(json!([["description", "Test pod"]])),
    );
    let created_object = dao.create(&new_object).unwrap();
    assert_eq!(created_object.object_type, "Pod");
    assert_eq!(created_object.object_name, "test-pod");

    // Test Get
    let fetched_object = dao.get(created_object.uuid).unwrap();
    assert_eq!(fetched_object, created_object);

    // Test Update
    let updated_new_object = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: updated-test-pod".to_string(),
        "Pod".to_string(),
        "updated-test-pod".to_string(),
        Some("default".to_string()),
        None,
        false,
        2,
        Some(json!(["app", "test", "updated"])),
        Some(json!([["description", "Updated test pod"]])),
    );
    let result = dao
        .update(created_object.uuid, &updated_new_object)
        .unwrap();
    assert_eq!(result.object_name, "updated-test-pod");
    assert_eq!(result.version, 2);

    // Test List
    let objects = dao.list().unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].uuid, created_object.uuid);

    // Test Delete
    let deleted_count = dao.delete(created_object.uuid).unwrap();
    assert_eq!(deleted_count, 1);

    // Verify deletion
    let result = dao.get(created_object.uuid);
    assert!(result.is_err());
}

#[test]
fn test_kubernetes_object_find_by_label_containing() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Create objects with different labels
    let object1 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod1".to_string(),
        "Pod".to_string(),
        "pod1".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test-app", "version", "1.0.0"])),
        Some(json!([["description", "Test app pod"]])),
    );
    let object2 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod2".to_string(),
        "Pod".to_string(),
        "pod2".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "production-app", "version", "2.0.0"])),
        Some(json!([["description", "Production app pod"]])),
    );

    dao.create(&object1).unwrap();
    dao.create(&object2).unwrap();

    // Test finding by label containing
    let test_objects = dao.find_by_label_containing("test").unwrap();
    assert_eq!(test_objects.len(), 1);
    assert_eq!(test_objects[0].object_name, "pod1");

    let app_objects = dao.find_by_label_containing("app").unwrap();
    assert_eq!(app_objects.len(), 2);

    let version_objects = dao.find_by_label_containing("1.0").unwrap();
    assert_eq!(version_objects.len(), 1);
    assert_eq!(version_objects[0].object_name, "pod1");

    let non_existent_objects = dao.find_by_label_containing("non-existent").unwrap();
    assert_eq!(non_existent_objects.len(), 0);
}

#[test]
fn test_kubernetes_object_find_by_annotation() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Create objects with different annotations
    let object1 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod1".to_string(),
        "Pod".to_string(),
        "pod1".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test"])),
        Some(json!([
            ["description", "Test pod"],
            ["created-by", "user1"]
        ])),
    );
    let object2 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod2".to_string(),
        "Pod".to_string(),
        "pod2".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "prod"])),
        Some(json!([
            ["description", "Production pod"],
            ["created-by", "user2"]
        ])),
    );

    dao.create(&object1).unwrap();
    dao.create(&object2).unwrap();

    // Test finding by annotation
    let test_objects = dao.find_by_annotation("description", "Test pod").unwrap();
    assert_eq!(test_objects.len(), 1);
    assert_eq!(test_objects[0].object_name, "pod1");

    let user1_objects = dao.find_by_annotation("created-by", "user1").unwrap();
    assert_eq!(user1_objects.len(), 1);
    assert_eq!(user1_objects[0].object_name, "pod1");

    let all_objects = dao.find_by_annotation("description", "pod").unwrap();
    assert_eq!(all_objects.len(), 0); // This should be 0 because we're looking for an exact match

    let non_existent_objects = dao.find_by_annotation("non-existent", "value").unwrap();
    assert_eq!(non_existent_objects.len(), 0);
}

#[test]
fn test_kubernetes_object_find_by_label() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Create objects with different labels
    let object1 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod1".to_string(),
        "Pod".to_string(),
        "pod1".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "environment", "dev"])),
        Some(json!([["description", "Test pod"]])),
    );
    let object2 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod2".to_string(),
        "Pod".to_string(),
        "pod2".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "environment", "prod"])),
        Some(json!([["description", "Production pod"]])),
    );

    dao.create(&object1).unwrap();
    dao.create(&object2).unwrap();

    // Test finding by label
    let environment_objects = dao.find_by_label("environment").unwrap();
    assert_eq!(environment_objects.len(), 2);

    let dev_objects = dao.find_by_label("dev").unwrap();
    assert_eq!(dev_objects.len(), 1);
    assert_eq!(dev_objects[0].object_name, "pod1");

    let non_existent_objects = dao.find_by_label("non-existent").unwrap();
    assert_eq!(non_existent_objects.len(), 0);
}

#[test]
fn test_kubernetes_object_find_by_any_label() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Create objects with different labels
    let object1 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod1".to_string(),
        "Pod".to_string(),
        "pod1".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "environment", "dev"])),
        Some(json!([["description", "Test pod"]])),
    );
    let object2 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod2".to_string(),
        "Pod".to_string(),
        "pod2".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "tier", "backend"])),
        Some(json!([["description", "Backend pod"]])),
    );

    dao.create(&object1).unwrap();
    dao.create(&object2).unwrap();

    // Test finding by any label
    let objects = dao.find_by_any_label(&["environment", "tier"]).unwrap();
    assert_eq!(objects.len(), 2);

    let dev_objects = dao.find_by_any_label(&["dev"]).unwrap();
    assert_eq!(dev_objects.len(), 1);
    assert_eq!(dev_objects[0].object_name, "pod1");

    let non_existent_objects = dao.find_by_any_label(&["non-existent"]).unwrap();
    assert_eq!(non_existent_objects.len(), 0);
}

#[test]
fn test_kubernetes_object_find_by_label_in() {
    let (_test_db, conn) = setup_test_db();
    let dao = KubernetesObjectDAO::new(Arc::clone(&conn));

    // Create a foundation first
    let foundation = create_foundation(&conn);

    // Create objects with different labels
    let object1 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod1".to_string(),
        "Pod".to_string(),
        "pod1".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "environment", "dev"])),
        Some(json!([["description", "Test pod"]])),
    );
    let object2 = NewKubernetesObject::new(
        foundation.uuid,
        "apiVersion: v1\nkind: Pod\nmetadata:\n  name: pod2".to_string(),
        "Pod".to_string(),
        "pod2".to_string(),
        Some("default".to_string()),
        None,
        false,
        1,
        Some(json!(["app", "test", "environment", "prod"])),
        Some(json!([["description", "Production pod"]])),
    );

    dao.create(&object1).unwrap();
    dao.create(&object2).unwrap();

    // Test finding by label in
    let objects = dao.find_by_label_in(&["dev", "prod"]).unwrap();
    assert_eq!(objects.len(), 2);

    let dev_objects = dao.find_by_label_in(&["dev"]).unwrap();
    assert_eq!(dev_objects.len(), 1);
    assert_eq!(dev_objects[0].object_name, "pod1");

    let non_existent_objects = dao.find_by_label_in(&["staging"]).unwrap();
    assert_eq!(non_existent_objects.len(), 0);
}
