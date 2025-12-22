/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::diagnostic_requests::NewDiagnosticRequest;
use brokkr_models::models::diagnostic_results::NewDiagnosticResult;
use chrono::Utc;

#[test]
fn test_create_diagnostic_result() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Result Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Result Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create a diagnostic request first
    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    // Create a diagnostic result
    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[{"name": "pod-1", "phase": "Running"}]"#.to_string(),
        r#"[{"type": "Normal", "reason": "Started"}]"#.to_string(),
        Some(r#"{"pod-1/container": ["log line 1", "log line 2"]}"#.to_string()),
        Utc::now(),
    )
    .expect("Failed to create NewDiagnosticResult");

    let created = fixture
        .dal
        .diagnostic_results()
        .create(&new_result)
        .expect("Failed to create diagnostic result");

    assert_eq!(created.request_id, request.id);
    assert!(created.pod_statuses.contains("pod-1"));
    assert!(created.events.contains("Normal"));
    assert!(created.log_tails.is_some());
}

#[test]
fn test_get_diagnostic_result() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Get Result Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Get Result Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    let created = fixture.dal.diagnostic_results().create(&new_result).unwrap();

    let retrieved = fixture
        .dal
        .diagnostic_results()
        .get(created.id)
        .expect("Failed to get")
        .expect("Result not found");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.request_id, request.id);
}

#[test]
fn test_get_diagnostic_result_by_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("By Request Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("By Request Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    // Should be None initially
    let result = fixture
        .dal
        .diagnostic_results()
        .get_by_request(request.id)
        .expect("Failed to get by request");
    assert!(result.is_none());

    // Create a result
    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    fixture.dal.diagnostic_results().create(&new_result).unwrap();

    // Now should find it
    let result = fixture
        .dal
        .diagnostic_results()
        .get_by_request(request.id)
        .expect("Failed to get by request")
        .expect("Result not found");

    assert_eq!(result.request_id, request.id);
}

#[test]
fn test_delete_diagnostic_result() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Delete Result Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Delete Result Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    let created = fixture.dal.diagnostic_results().create(&new_result).unwrap();

    let deleted = fixture
        .dal
        .diagnostic_results()
        .delete(created.id)
        .expect("Failed to delete");

    assert_eq!(deleted, 1);

    let retrieved = fixture
        .dal
        .diagnostic_results()
        .get(created.id)
        .expect("Failed to get");
    assert!(retrieved.is_none());
}

#[test]
fn test_delete_diagnostic_result_by_request() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Del By Req Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Del By Req Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    fixture.dal.diagnostic_results().create(&new_result).unwrap();

    let deleted = fixture
        .dal
        .diagnostic_results()
        .delete_by_request(request.id)
        .expect("Failed to delete by request");

    assert_eq!(deleted, 1);

    let retrieved = fixture
        .dal
        .diagnostic_results()
        .get_by_request(request.id)
        .expect("Failed to get");
    assert!(retrieved.is_none());
}

#[test]
fn test_cascade_delete_on_request_deletion() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Cascade Agent".to_string(), "Cluster".to_string());
    let stack = fixture.create_test_stack("Cascade Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_request = NewDiagnosticRequest::new(agent.id, deployment_object.id, None, Some(60)).unwrap();
    let request = fixture.dal.diagnostic_requests().create(&new_request).unwrap();

    let new_result = NewDiagnosticResult::new(
        request.id,
        r#"[]"#.to_string(),
        r#"[]"#.to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    let result = fixture.dal.diagnostic_results().create(&new_result).unwrap();

    // Delete the request - result should cascade delete
    fixture
        .dal
        .diagnostic_requests()
        .delete(request.id)
        .expect("Failed to delete request");

    // Result should be gone
    let retrieved = fixture
        .dal
        .diagnostic_results()
        .get(result.id)
        .expect("Failed to get");
    assert!(retrieved.is_none());
}
