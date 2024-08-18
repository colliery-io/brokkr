use brokkr_models::dao::foundations::FoundationDAO;
use brokkr_models::models::NewFoundation;
use std::sync::Arc;
use uuid::Uuid;

// Import TestDb
use crate::common::setup_test_db;

#[test]
fn test_foundation_crud_operations() {
    let (_test_db, conn) = setup_test_db();
    let dao = FoundationDAO::new(Arc::clone(&conn));

    // Test Create
    let new_foundation = NewFoundation::new(
        "Test Foundation".to_string(),
        Some("This is a test foundation".to_string()),
    );
    let created_foundation = dao.create(&new_foundation).unwrap();
    assert_eq!(created_foundation.name, "Test Foundation");
    assert_eq!(
        created_foundation.description,
        Some("This is a test foundation".to_string())
    );

    // Test Get
    let fetched_foundation = dao.get(created_foundation.uuid).unwrap();
    assert_eq!(fetched_foundation, created_foundation);

    // Test Update
    let updated_new_foundation = NewFoundation::new(
        "Updated Test Foundation".to_string(),
        Some("This is an updated test foundation".to_string()),
    );
    let result = dao
        .update(created_foundation.uuid, &updated_new_foundation)
        .unwrap();
    assert_eq!(result.name, "Updated Test Foundation");
    assert_eq!(
        result.description,
        Some("This is an updated test foundation".to_string())
    );

    // Test List
    let foundations = dao.list().unwrap();
    assert_eq!(foundations.len(), 1);
    assert_eq!(foundations[0].uuid, created_foundation.uuid);

    // Test Delete
    let deleted_count = dao.delete(created_foundation.uuid).unwrap();
    assert_eq!(deleted_count, 1);

    // Verify deletion
    let result = dao.get(created_foundation.uuid);
    assert!(result.is_err());
}

#[test]
fn test_foundation_list_multiple() {
    let (_test_db, conn) = setup_test_db();
    let dao = FoundationDAO::new(Arc::clone(&conn));

    // Create multiple foundations
    let foundation1 = NewFoundation::new("Foundation 1".to_string(), None);
    let foundation2 = NewFoundation::new(
        "Foundation 2".to_string(),
        Some("Description 2".to_string()),
    );
    let foundation3 = NewFoundation::new(
        "Foundation 3".to_string(),
        Some("Description 3".to_string()),
    );

    let created1 = dao.create(&foundation1).unwrap();
    let created2 = dao.create(&foundation2).unwrap();
    let created3 = dao.create(&foundation3).unwrap();

    // List all foundations
    let foundations = dao.list().unwrap();

    // Check if all created foundations are in the list
    assert_eq!(foundations.len(), 3);
    assert!(foundations.iter().any(|f| f.uuid == created1.uuid));
    assert!(foundations.iter().any(|f| f.uuid == created2.uuid));
    assert!(foundations.iter().any(|f| f.uuid == created3.uuid));
}

#[test]
fn test_foundation_not_found() {
    let (_test_db, conn) = setup_test_db();
    let dao = FoundationDAO::new(Arc::clone(&conn));

    let non_existent_uuid = Uuid::new_v4();
    let result = dao.get(non_existent_uuid);
    assert!(result.is_err());
}

#[test]
fn test_foundation_update_non_existent() {
    let (_test_db, conn) = setup_test_db();
    let dao = FoundationDAO::new(Arc::clone(&conn));

    let non_existent_uuid = Uuid::new_v4();
    let new_foundation = NewFoundation::new("Non-existent Foundation".to_string(), None);

    let result = dao.update(non_existent_uuid, &new_foundation);
    assert!(result.is_err());
}
