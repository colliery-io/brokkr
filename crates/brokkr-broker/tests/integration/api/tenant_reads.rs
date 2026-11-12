/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! The reads a tenant may make about its own deployments (BROKKR-T-0379,
//! BROKKR-T-0380): the health of a stack it owns, with each agent's report,
//! and the labels and annotations of the agents registered with it. Another
//! tenant gets 403 on each.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::fixtures::TestFixture;

async fn get_json(app: Router, token: &str, uri: &str) -> (StatusCode, serde_json::Value) {
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
    (
        status,
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn the_owner_reads_its_stack_health_with_each_agent_report() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (owner, owner_pak) = fixture.create_test_generator_with_pak("team-owner".to_string(), None);
    let (agent, _) =
        fixture.create_bare_agent_with_pak("owner-agent".to_string(), "cluster-a".to_string());
    fixture.register_agent_with_generator(agent.id, owner.id);
    let stack = fixture.create_test_stack("owner-stack".to_string(), None, owner.id);
    // Health counts only for an agent that serves the stack (BROKKR-T-0393).
    fixture.create_test_agent_target(agent.id, stack.id);
    let object = fixture.create_test_deployment_object(
        stack.id,
        "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: owner".to_string(),
        false,
    );
    let record = brokkr_models::models::deployment_health::NewDeploymentHealth::new(
        agent.id,
        object.id,
        "healthy".to_string(),
        Some(r#"{"pods_ready": 1, "pods_total": 1}"#.to_string()),
        chrono::Utc::now(),
    )
    .expect("health record");
    fixture
        .dal
        .deployment_health()
        .upsert(&record)
        .expect("health record stored");

    let (status, body) = get_json(
        app.clone(),
        &owner_pak,
        &format!("/api/v1/stacks/{}/health", stack.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["overall_status"], "healthy");
    let objects = body["deployment_objects"].as_array().expect("objects");
    assert_eq!(objects.len(), 1);
    let agents = objects[0]["agents"].as_array().expect("agents");
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["agent_id"], agent.id.to_string());
    assert_eq!(agents[0]["status"], "healthy");
}

#[tokio::test]
async fn another_tenant_cannot_read_the_stack_health() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (owner, _) = fixture.create_test_generator_with_pak("team-owner".to_string(), None);
    let (_, other_pak) = fixture.create_test_generator_with_pak("team-other".to_string(), None);
    let stack = fixture.create_test_stack("owner-stack".to_string(), None, owner.id);

    let (status, body) = get_json(
        app.clone(),
        &other_pak,
        &format!("/api/v1/stacks/{}/health", stack.id),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["code"], "stack_not_owned");
}

#[tokio::test]
async fn a_tenant_reads_the_labels_of_its_registered_agents_only() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());
    let (tenant, tenant_pak) =
        fixture.create_test_generator_with_pak("team-tenant".to_string(), None);
    let (mine, _) = fixture.create_bare_agent_with_pak("mine".to_string(), "cluster-a".to_string());
    let (theirs, _) =
        fixture.create_bare_agent_with_pak("theirs".to_string(), "cluster-b".to_string());
    fixture.register_agent_with_generator(mine.id, tenant.id);
    fixture.create_test_agent_label(mine.id, "env:prod".to_string());
    fixture.create_test_agent_label(theirs.id, "env:prod".to_string());
    fixture.create_test_agent_annotation(mine.id, "team".to_string(), "tenant".to_string());

    let (status, body) = get_json(
        app.clone(),
        &tenant_pak,
        &format!("/api/v1/agents/{}/labels", mine.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body[0]["label"], "env:prod");

    let (status, body) = get_json(
        app.clone(),
        &tenant_pak,
        &format!("/api/v1/agents/{}/annotations", mine.id),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body[0]["key"], "team");

    let (status, body) = get_json(
        app.clone(),
        &tenant_pak,
        &format!("/api/v1/agents/{}/labels", theirs.id),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, _) = get_json(
        app.clone(),
        &tenant_pak,
        &format!("/api/v1/agents/{}/annotations", theirs.id),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
