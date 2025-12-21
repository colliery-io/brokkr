/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
use chrono::{Duration, Utc};

#[test]
fn test_create_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Diag Agent".to_string(), "Diag Cluster".to_string());
    let stack = fixture.create_test_stack("Diag Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(
        agent.id,
        deployment_object.id,
        Some("admin@example.com".to_string()),
        Some(60), // 60 minute retention
    )
    .expect("Failed to create NewDiagnosticRequest");

    let created = fixture
        .dal
        .diagnostic_requests()
        .create(&new_request)
        .expect("Failed to create diagnostic request");

    assert_eq!(created.agent_id, agent.id);
    assert_eq!(created.deployment_object_id, deployment_object.id);
    assert_eq!(created.status, "pending");
    assert_eq!(created.requested_by, Some("admin@example.com".to_string()));
    assert!(created.expires_at > Utc::now());
}

#[test]
fn test_get_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Get Diag Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Get Diag Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();

    let created = fixture
        .dal
        .diagnostic_requests()
        .create(&new_request)
        .expect("Failed to create");

    let retrieved = fixture
        .dal
        .diagnostic_requests()
        .get(created.id)
        .expect("Failed to get")
        .expect("Request not found");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.status, "pending");
}

#[test]
fn test_get_pending_for_agent() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Pending Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Pending Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create multiple requests
    for _ in 0..3 {
        let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
        fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    }

    let pending = fixture
        .dal
        .diagnostic_requests()
        .get_pending_for_agent(agent.id)
        .expect("Failed to get pending");

    assert_eq!(pending.len(), 3);
    for request in pending {
        assert_eq!(request.status, "pending");
        assert_eq!(request.agent_id, agent.id);
    }
}

#[test]
fn test_claim_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Claim Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Claim Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let created = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let claimed = fixture
        .dal
        .diagnostic_requests()
        .claim(created.id)
        .expect("Failed to claim");

    assert_eq!(claimed.status, "claimed");
    assert!(claimed.claimed_at.is_some());

    // Should no longer appear in pending
    let pending = fixture
        .dal
        .diagnostic_requests()
        .get_pending_for_agent(agent.id)
        .expect("Failed to get pending");

    assert!(pending.is_empty());
}

#[test]
fn test_complete_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Complete Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Complete Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let created = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    fixture.dal.diagnostic_requests().claim(created.id).unwrap();

    let completed = fixture
        .dal
        .diagnostic_requests()
        .complete(created.id)
        .expect("Failed to complete");

    assert_eq!(completed.status, "completed");
    assert!(completed.completed_at.is_some());
}

#[test]
fn test_fail_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Fail Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Fail Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let created = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    fixture.dal.diagnostic_requests().claim(created.id).unwrap();

    let failed = fixture
        .dal
        .diagnostic_requests()
        .fail(created.id)
        .expect("Failed to fail request");

    assert_eq!(failed.status, "failed");
    assert!(failed.completed_at.is_some());
}

#[test]
fn test_list_by_deployment_object() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("List Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("List Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create multiple requests
    for _ in 0..3 {
        let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
        fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    }

    let list = fixture
        .dal
        .diagnostic_requests()
        .list_by_deployment_object(deployment_object.id)
        .expect("Failed to list");

    assert_eq!(list.len(), 3);
}

#[test]
fn test_expire_old_requests() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Expire Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Expire Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create a request with very short retention (already expired)
    // We need to manually insert with a past expires_at
    let new_request = NewDiagnosticRequest {
        agent_id: agent.id,
        deployment_object_id: deployment_object.id,
        status: "pending".to_string(),
        requested_by: None,
        expires_at: Utc::now() - Duration::minutes(5), // Already expired
    };

    fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    // Run expire
    let expired_count = fixture
        .dal
        .diagnostic_requests()
        .expire_old_requests()
        .expect("Failed to expire");

    assert_eq!(expired_count, 1);

    // Verify status changed
    let pending = fixture
        .dal
        .diagnostic_requests()
        .get_pending_for_agent(agent.id)
        .expect("Failed to get pending");

    assert!(pending.is_empty());
}

#[test]
fn test_cleanup_old_requests() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Cleanup Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Cleanup Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create and complete a request
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let created = fixture.dal.diagnostic_requests().create(&new_request).unwrap();
    fixture.dal.diagnostic_requests().complete(created.id).unwrap();

    // Cleanup with 0 hours max age should delete it
    let deleted = fixture
        .dal
        .diagnostic_requests()
        .cleanup_old_requests(0)
        .expect("Failed to cleanup");

    assert_eq!(deleted, 1);

    // Verify deleted
    let retrieved = fixture
        .dal
        .diagnostic_requests()
        .get(created.id)
        .expect("Failed to get");

    assert!(retrieved.is_none());
}

#[test]
fn test_delete_diagnostic_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Delete Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Delete Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let created = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let deleted = fixture
        .dal
        .diagnostic_requests()
        .delete(created.id)
        .expect("Failed to delete");

    assert_eq!(deleted, 1);

    let retrieved = fixture
        .dal
        .diagnostic_requests()
        .get(created.id)
        .expect("Failed to get");

    assert!(retrieved.is_none());
}
