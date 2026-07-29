/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Tenant-scoped `GET /agents` for generator PAKs (BROKKR-T-0315).
//!
//! Tenants *are* generators (BROKKR-A-0009), so "which agents serve me?" is a
//! tenant-scoped read. Before this, it required an admin PAK — the whole fleet
//! plus full write access — which pushed operators toward handing tenants the
//! strongest credential in the system for a read that is inherently narrow.
//!
//! The listing is derived from registrations, so it reflects agent *consent*
//! (BROKKR-T-0287): an agent appears to a generator because it registered with
//! that generator, not because a label happened to match.
//!
//! Decision (Dylan, 2026-07-28): the system generator is **not** a tenant and
//! gets no tenant-scoped view. Every agent is auto-registered with `__system__`,
//! so scoping to it would quietly return the entire fleet through a non-admin
//! credential.

use crate::fixtures::TestFixture;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

/// Calls `GET /api/v1/agents` with `pak` and returns (status, body).
async fn list_agents(fixture: &TestFixture, pak: &str) -> (StatusCode, Value) {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/agents")
                .header("Authorization", format!("Bearer {pak}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Extracts agent ids from a successful listing body.
fn agent_ids(body: &Value) -> Vec<Uuid> {
    body.as_array()
        .expect("agent listing must be an array")
        .iter()
        .map(|a| a["id"].as_str().unwrap().parse().unwrap())
        .collect()
}

/// The core promise: a generator sees the agents registered with it.
#[tokio::test]
async fn test_generator_lists_its_registered_agents() {
    let fixture = TestFixture::new();

    let (generator, generator_pak) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (agent, _) =
        fixture.create_test_agent_with_pak("agent-a".to_string(), "cluster-a".to_string());
    fixture
        .dal
        .agent_generator_registrations()
        .create(agent.id, generator.id)
        .expect("failed to register agent with generator");

    let (status, body) = list_agents(&fixture, &generator_pak).await;

    assert_eq!(status, StatusCode::OK, "generator PAK must be accepted");
    assert_eq!(
        agent_ids(&body),
        vec![agent.id],
        "generator must see exactly the agent registered with it"
    );
}

/// The isolation property: one tenant's agents are invisible to another.
/// This is the test that would fail if the handler forgot to scope the query.
#[tokio::test]
async fn test_generator_does_not_see_another_generators_agents() {
    let fixture = TestFixture::new();

    let (gen_a, pak_a) = fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (gen_b, pak_b) = fixture.create_test_generator_with_pak("tenant-b".to_string(), None);

    let (agent_a, _) =
        fixture.create_test_agent_with_pak("agent-a".to_string(), "cluster-a".to_string());
    let (agent_b, _) =
        fixture.create_test_agent_with_pak("agent-b".to_string(), "cluster-b".to_string());

    let regs = fixture.dal.agent_generator_registrations();
    regs.create(agent_a.id, gen_a.id).expect("register a->A");
    regs.create(agent_b.id, gen_b.id).expect("register b->B");

    let (status_a, body_a) = list_agents(&fixture, &pak_a).await;
    assert_eq!(status_a, StatusCode::OK);
    let ids_a = agent_ids(&body_a);
    assert!(ids_a.contains(&agent_a.id), "A must see its own agent");
    assert!(
        !ids_a.contains(&agent_b.id),
        "A must NOT see B's agent — this is the tenant isolation boundary"
    );

    let (status_b, body_b) = list_agents(&fixture, &pak_b).await;
    assert_eq!(status_b, StatusCode::OK);
    let ids_b = agent_ids(&body_b);
    assert!(ids_b.contains(&agent_b.id), "B must see its own agent");
    assert!(!ids_b.contains(&agent_a.id), "B must NOT see A's agent");
}

/// The decision under test: a system generator is not a tenant.
///
/// This deliberately mints a PAK for the system generator **at the DAL layer**
/// rather than asserting that the API cannot mint one. Relying on "the system
/// generator has no PAK" would test `provision_system_generator`, not this
/// handler — and that invariant lives nowhere near the code depending on it.
/// Every agent is auto-registered with `__system__`, so without the guard this
/// call returns the entire fleet.
#[tokio::test]
async fn test_system_generator_pak_is_not_a_tenant() {
    let fixture = TestFixture::new();

    // Two agents from unrelated tenants; both are auto-registered with the
    // system generator, so an unguarded scoped query would return both.
    let (agent_a, _) =
        fixture.create_test_agent_with_pak("agent-a".to_string(), "cluster-a".to_string());
    let (agent_b, _) =
        fixture.create_test_agent_with_pak("agent-b".to_string(), "cluster-b".to_string());

    let system_id = fixture
        .dal
        .generators()
        .get_system_generator_id()
        .expect("failed to query system generator")
        .expect("system generator must be provisioned");

    // Force the situation the guard exists for.
    let (system_pak, system_hash) =
        brokkr_broker::utils::pak::create_pak().expect("failed to create PAK");
    fixture
        .dal
        .generators()
        .update_pak_hash(system_id, system_hash)
        .expect("failed to set system generator PAK");

    let (status, body) = list_agents(&fixture, &system_pak).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "the system generator must not receive a tenant-scoped listing"
    );
    assert_eq!(
        body["code"].as_str(),
        Some("system_generator_not_a_tenant"),
        "the refusal must be distinguishable from an ordinary auth failure, \
         body was: {body}"
    );

    // Guard against the failure this test exists to prevent: the fleet leaking
    // through a non-admin credential.
    let serialized = body.to_string();
    assert!(
        !serialized.contains(&agent_a.id.to_string())
            && !serialized.contains(&agent_b.id.to_string()),
        "no agent may appear in the refusal body"
    );
}

/// Admin behavior is unchanged, and `admin-generator` is correctly unaffected:
/// it is `is_system = false`, and an admin PAK resolves to admin before the
/// generator branch is ever reached.
#[tokio::test]
async fn test_admin_still_lists_every_agent() {
    let fixture = TestFixture::new();

    let (gen_a, _) = fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (agent_a, _) =
        fixture.create_test_agent_with_pak("agent-a".to_string(), "cluster-a".to_string());
    let (agent_b, _) =
        fixture.create_test_agent_with_pak("agent-b".to_string(), "cluster-b".to_string());
    fixture
        .dal
        .agent_generator_registrations()
        .create(agent_a.id, gen_a.id)
        .expect("register a->A");

    let (status, body) = list_agents(&fixture, &fixture.admin_pak).await;

    assert_eq!(status, StatusCode::OK);
    let ids = agent_ids(&body);
    assert!(
        ids.contains(&agent_a.id) && ids.contains(&agent_b.id),
        "admin must still see the whole fleet regardless of registrations"
    );
}

/// Soft-delete handling must match the admin `list()` path, or the two branches
/// of the same endpoint would disagree about which agents exist.
#[tokio::test]
async fn test_soft_deleted_agents_are_excluded() {
    let fixture = TestFixture::new();

    let (generator, generator_pak) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (live, _) =
        fixture.create_test_agent_with_pak("agent-live".to_string(), "cluster-a".to_string());
    let (deleted, _) =
        fixture.create_test_agent_with_pak("agent-deleted".to_string(), "cluster-a".to_string());

    let regs = fixture.dal.agent_generator_registrations();
    regs.create(live.id, generator.id).expect("register live");
    regs.create(deleted.id, generator.id)
        .expect("register deleted");

    fixture
        .dal
        .agents()
        .soft_delete(deleted.id)
        .expect("failed to soft-delete agent");

    let (status, body) = list_agents(&fixture, &generator_pak).await;

    assert_eq!(status, StatusCode::OK);
    let ids = agent_ids(&body);
    assert!(ids.contains(&live.id), "live agent must be listed");
    assert!(
        !ids.contains(&deleted.id),
        "soft-deleted agent must be excluded, matching the admin list() path"
    );
}

/// An agent PAK is neither admin nor generator: it still gets 403, and the code
/// says which of the two it is missing.
#[tokio::test]
async fn test_agent_pak_cannot_list_agents() {
    let fixture = TestFixture::new();

    let (_, agent_pak) =
        fixture.create_test_agent_with_pak("agent-a".to_string(), "cluster-a".to_string());

    let (status, body) = list_agents(&fixture, &agent_pak).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        body["code"].as_str(),
        Some("generator_required"),
        "body was: {body}"
    );
}

/// A generator with no registered agents gets an empty list, not an error and
/// not the fleet.
#[tokio::test]
async fn test_generator_with_no_agents_gets_empty_list() {
    let fixture = TestFixture::new();

    let (_, generator_pak) =
        fixture.create_test_generator_with_pak("tenant-empty".to_string(), None);
    let _ =
        fixture.create_test_agent_with_pak("agent-elsewhere".to_string(), "cluster-a".to_string());

    let (status, body) = list_agents(&fixture, &generator_pak).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        agent_ids(&body).is_empty(),
        "a generator with no registrations must see nothing, got: {body}"
    );
}
