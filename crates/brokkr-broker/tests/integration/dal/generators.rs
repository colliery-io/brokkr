use crate::fixtures::TestFixture;
use brokkr_models::models::generator::{Generator, NewGenerator};
use uuid::Uuid;

#[test]
fn test_create_generator() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
    )
    .expect("Failed to create NewGenerator");

    let created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    assert_eq!(created_generator.name, new_generator.name);
    assert_eq!(created_generator.description, new_generator.description);
    assert_eq!(created_generator.api_key_hash, new_generator.api_key_hash);
    assert!(created_generator.is_active);
}

#[test]
fn test_get_generator() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
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
fn test_update_generator() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
    )
    .expect("Failed to create NewGenerator");

    let mut created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    created_generator.name = "Updated Generator".to_string();
    created_generator.description = Some("Updated Description".to_string());

    let updated_generator = fixture
        .dal
        .generators()
        .update(created_generator.id, &created_generator)
        .expect("Failed to update generator");

    assert_eq!(updated_generator.name, "Updated Generator");
    assert_eq!(updated_generator.description, Some("Updated Description".to_string()));
}

#[test]
fn test_delete_generator() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
    )
    .expect("Failed to create NewGenerator");

    let created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    let deleted_count = fixture
        .dal
        .generators()
        .delete(created_generator.id)
        .expect("Failed to delete generator");

    assert_eq!(deleted_count, 1);

    let retrieved_generator = fixture
        .dal
        .generators()
        .get(created_generator.id)
        .expect("Failed to get generator");

    assert!(retrieved_generator.is_none());
}

#[test]
fn test_list_generators() {
    let fixture = TestFixture::new();

    let new_generator1 = NewGenerator::new(
        "Test Generator 1".to_string(),
        Some("Test Description 1".to_string()),
        "hashed_api_key_1".to_string(),
    )
    .expect("Failed to create NewGenerator");

    let new_generator2 = NewGenerator::new(
        "Test Generator 2".to_string(),
        Some("Test Description 2".to_string()),
        "hashed_api_key_2".to_string(),
    )
    .expect("Failed to create NewGenerator");

    fixture
        .dal
        .generators()
        .create(&new_generator1)
        .expect("Failed to create generator 1");

    fixture
        .dal
        .generators()
        .create(&new_generator2)
        .expect("Failed to create generator 2");

    let generators = fixture
        .dal
        .generators()
        .list()
        .expect("Failed to list generators");

    assert_eq!(generators.len(), 2);
    assert!(generators.iter().any(|g| g.name == "Test Generator 1"));
    assert!(generators.iter().any(|g| g.name == "Test Generator 2"));
}

#[test]
fn test_get_by_api_key() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
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
        .get_by_api_key(&created_generator.api_key_hash)
        .expect("Failed to get generator by API key")
        .expect("Generator not found");

    assert_eq!(retrieved_generator.id, created_generator.id);
    assert_eq!(retrieved_generator.name, created_generator.name);
}

#[test]
fn test_update_last_active() {
    let fixture = TestFixture::new();

    let new_generator = NewGenerator::new(
        "Test Generator".to_string(),
        Some("Test Description".to_string()),
        "hashed_api_key".to_string(),
    )
    .expect("Failed to create NewGenerator");

    let created_generator = fixture
        .dal
        .generators()
        .create(&new_generator)
        .expect("Failed to create generator");

    assert!(created_generator.last_active_at.is_none());

    let updated_generator = fixture
        .dal
        .generators()
        .update_last_active(created_generator.id)
        .expect("Failed to update last active");

    assert!(updated_generator.last_active_at.is_some());
}
