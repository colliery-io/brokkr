/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for CLI command functions (BROKKR-T-0186 /
//! BROKKR-T-0189): PAK rotation must return the new credential and record
//! synchronous audit entries.

use crate::fixtures::TestFixture;
use brokkr_broker::cli::commands::{rotate_agent_key, rotate_generator_key};
use brokkr_broker::utils::pak;
use brokkr_models::models::audit_logs::{ACTION_PAK_ROTATED, AuditLogFilter};

#[tokio::test]
async fn test_rotate_agent_key_returns_usable_pak_and_audits() {
    let fixture = TestFixture::new();
    let agent =
        fixture.create_test_agent("rotate-cli-agent".to_string(), "test-cluster".to_string());

    let new_pak = rotate_agent_key(&fixture.settings, agent.id).expect("rotation should succeed");
    assert!(!new_pak.is_empty(), "rotation must return the new PAK");

    // The returned PAK must verify against the stored hash.
    let stored_hash = fixture
        .dal
        .agents()
        .get(agent.id)
        .expect("agent fetch")
        .expect("agent exists")
        .pak_hash;
    assert!(
        pak::verify_pak(new_pak.clone(), stored_hash).expect("verification should parse"),
        "returned PAK must match the stored hash"
    );

    // The rotation must be recorded synchronously with the CLI marker.
    let filter = AuditLogFilter {
        action: Some(ACTION_PAK_ROTATED.to_string()),
        resource_id: Some(agent.id),
        ..Default::default()
    };
    let logs = fixture
        .dal
        .audit_logs()
        .list(Some(&filter), Some(10), Some(0))
        .expect("audit query");
    assert!(
        logs.iter().any(|l| {
            l.details
                .as_ref()
                .and_then(|d| d.get("via"))
                .and_then(|v| v.as_str())
                == Some("cli")
        }),
        "CLI rotation must write a pak.rotated audit row with details.via=cli"
    );
}

#[tokio::test]
async fn test_rotate_generator_key_returns_usable_pak() {
    let fixture = TestFixture::new();
    let generator = fixture.create_test_generator(
        "rotate-cli-generator".to_string(),
        Some("rotation test".to_string()),
        "initial-hash".to_string(),
    );

    let new_pak =
        rotate_generator_key(&fixture.settings, generator.id).expect("rotation should succeed");
    assert!(!new_pak.is_empty(), "rotation must return the new PAK");

    let stored_hash = fixture
        .dal
        .generators()
        .get(generator.id)
        .expect("generator fetch")
        .expect("generator exists")
        .pak_hash
        .expect("generator has a pak hash after rotation");
    assert!(
        pak::verify_pak(new_pak, stored_hash).expect("verification should parse"),
        "returned PAK must match the stored hash"
    );
}
