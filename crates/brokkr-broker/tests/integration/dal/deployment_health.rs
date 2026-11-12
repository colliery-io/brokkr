/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::fixtures::TestFixture;
use brokkr_models::models::deployment_health::NewDeploymentHealth;
use chrono::Utc;

#[test]
fn test_upsert_deployment_health() {
    let fixture = TestFixture::new();

    // Create an agent and deployment object
    let agent = fixture.create_test_agent("Health Agent".to_string(), "Health Cluster".to_string());
    let stack =
        fixture.create_test_stack("Health Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Create a new health record
    let new_health = NewDeploymentHealth::new(
        agent.id,
        deployment_object.id,
        "healthy".to_string(),
        Some(r#"{"pods_ready": 3, "pods_total": 3}"#.to_string()),
        Utc::now(),
    )
    .expect("Failed to create NewDeploymentHealth");

    let created_health = fixture
        .dal
        .deployment_health()
        .upsert(&new_health)
        .expect("Failed to upsert deployment health");

    assert_eq!(created_health.agent_id, agent.id);
    assert_eq!(created_health.deployment_object_id, deployment_object.id);
    assert_eq!(created_health.status, "healthy");
    assert!(created_health.summary.is_some());

    // Upsert with updated status
    let updated_health = NewDeploymentHealth::new(
        agent.id,
        deployment_object.id,
        "degraded".to_string(),
        Some(
            r#"{"pods_ready": 2, "pods_total": 3, "conditions": ["ImagePullBackOff"]}"#.to_string(),
        ),
        Utc::now(),
    )
    .expect("Failed to create updated NewDeploymentHealth");

    let upserted_health = fixture
        .dal
        .deployment_health()
        .upsert(&updated_health)
        .expect("Failed to upsert updated deployment health");

    assert_eq!(upserted_health.id, created_health.id);
    assert_eq!(upserted_health.status, "degraded");
}

#[test]
fn test_upsert_batch_deployment_health() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Batch Agent".to_string(), "Batch Cluster".to_string());
    let stack =
        fixture.create_test_stack("Batch Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object1 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test1".to_string(),
        false,
    );
    let deployment_object2 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test2".to_string(),
        false,
    );

    let health_records = vec![
        NewDeploymentHealth::new(
            agent.id,
            deployment_object1.id,
            "healthy".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
        NewDeploymentHealth::new(
            agent.id,
            deployment_object2.id,
            "degraded".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
    ];

    let count = fixture
        .dal
        .deployment_health()
        .upsert_batch(&health_records)
        .expect("Failed to batch upsert deployment health");

    assert_eq!(count, 2);

    // Verify records were created
    let health1 = fixture
        .dal
        .deployment_health()
        .get_by_agent_and_deployment(agent.id, deployment_object1.id)
        .expect("Failed to get health record")
        .expect("Health record not found");
    assert_eq!(health1.status, "healthy");

    let health2 = fixture
        .dal
        .deployment_health()
        .get_by_agent_and_deployment(agent.id, deployment_object2.id)
        .expect("Failed to get health record")
        .expect("Health record not found");
    assert_eq!(health2.status, "degraded");
}

#[test]
fn test_get_deployment_health_by_agent_and_deployment() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Get Agent".to_string(), "Get Cluster".to_string());
    let stack =
        fixture.create_test_stack("Get Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    // Should be None initially
    let health = fixture
        .dal
        .deployment_health()
        .get_by_agent_and_deployment(agent.id, deployment_object.id)
        .expect("Failed to query health");
    assert!(health.is_none());

    // Create a health record
    let new_health = NewDeploymentHealth::new(
        agent.id,
        deployment_object.id,
        "healthy".to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    fixture
        .dal
        .deployment_health()
        .upsert(&new_health)
        .expect("Failed to upsert health");

    // Now should find it
    let health = fixture
        .dal
        .deployment_health()
        .get_by_agent_and_deployment(agent.id, deployment_object.id)
        .expect("Failed to query health")
        .expect("Health record not found");

    assert_eq!(health.status, "healthy");
}

#[test]
fn test_list_deployment_health_by_agent() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("List Agent".to_string(), "List Cluster".to_string());
    let stack =
        fixture.create_test_stack("List Stack".to_string(), None, fixture.admin_generator.id);

    let deployment_object1 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test1".to_string(),
        false,
    );
    let deployment_object2 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test2".to_string(),
        false,
    );

    // Create health records
    let health_records = vec![
        NewDeploymentHealth::new(
            agent.id,
            deployment_object1.id,
            "healthy".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
        NewDeploymentHealth::new(
            agent.id,
            deployment_object2.id,
            "failing".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
    ];

    fixture
        .dal
        .deployment_health()
        .upsert_batch(&health_records)
        .expect("Failed to batch upsert");

    let health_list = fixture
        .dal
        .deployment_health()
        .list_by_agent(agent.id)
        .expect("Failed to list by agent");

    assert_eq!(health_list.len(), 2);
}

#[test]
fn test_list_deployment_health_by_stack() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Stack Agent".to_string(), "Stack Cluster".to_string());
    let stack =
        fixture.create_test_stack("Health Stack".to_string(), None, fixture.admin_generator.id);

    fixture.create_test_agent_target(agent.id, stack.id);

    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_health = NewDeploymentHealth::new(
        agent.id,
        deployment_object.id,
        "healthy".to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    fixture
        .dal
        .deployment_health()
        .upsert(&new_health)
        .expect("Failed to upsert");

    let health_list = fixture
        .dal
        .deployment_health()
        .list_by_stack(stack.id)
        .expect("Failed to list by stack");

    assert_eq!(health_list.len(), 1);
    assert_eq!(health_list[0].status, "healthy");
}

#[test]
fn test_list_deployment_health_by_status() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Status Agent".to_string(), "Status Cluster".to_string());
    let stack =
        fixture.create_test_stack("Status Stack".to_string(), None, fixture.admin_generator.id);

    let deployment_object1 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test1".to_string(),
        false,
    );
    let deployment_object2 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test2".to_string(),
        false,
    );

    let health_records = vec![
        NewDeploymentHealth::new(
            agent.id,
            deployment_object1.id,
            "degraded".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
        NewDeploymentHealth::new(
            agent.id,
            deployment_object2.id,
            "healthy".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
    ];

    fixture
        .dal
        .deployment_health()
        .upsert_batch(&health_records)
        .expect("Failed to batch upsert");

    let degraded_list = fixture
        .dal
        .deployment_health()
        .list_by_status("degraded")
        .expect("Failed to list by status");

    assert_eq!(degraded_list.len(), 1);
    assert_eq!(degraded_list[0].deployment_object_id, deployment_object1.id);
}

#[test]
fn test_delete_deployment_health() {
    let fixture = TestFixture::new();

    let agent = fixture.create_test_agent("Delete Agent".to_string(), "Delete Cluster".to_string());
    let stack =
        fixture.create_test_stack("Delete Stack".to_string(), None, fixture.admin_generator.id);
    let deployment_object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test".to_string(),
        false,
    );

    let new_health = NewDeploymentHealth::new(
        agent.id,
        deployment_object.id,
        "healthy".to_string(),
        None,
        Utc::now(),
    )
    .unwrap();

    fixture
        .dal
        .deployment_health()
        .upsert(&new_health)
        .expect("Failed to upsert");

    // Delete the health record
    let deleted = fixture
        .dal
        .deployment_health()
        .delete_by_agent_and_deployment(agent.id, deployment_object.id)
        .expect("Failed to delete");

    assert_eq!(deleted, 1);

    // Verify deletion
    let health = fixture
        .dal
        .deployment_health()
        .get_by_agent_and_deployment(agent.id, deployment_object.id)
        .expect("Failed to query");
    assert!(health.is_none());
}

#[test]
fn test_delete_deployment_health_by_agent() {
    let fixture = TestFixture::new();

    let agent =
        fixture.create_test_agent("Delete All Agent".to_string(), "Delete Cluster".to_string());
    let stack = fixture.create_test_stack(
        "Delete All Stack".to_string(),
        None,
        fixture.admin_generator.id,
    );

    let deployment_object1 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test1".to_string(),
        false,
    );
    let deployment_object2 = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test2".to_string(),
        false,
    );

    let health_records = vec![
        NewDeploymentHealth::new(
            agent.id,
            deployment_object1.id,
            "healthy".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
        NewDeploymentHealth::new(
            agent.id,
            deployment_object2.id,
            "healthy".to_string(),
            None,
            Utc::now(),
        )
        .unwrap(),
    ];

    fixture
        .dal
        .deployment_health()
        .upsert_batch(&health_records)
        .expect("Failed to batch upsert");

    let deleted = fixture
        .dal
        .deployment_health()
        .delete_by_agent(agent.id)
        .expect("Failed to delete by agent");

    assert_eq!(deleted, 2);

    let remaining = fixture
        .dal
        .deployment_health()
        .list_by_agent(agent.id)
        .expect("Failed to list");
    assert!(remaining.is_empty());
}

// ---------------------------------------------------------------------------
// BROKKR-T-0393: a health row counts only while the agent serves the object.
// ---------------------------------------------------------------------------

/// Reports `status` from `agent` for `object`.
fn report(fixture: &TestFixture, agent: uuid::Uuid, object: uuid::Uuid, status: &str) {
    let h = NewDeploymentHealth::new(agent, object, status.to_string(), None, Utc::now()).unwrap();
    fixture
        .dal
        .deployment_health()
        .upsert(&h)
        .expect("Failed to upsert");
}

/// `(failing, degraded)` for one agent, from the grouped fleet query.
fn fleet_counts(fixture: &TestFixture, agent: uuid::Uuid) -> (i64, i64) {
    let mut failing = 0;
    let mut degraded = 0;
    for (agent_id, status, count) in fixture
        .dal
        .deployment_health()
        .status_counts_by_agent()
        .expect("status_counts_by_agent failed")
    {
        if agent_id != agent {
            continue;
        }
        match status.as_str() {
            "failing" => failing += count,
            "degraded" => degraded += count,
            _ => {}
        }
    }
    (failing, degraded)
}

#[test]
fn test_fleet_health_counts_drop_a_deleted_stack() {
    let fixture = TestFixture::new();
    let gen_id = fixture.admin_generator.id;
    let agent = fixture.create_test_agent("t0393-del".to_string(), "c".to_string());
    let s_fail = fixture.create_test_stack("t0393-del-fail".to_string(), None, gen_id);
    let s_degr = fixture.create_test_stack("t0393-del-degr".to_string(), None, gen_id);
    fixture.create_test_agent_target(agent.id, s_fail.id);
    fixture.create_test_agent_target(agent.id, s_degr.id);
    let o_fail = fixture.create_test_deployment_object(s_fail.id, "f".to_string(), false);
    let o_degr = fixture.create_test_deployment_object(s_degr.id, "d".to_string(), false);
    report(&fixture, agent.id, o_fail.id, "failing");
    report(&fixture, agent.id, o_degr.id, "degraded");
    assert_eq!(fleet_counts(&fixture, agent.id), (1, 1));

    fixture.dal.stacks().soft_delete(s_fail.id).unwrap();
    fixture.dal.stacks().soft_delete(s_degr.id).unwrap();

    assert_eq!(
        fleet_counts(&fixture, agent.id),
        (0, 0),
        "the health rows of deleted stacks must not count"
    );
    assert!(
        fixture
            .dal
            .deployment_health()
            .list_by_stack(s_fail.id)
            .unwrap()
            .is_empty(),
        "a deleted stack has no health"
    );
}

#[test]
fn test_health_drops_a_replaced_object() {
    let fixture = TestFixture::new();
    let gen_id = fixture.admin_generator.id;
    let agent = fixture.create_test_agent("t0393-repl".to_string(), "c".to_string());
    let stack = fixture.create_test_stack("t0393-repl".to_string(), None, gen_id);
    fixture.create_test_agent_target(agent.id, stack.id);
    let v1 = fixture.create_test_deployment_object(stack.id, "v1".to_string(), false);
    report(&fixture, agent.id, v1.id, "failing");
    assert_eq!(fleet_counts(&fixture, agent.id), (1, 0));

    // v2 replaces v1. The agent reports only on v2 from now on.
    let v2 = fixture.create_test_deployment_object(stack.id, "v2".to_string(), false);
    assert_eq!(
        fleet_counts(&fixture, agent.id),
        (0, 0),
        "the row of a replaced object must not count"
    );
    assert!(
        fixture
            .dal
            .deployment_health()
            .list_by_stack(stack.id)
            .unwrap()
            .is_empty(),
        "no report on the latest object yet"
    );

    report(&fixture, agent.id, v2.id, "healthy");
    let rows = fixture
        .dal
        .deployment_health()
        .list_by_stack(stack.id)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].deployment_object_id, v2.id);
    assert_eq!(rows[0].status, "healthy");
}

#[test]
fn test_health_drops_an_agent_that_stopped_serving() {
    let fixture = TestFixture::new();
    let gen_id = fixture.admin_generator.id;
    let agent = fixture.create_test_agent("t0393-untarget".to_string(), "c".to_string());
    let stack = fixture.create_test_stack("t0393-untarget".to_string(), None, gen_id);
    fixture.create_test_agent_target(agent.id, stack.id);
    let object = fixture.create_test_deployment_object(stack.id, "v1".to_string(), false);
    report(&fixture, agent.id, object.id, "degraded");
    assert_eq!(fleet_counts(&fixture, agent.id), (0, 1));

    fixture
        .dal
        .agent_targets()
        .delete_by_agent_and_stack(agent.id, stack.id)
        .unwrap();

    assert_eq!(
        fleet_counts(&fixture, agent.id),
        (0, 0),
        "an agent that no longer serves the stack must not count"
    );
    assert!(
        fixture
            .dal
            .deployment_health()
            .list_by_stack(stack.id)
            .unwrap()
            .is_empty()
    );
}
