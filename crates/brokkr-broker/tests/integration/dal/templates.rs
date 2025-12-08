/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;

const TEST_TEMPLATE_CONTENT: &str = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}"#;

#[test]
fn test_create_template() {
    let fixture = TestFixture::new();

    let template = fixture
        .dal
        .templates()
        .create_new_version(
            None,
            "test-template".to_string(),
            Some("A test template".to_string()),
            TEST_TEMPLATE_CONTENT.to_string(),
            "{}".to_string(),
        )
        .expect("Failed to create template");

    assert_eq!(template.name, "test-template");
    assert_eq!(template.version, 1);
    assert!(template.generator_id.is_none());
}

#[test]
fn test_create_template_with_generator() {
    let fixture = TestFixture::new();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_hash".to_string(),
    );

    let template = fixture
        .dal
        .templates()
        .create_new_version(
            Some(generator.id),
            "generator-template".to_string(),
            None,
            TEST_TEMPLATE_CONTENT.to_string(),
            "{}".to_string(),
        )
        .expect("Failed to create template");

    assert_eq!(template.generator_id, Some(generator.id));
}

#[test]
fn test_get_template() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "test-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let fetched = fixture
        .dal
        .templates()
        .get(template.id)
        .expect("Failed to fetch template")
        .expect("Template not found");

    assert_eq!(fetched.id, template.id);
    assert_eq!(fetched.name, template.name);
}

#[test]
fn test_list_templates() {
    let fixture = TestFixture::new();

    fixture.create_test_template(
        None,
        "template-1".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );
    fixture.create_test_template(
        None,
        "template-2".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let templates = fixture
        .dal
        .templates()
        .list()
        .expect("Failed to list templates");

    assert!(templates.len() >= 2);
}

#[test]
fn test_list_templates_by_generator() {
    let fixture = TestFixture::new();

    let generator = fixture.create_test_generator(
        "Test Generator".to_string(),
        None,
        "test_hash".to_string(),
    );

    fixture.create_test_template(
        Some(generator.id),
        "generator-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );
    fixture.create_test_template(
        None,
        "system-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let templates = fixture
        .dal
        .templates()
        .list_by_generator(generator.id)
        .expect("Failed to list templates");

    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].generator_id, Some(generator.id));
}

#[test]
fn test_versioning() {
    let fixture = TestFixture::new();

    // Create version 1
    let v1 = fixture
        .dal
        .templates()
        .create_new_version(
            None,
            "versioned-template".to_string(),
            Some("Version 1".to_string()),
            "v1 content".to_string(),
            "{}".to_string(),
        )
        .expect("Failed to create v1");

    assert_eq!(v1.version, 1);

    // Create version 2
    let v2 = fixture
        .dal
        .templates()
        .create_new_version(
            None,
            "versioned-template".to_string(),
            Some("Version 2".to_string()),
            "v2 content".to_string(),
            "{}".to_string(),
        )
        .expect("Failed to create v2");

    assert_eq!(v2.version, 2);
    assert_ne!(v1.id, v2.id);
}

#[test]
fn test_get_latest_version() {
    let fixture = TestFixture::new();

    // Create multiple versions
    fixture.create_test_template(
        None,
        "versioned".to_string(),
        Some("v1".to_string()),
        "v1".to_string(),
        "{}".to_string(),
    );
    fixture.create_test_template(
        None,
        "versioned".to_string(),
        Some("v2".to_string()),
        "v2".to_string(),
        "{}".to_string(),
    );

    let latest = fixture
        .dal
        .templates()
        .get_latest_version(None, "versioned")
        .expect("Failed to get latest version")
        .expect("Latest version not found");

    assert_eq!(latest.version, 2);
    assert_eq!(latest.description, Some("v2".to_string()));
}

#[test]
fn test_list_versions() {
    let fixture = TestFixture::new();

    // Create 3 versions
    for i in 1..=3 {
        fixture.create_test_template(
            None,
            "multi-version".to_string(),
            Some(format!("Version {}", i)),
            format!("content {}", i),
            "{}".to_string(),
        );
    }

    let versions = fixture
        .dal
        .templates()
        .list_versions(None, "multi-version")
        .expect("Failed to list versions");

    assert_eq!(versions.len(), 3);
    // Should be ordered by version
    assert_eq!(versions[0].version, 1);
    assert_eq!(versions[1].version, 2);
    assert_eq!(versions[2].version, 3);
}

#[test]
fn test_soft_delete() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "to-delete".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    fixture
        .dal
        .templates()
        .soft_delete(template.id)
        .expect("Failed to soft delete");

    // Should not be found in normal get
    let fetched = fixture
        .dal
        .templates()
        .get(template.id)
        .expect("Failed to fetch");

    assert!(fetched.is_none());
}

#[test]
fn test_template_labels() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "labeled-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    // Add labels
    fixture.create_test_template_label(template.id, "env=prod".to_string());
    fixture.create_test_template_label(template.id, "tier=1".to_string());

    let labels = fixture
        .dal
        .template_labels()
        .list_for_template(template.id)
        .expect("Failed to list labels");

    assert_eq!(labels.len(), 2);
}

#[test]
fn test_template_annotations() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "annotated-template".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    // Add annotations
    fixture.create_test_template_annotation(template.id, "region", "us-east");
    fixture.create_test_template_annotation(template.id, "owner", "platform-team");

    let annotations = fixture
        .dal
        .template_annotations()
        .list_for_template(template.id)
        .expect("Failed to list annotations");

    assert_eq!(annotations.len(), 2);
}

#[test]
fn test_delete_label() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "label-delete-test".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let label = fixture.create_test_template_label(template.id, "to-delete".to_string());

    fixture
        .dal
        .template_labels()
        .delete(label.id)
        .expect("Failed to delete label");

    let labels = fixture
        .dal
        .template_labels()
        .list_for_template(template.id)
        .expect("Failed to list labels");

    assert_eq!(labels.len(), 0);
}

#[test]
fn test_delete_annotation() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "annotation-delete-test".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let annotation = fixture.create_test_template_annotation(template.id, "to-delete", "value");

    fixture
        .dal
        .template_annotations()
        .delete(annotation.id)
        .expect("Failed to delete annotation");

    let annotations = fixture
        .dal
        .template_annotations()
        .list_for_template(template.id)
        .expect("Failed to list annotations");

    assert_eq!(annotations.len(), 0);
}

#[test]
fn test_checksum_generation() {
    let fixture = TestFixture::new();

    let template = fixture.create_test_template(
        None,
        "checksum-test".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    // Checksum should be a 64-character hex string (SHA-256)
    assert_eq!(template.checksum.len(), 64);
    assert!(template.checksum.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_same_content_same_checksum() {
    let fixture = TestFixture::new();

    let t1 = fixture.create_test_template(
        None,
        "checksum-test-1".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    let t2 = fixture.create_test_template(
        None,
        "checksum-test-2".to_string(),
        None,
        TEST_TEMPLATE_CONTENT.to_string(),
        "{}".to_string(),
    );

    assert_eq!(t1.checksum, t2.checksum);
}
