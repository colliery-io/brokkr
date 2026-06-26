/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for the generator registration model (BROKKR-I-0030).

use crate::fixtures::TestFixture;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use brokkr_models::models::agent_targets::NewAgentTarget;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// POST /agents — system generator auto-registration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_agent_via_api_auto_registers_with_system_generator() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let body = json!({ "name": "AutoReg Agent", "cluster_name": "cluster-1" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agents")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    let agent_id: Uuid = json["agent"]["id"].as_str().unwrap().parse().unwrap();

    let sys_gen_id = fixture
        .dal
        .generators()
        .get_system_generator_id()
        .expect("db error")
        .expect("system generator must exist");

    let registered = fixture
        .dal
        .agent_generator_registrations()
        .is_registered(agent_id, sys_gen_id)
        .expect("db error");
    assert!(registered, "agent must be auto-registered with system generator");
}

// ---------------------------------------------------------------------------
// POST /agents/:id/targets — registration enforcement
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_add_target_unregistered_agent_returns_403() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    // Create a fresh generator whose stacks the agent won't be registered with.
    let (custom_gen, _) =
        fixture.create_test_generator_with_pak("Custom Generator".to_string(), None);
    let stack = fixture.create_test_stack("Custom Stack".to_string(), None, custom_gen.id);

    // Agent created WITHOUT registration to custom_gen (bare = no auto-register).
    let (agent, _) = fixture.create_bare_agent_with_pak("Bare Agent".to_string(), "c1".to_string());

    let target = NewAgentTarget::new(agent.id, stack.id).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/targets", agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(serde_json::to_string(&target).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"].as_str().unwrap_or(""), "agent_not_registered");
}

#[tokio::test]
async fn test_add_target_registered_agent_returns_201() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (custom_gen, _) =
        fixture.create_test_generator_with_pak("Custom Generator".to_string(), None);
    let stack = fixture.create_test_stack("Custom Stack".to_string(), None, custom_gen.id);

    let (agent, _) =
        fixture.create_bare_agent_with_pak("Registered Agent".to_string(), "c1".to_string());
    // Explicitly register with the custom generator.
    fixture.register_agent_with_generator(agent.id, custom_gen.id);

    let target = NewAgentTarget::new(agent.id, stack.id).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/targets", agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(serde_json::to_string(&target).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_admin_cannot_bypass_registration_check() {
    // Admin callers are NOT exempt from the registration gate (T-0246 spec).
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Gen A".to_string(), None);
    let stack = fixture.create_test_stack("Stack A".to_string(), None, generator.id);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("Unregistered".to_string(), "c1".to_string());

    let target = NewAgentTarget::new(agent.id, stack.id).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/agents/{}/targets", agent.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(serde_json::to_string(&target).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"].as_str().unwrap_or(""), "agent_not_registered");
}

// ---------------------------------------------------------------------------
// Two generators, two agents — scope isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_registration_scope_isolation() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (gen_a, _) = fixture.create_test_generator_with_pak("Gen A".to_string(), None);
    let (gen_b, _) = fixture.create_test_generator_with_pak("Gen B".to_string(), None);
    let stack_a = fixture.create_test_stack("Stack A".to_string(), None, gen_a.id);
    let stack_b = fixture.create_test_stack("Stack B".to_string(), None, gen_b.id);

    // Agent registered only with Gen A.
    let (agent, _) = fixture.create_bare_agent_with_pak("Agent A".to_string(), "c1".to_string());
    fixture.register_agent_with_generator(agent.id, gen_a.id);

    let make_target_req = |stack_id: uuid::Uuid| {
        let target = NewAgentTarget::new(agent.id, stack_id).unwrap();
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/agents/{}/targets", agent.id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", admin_pak))
            .body(Body::from(serde_json::to_string(&target).unwrap()))
            .unwrap()
    };

    // Targeting Gen B's stack → 403.
    let response_b = app
        .clone()
        .oneshot(make_target_req(stack_b.id))
        .await
        .unwrap();
    let bytes = to_bytes(response_b.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json["code"].as_str().unwrap_or(""),
        "agent_not_registered",
        "targeting Gen B stack should be forbidden"
    );

    // Targeting Gen A's stack → 201.
    let response_a = app.oneshot(make_target_req(stack_a.id)).await.unwrap();
    assert_eq!(response_a.status(), StatusCode::CREATED);
}

// ---------------------------------------------------------------------------
// DELETE /generators/:id/register — cascade
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_deregister_cascades_agent_targets() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("Deregister Gen".to_string(), None);
    let stack = fixture.create_test_stack("Deregister Stack".to_string(), None, generator.id);
    let (agent, _) = fixture.create_bare_agent_with_pak("Cascade Agent".to_string(), "c1".to_string());

    // Register and add target via DAL to skip the API enforcement.
    fixture.register_agent_with_generator(agent.id, generator.id);
    let _target = fixture
        .dal
        .agent_targets()
        .create(
            &brokkr_models::models::agent_targets::NewAgentTarget::new(agent.id, stack.id)
                .unwrap(),
        )
        .expect("Failed to create agent target");

    // Confirm target exists.
    let targets_before = fixture
        .dal
        .agent_targets()
        .list_for_agent(agent.id)
        .expect("db error");
    assert!(
        targets_before.iter().any(|t| t.stack_id == stack.id),
        "target must exist before deregistration"
    );

    // Deregister via API.
    let body = json!({ "agent_id": agent.id });
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/generators/{}/register", generator.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Target rows for this generator's stacks must be gone.
    let targets_after = fixture
        .dal
        .agent_targets()
        .list_for_agent(agent.id)
        .expect("db error");
    assert!(
        !targets_after.iter().any(|t| t.stack_id == stack.id),
        "agent_target must be removed after deregistration"
    );

    // Registration row must also be gone.
    let still_registered = fixture
        .dal
        .agent_generator_registrations()
        .is_registered(agent.id, generator.id)
        .expect("db error");
    assert!(!still_registered, "registration must be removed");
}

// ---------------------------------------------------------------------------
// GET endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_agent_registrations() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("List Gen".to_string(), None);
    let (agent, _) = fixture.create_bare_agent_with_pak("List Agent".to_string(), "c1".to_string());
    fixture.register_agent_with_generator(agent.id, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{}/registrations", agent.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let regs: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(
        regs.iter()
            .any(|r| r["generator_id"].as_str().unwrap_or("") == generator.id.to_string()),
        "registered generator must appear in list"
    );
}

#[tokio::test]
async fn test_list_generator_registered_agents() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let admin_pak = fixture.admin_pak.clone();

    let (generator, _) = fixture.create_test_generator_with_pak("GenReg Gen".to_string(), None);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("GenReg Agent".to_string(), "c1".to_string());
    fixture.register_agent_with_generator(agent.id, generator.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/generators/{}/registered-agents", generator.id))
                .header("Authorization", format!("Bearer {}", admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let regs: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(
        regs.iter()
            .any(|r| r["agent_id"].as_str().unwrap_or("") == agent.id.to_string()),
        "registered agent must appear in generator's agent list"
    );
}
