/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Registration-consent semantics for label/annotation stack matching
//! (BROKKR-T-0287).
//!
//! Decision (2026-07-27): registration is the consent boundary for ALL
//! stack-to-agent association paths. A generator says "these are the labels I
//! push to"; an agent's registrations say "these are the generators I accept
//! stacks from". Label and annotation matches must therefore only associate an
//! agent with stacks whose owning generator the agent is registered with —
//! matching selects *within* consented generators, it does not create
//! responsibility across them. Explicit targets already enforce this at
//! creation time (403 `agent_not_registered`).

use crate::fixtures::TestFixture;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

/// Fetches the agent's incremental target state as admin and returns the
/// deployment-object ids it contains.
async fn target_state_object_ids(fixture: &TestFixture, agent_id: Uuid) -> Vec<Uuid> {
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/agents/{agent_id}/target-state"))
                .header("Authorization", format!("Bearer {}", fixture.admin_pak))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    json.as_array()
        .expect("target state must be an array")
        .iter()
        .map(|o| o["id"].as_str().unwrap().parse().unwrap())
        .collect()
}

/// Positive path: a label match delivers a stack owned by a generator the
/// agent IS registered with. Holds both before and after the consent fix.
#[tokio::test]
async fn test_label_match_delivers_stack_from_registered_generator() {
    let fixture = TestFixture::new();

    let (gen_a, _) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("consent-agent".to_string(), "cluster-1".to_string());
    fixture.register_agent_with_generator(agent.id, gen_a.id);

    // Admin wires the agent side; tenant A wires its own stack side.
    fixture.create_test_agent_label(agent.id, "env=consent-prod".to_string());
    let stack_a = fixture.create_test_stack("tenant-a-stack".to_string(), None, gen_a.id);
    fixture.create_test_stack_label(stack_a.id, "env=consent-prod".to_string());
    let obj_a = fixture.create_test_deployment_object(
        stack_a.id,
        "kind: ConfigMap\nmetadata:\n  name: a".to_string(),
        false,
    );

    let ids = target_state_object_ids(&fixture, agent.id).await;
    assert!(
        ids.contains(&obj_a.id),
        "label-matched stack from a REGISTERED generator must be in the agent's target state"
    );
}

/// Consent boundary, label leg: a stack owned by a generator the agent is NOT
/// registered with must not reach the agent, even when labels collide.
///
/// Regression guard for BROKKR-T-0287: before the fix, the matching union
/// applied no registration filter to label matches and tenant B's object
/// appeared in the agent's target state (failure reproduced live 2026-07-27).
#[tokio::test]
async fn test_label_match_requires_registration_consent() {
    let fixture = TestFixture::new();

    let (gen_a, _) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (gen_b, _) =
        fixture.create_test_generator_with_pak("tenant-b".to_string(), None);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("consent-agent".to_string(), "cluster-1".to_string());
    fixture.register_agent_with_generator(agent.id, gen_a.id);

    fixture.create_test_agent_label(agent.id, "env=consent-prod".to_string());

    // Tenant B (never consented to by the agent) picks the colliding label.
    let stack_b = fixture.create_test_stack("tenant-b-stack".to_string(), None, gen_b.id);
    fixture.create_test_stack_label(stack_b.id, "env=consent-prod".to_string());
    let obj_b = fixture.create_test_deployment_object(
        stack_b.id,
        "kind: ConfigMap\nmetadata:\n  name: b".to_string(),
        false,
    );

    let ids = target_state_object_ids(&fixture, agent.id).await;
    assert!(
        !ids.contains(&obj_b.id),
        "label-matched stack from an UNREGISTERED generator must NOT be in the agent's \
         target state: registration is the consent boundary (BROKKR-T-0287)"
    );
}

/// An agent with no registrations at all receives nothing from label matching.
///
/// This is the interaction that makes BROKKR-T-0289 load-bearing: before that
/// fix, `brokkr-broker create agent` produced exactly this state (no
/// registrations, not even with the system generator), so such an agent goes
/// silent under the consent rule rather than silently over-receiving.
#[tokio::test]
async fn test_unregistered_agent_receives_nothing_from_label_match() {
    let fixture = TestFixture::new();

    let (gen_a, _) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (agent, _) = fixture
        .create_bare_agent_with_pak("no-registrations".to_string(), "cluster-1".to_string());

    fixture.create_test_agent_label(agent.id, "env=consent-prod".to_string());
    let stack_a = fixture.create_test_stack("tenant-a-stack".to_string(), None, gen_a.id);
    fixture.create_test_stack_label(stack_a.id, "env=consent-prod".to_string());
    fixture.create_test_deployment_object(
        stack_a.id,
        "kind: ConfigMap\nmetadata:\n  name: a".to_string(),
        false,
    );

    let ids = target_state_object_ids(&fixture, agent.id).await;
    assert!(
        ids.is_empty(),
        "an agent with zero registrations must receive nothing from label matching"
    );
}

/// Consent boundary, annotation leg: same rule for (key, value) annotation
/// matches. Regression guard for BROKKR-T-0287 (failure reproduced live
/// 2026-07-27 before the fix).
#[tokio::test]
async fn test_annotation_match_requires_registration_consent() {
    let fixture = TestFixture::new();

    let (gen_a, _) =
        fixture.create_test_generator_with_pak("tenant-a".to_string(), None);
    let (gen_b, _) =
        fixture.create_test_generator_with_pak("tenant-b".to_string(), None);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("consent-agent".to_string(), "cluster-1".to_string());
    fixture.register_agent_with_generator(agent.id, gen_a.id);

    fixture.create_test_agent_annotation(
        agent.id,
        "team".to_string(),
        "consent-platform".to_string(),
    );

    let stack_b = fixture.create_test_stack("tenant-b-annot-stack".to_string(), None, gen_b.id);
    fixture.create_test_stack_annotation(stack_b.id, "team", "consent-platform");
    let obj_b = fixture.create_test_deployment_object(
        stack_b.id,
        "kind: ConfigMap\nmetadata:\n  name: b-annot".to_string(),
        false,
    );

    let ids = target_state_object_ids(&fixture, agent.id).await;
    assert!(
        !ids.contains(&obj_b.id),
        "annotation-matched stack from an UNREGISTERED generator must NOT be in the agent's \
         target state: registration is the consent boundary (BROKKR-T-0287)"
    );
}
