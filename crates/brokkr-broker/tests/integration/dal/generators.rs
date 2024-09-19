use crate::fixtures::TestFixture;
use brokkr_models::models::generator::{Generator, NewGenerator};
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_create_generator() {
    let fixture = TestFixture::new();
    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
    )
    .expect("Failed to create NewGenerator");

    let created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    assert_eq!(created_generator.name, new_generator.name);
    assert_eq!(created_generator.description, new_generator.description);
    assert!(created_generator.is_active);
}

#[test]
fn test_get_generator() {
    let fixture = TestFixture::new();
    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
    )
    .expect("Failed to create NewGenerator");

    let created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    let retrieved_generator = fixture
        .dal
        .generators()
        .get(created_generator.id)
        .expect("Failed to get generator")
        .expect("Generator not found");

    assert_eq!(retrieved_generator.id, created_generator.id);
    assert_eq!(retrieved_generator.name, created_generator.name);
    assert_eq!(retrieved_generator.description, created_generator.description);
}

#[test]
fn test_list_generators() {
    let fixture = TestFixture::new();
    fixture.create_test_generator("Generator 1".to_string(), Some("Description 1".to_string()), "".to_string());
    let deleted_generator = fixture.create_test_generator("Generator 2".to_string(), Some("Description 2".to_string()), "".to_string());
    fixture
        .dal
        .generators()
        .soft_delete(deleted_generator.id)
        .expect("Failed to soft delete generator");

    let active_generators = fixture.dal.generators().list().expect("Failed to list generators");
    assert_eq!(active_generators.len(), 1);
    assert_eq!(active_generators[0].name, "Generator 1");

    let all_generators = fixture
        .dal
        .generators()
        .list_all()
        .expect("Failed to list all generators");
    assert_eq!(all_generators.len(), 2);
}

#[test]
fn test_update_generator() {
    let fixture = TestFixture::new();
    let created_generator = fixture.create_test_generator(
        "Original Generator".to_string(),
        Some("Original Description".to_string()),
        "".to_string(),
    );

    let mut updated_generator = created_generator.clone();
    updated_generator.name = "Updated Generator".to_string();
    updated_generator.description = Some("Updated Description".to_string());

    let result = fixture
        .dal
        .generators()
        .update(created_generator.id, &updated_generator)
        .expect("Failed to update generator");

    assert_eq!(result.name, "Updated Generator");
    assert_eq!(result.description, Some("Updated Description".to_string()));
}

#[test]
fn test_soft_delete_generator() {
    let fixture = TestFixture::new();
    let created_generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "".to_string(),
    );

    fixture
        .dal
        .generators()
        .soft_delete(created_generator.id)
        .expect("Failed to soft delete generator");

    let retrieved_generator = fixture
        .dal
        .generators()
        .get(created_generator.id)
        .expect("Failed to get generator");

    assert!(retrieved_generator.is_none());

    let retrieved_deleted_generator = fixture
        .dal
        .generators()
        .get_including_deleted(created_generator.id)
        .expect("Failed to get deleted generator")
        .expect("Deleted generator not found");

    assert!(retrieved_deleted_generator.deleted_at.is_some());
}

#[test]
fn test_update_pak_hash() {
    let fixture = TestFixture::new();
    let created_generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "".to_string(),
    );

    let new_pak_hash = "new_pak_hash".to_string();
    let updated_generator = fixture
        .dal
        .generators()
        .update_pak_hash(created_generator.id, new_pak_hash.clone())
        .expect("Failed to update pak_hash");

    assert_eq!(updated_generator.pak_hash, Some(new_pak_hash));
}

#[test]
fn test_update_last_active() {
    let fixture = TestFixture::new();
    let created_generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "".to_string(),
    );

    let before_update = Utc::now();
    let updated_generator = fixture
        .dal
        .generators()
        .update_last_active(created_generator.id)
        .expect("Failed to update last_active");

    assert!(updated_generator.last_active_at.is_some());
    assert!(updated_generator.last_active_at.unwrap() > before_update);
    assert!(updated_generator.is_active);
}

#[test]
fn test_get_by_name() {
    let fixture = TestFixture::new();
    let generator_name = "Test Generator".to_string();
    fixture.create_test_generator(generator_name.clone(), Some("Test Description".to_string()), "".to_string());

    let retrieved_generator = fixture
        .dal
        .generators()
        .get_by_name(&generator_name)
        .expect("Failed to get generator by name")
        .expect("Generator not found");

    assert_eq!(retrieved_generator.name, generator_name);
}

#[test]
fn test_get_by_active_status() {
    let fixture = TestFixture::new();
    fixture.create_test_generator("Active Generator".to_string(), Some("Active Description".to_string()), "".to_string());
    let inactive_generator = fixture.create_test_generator("Inactive Generator".to_string(), Some("Inactive Description".to_string()), "".to_string());

    fixture
        .dal
        .generators()
        .update_last_active(inactive_generator.id)
        .expect("Failed to update last_active");

    fixture
        .dal
        .generators()
        .soft_delete(inactive_generator.id)
        .expect("Failed to soft delete generator");

    let active_generators = fixture
        .dal
        .generators()
        .get_by_active_status(true)
        .expect("Failed to get active generators");

    assert_eq!(active_generators.len(), 1);
    assert_eq!(active_generators[0].name, "Active Generator");

    let inactive_generators = fixture
        .dal
        .generators()
        .get_by_active_status(false)
        .expect("Failed to get inactive generators");

    assert_eq!(inactive_generators.len(), 0);
}
