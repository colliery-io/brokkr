/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for `?pak_id=` tenant scoping on fleet, stacks, and
//! agent-events (BROKKR-T-0270).

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

use brokkr_models::models::{agents::Agent, generator::Generator, stacks::Stack};

use crate::fixtures::TestFixture;

/// Two tenants, each with one registered agent, one stack, and one event.
struct TwoTenants {
    payments: Generator,
    payments_agent: Agent,
    payments_stack: Stack,
    ingest_agent: Agent,
    ingest_stack: Stack,
}

fn seed_two_tenants(fixture: &TestFixture) -> TwoTenants {
    let (payments, _) = fixture.create_test_generator_with_pak("team-payments".to_string(), None);
    let (ingest, _) = fixture.create_test_generator_with_pak("team-ingest".to_string(), None);

    let (payments_agent, _) =
        fixture.create_bare_agent_with_pak("payments-agent".to_string(), "cluster-a".to_string());
    let (ingest_agent, _) =
        fixture.create_bare_agent_with_pak("ingest-agent".to_string(), "cluster-b".to_string());
    fixture.register_agent_with_generator(payments_agent.id, payments.id);
    fixture.register_agent_with_generator(ingest_agent.id, ingest.id);

    let payments_stack = fixture.create_test_stack("payments-stack".to_string(), None, payments.id);
    let ingest_stack = fixture.create_test_stack("ingest-stack".to_string(), None, ingest.id);

    let payments_object = fixture.create_test_deployment_object(
        payments_stack.id,
        "kind: ConfigMap".to_string(),
        false,
    );
    let ingest_object =
        fixture.create_test_deployment_object(ingest_stack.id, "kind: Secret".to_string(), false);
    fixture.create_test_agent_event(
        &payments_agent,
        &payments_object,
        "Apply",
        "SUCCESS",
        Some("payments event"),
    );
    fixture.create_test_agent_event(
        &ingest_agent,
        &ingest_object,
        "Apply",
        "SUCCESS",
        Some("ingest event"),
    );

    TwoTenants {
        payments,
        payments_agent,
        payments_stack,
        ingest_agent,
        ingest_stack,
    }
}

async fn get_json(
    app: Router,
    token: &str,
    uri: &str,
) -> (StatusCode, Option<Vec<serde_json::Value>>) {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).ok())
}

fn ids(items: &[serde_json::Value], key: &str) -> Vec<String> {
    items
        .iter()
        .map(|v| v[key].as_str().unwrap_or_default().to_string())
        .collect()
}

#[tokio::test]
async fn test_fleet_scoped_by_pak_id() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let t = seed_two_tenants(&fixture);

    // Scoped: only the payments agent.
    let (status, body) = get_json(
        app.clone(),
        &fixture.admin_pak,
        &format!("/api/v1/fleet?pak_id={}", t.payments.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let agent_ids = ids(&body.unwrap(), "agent_id");
    assert!(agent_ids.contains(&t.payments_agent.id.to_string()));
    assert!(!agent_ids.contains(&t.ingest_agent.id.to_string()));

    // Unscoped: both agents.
    let (status, body) = get_json(app.clone(), &fixture.admin_pak, "/api/v1/fleet").await;
    assert_eq!(status, StatusCode::OK);
    let agent_ids = ids(&body.unwrap(), "agent_id");
    assert!(agent_ids.contains(&t.payments_agent.id.to_string()));
    assert!(agent_ids.contains(&t.ingest_agent.id.to_string()));

    // Unknown tenant: empty list, not an error.
    let (status, body) = get_json(
        app.clone(),
        &fixture.admin_pak,
        &format!("/api/v1/fleet?pak_id={}", Uuid::new_v4()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.unwrap().is_empty());

    // Malformed pak_id: 400 from query deserialization.
    let (status, _) = get_json(app, &fixture.admin_pak, "/api/v1/fleet?pak_id=not-a-uuid").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_stacks_scoped_by_pak_id() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let t = seed_two_tenants(&fixture);

    let (status, body) = get_json(
        app.clone(),
        &fixture.admin_pak,
        &format!("/api/v1/stacks?pak_id={}", t.payments.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let stack_ids = ids(&body.unwrap(), "id");
    assert!(stack_ids.contains(&t.payments_stack.id.to_string()));
    assert!(!stack_ids.contains(&t.ingest_stack.id.to_string()));

    let (status, body) = get_json(app, &fixture.admin_pak, "/api/v1/stacks").await;
    assert_eq!(status, StatusCode::OK);
    let stack_ids = ids(&body.unwrap(), "id");
    assert!(stack_ids.contains(&t.payments_stack.id.to_string()));
    assert!(stack_ids.contains(&t.ingest_stack.id.to_string()));
}

#[tokio::test]
async fn test_agent_events_scoped_by_pak_id() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let t = seed_two_tenants(&fixture);

    let (status, body) = get_json(
        app.clone(),
        &fixture.admin_pak,
        &format!("/api/v1/agent-events?pak_id={}", t.payments.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let events = body.unwrap();
    assert!(!events.is_empty());
    let event_agents = ids(&events, "agent_id");
    assert!(event_agents.contains(&t.payments_agent.id.to_string()));
    assert!(!event_agents.contains(&t.ingest_agent.id.to_string()));

    // Unscoped shows both tenants' events.
    let (status, body) = get_json(app, &fixture.admin_pak, "/api/v1/agent-events").await;
    assert_eq!(status, StatusCode::OK);
    let event_agents = ids(&body.unwrap(), "agent_id");
    assert!(event_agents.contains(&t.payments_agent.id.to_string()));
    assert!(event_agents.contains(&t.ingest_agent.id.to_string()));
}
