/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for the internal `/internal/ws/agent` upgrade endpoint
//! (WS-02, BROKKR-T-0157).
//!
//! Two layers of coverage:
//! - `tower::ServiceExt::oneshot` for the auth-gating *before* upgrade
//!   (unauthenticated → 401, OpenAPI exclusion).
//! - A real `TcpListener` + `tokio-tungstenite` round-trip for the actual
//!   WebSocket upgrade (admin/generator → 403, agent → 101 + frame exchange).
//!   In-process `oneshot` cannot complete a WS upgrade because hyper's
//!   `OnUpgrade` extension is only installed by the real HTTP/1.1 server
//!   path; this is why we bind a TCP listener for the upgrade tests.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    http::{header, HeaderValue},
    Message,
};
use tower::ServiceExt;
use uuid::Uuid;

use brokkr_broker::utils::pak;
use brokkr_broker::ws::{ConnectionRegistry, INTERNAL_WS_PATH};
use brokkr_wire::{Heartbeat, WsMessage};

use crate::fixtures::TestFixture;

/// Bind the broker on a random local port and return the bound address plus
/// the shared `ConnectionRegistry` so tests can push synthetic messages.
async fn spawn_broker(
    fixture: &TestFixture,
) -> (std::net::SocketAddr, Arc<ConnectionRegistry>) {
    use brokkr_broker::ws::internal_routes;
    use brokkr_utils::config::Cors;

    let registry = ConnectionRegistry::new();
    let cors = Cors {
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec!["GET".to_string(), "POST".to_string()],
        allowed_headers: vec!["*".to_string()],
        max_age_seconds: 60,
    };

    // Mount only the internal WS router for these tests — we don't need the
    // full v1 surface, and skipping it keeps the harness lean.
    let app = axum::Router::new()
        .merge(internal_routes(fixture.dal.clone(), registry.clone()))
        .layer(axum::extract::Extension(cors))
        .with_state(fixture.dal.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, registry)
}

fn ws_url(addr: std::net::SocketAddr) -> String {
    format!("ws://{}{}", addr, INTERNAL_WS_PATH)
}

#[tokio::test]
async fn ws_upgrade_rejects_unauthenticated() {
    // No Authorization header — auth middleware bails with 401 before the
    // WS extractor ever sees the request, so this we can validate via
    // in-process oneshot without standing up a server.
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(INTERNAL_WS_PATH)
                .header("Host", "broker.test")
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ws_endpoint_is_not_in_openapi_spec() {
    let fixture = TestFixture::new();
    let app = fixture.create_test_router().with_state(fixture.dal.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let paths = spec["paths"].as_object().expect("spec has paths");
    assert!(
        !paths.contains_key(INTERNAL_WS_PATH),
        "internal WS endpoint must not appear in the public OpenAPI spec; \
         this is an explicit ADR-0008 boundary."
    );
}

/// Build a tokio-tungstenite client request with `Authorization: Bearer <pak>`.
/// Tungstenite synthesises the WebSocket headers from the URL automatically;
/// we only need to add the auth header.
fn ws_request_with_pak(url: &str, pak_value: &str) -> tokio_tungstenite::tungstenite::handshake::client::Request {
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", pak_value)).unwrap(),
    );
    request
}

#[tokio::test]
async fn ws_upgrade_rejects_admin_pak() {
    let fixture = TestFixture::new();
    let (addr, _registry) = spawn_broker(&fixture).await;

    let request = ws_request_with_pak(&ws_url(addr), &fixture.admin_pak);
    let err = tokio_tungstenite::connect_async(request)
        .await
        .expect_err("admin PAK must not be allowed to upgrade");

    match err {
        tokio_tungstenite::tungstenite::Error::Http(resp) => {
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
        other => panic!("expected HTTP rejection, got {other:?}"),
    }
}

#[tokio::test]
async fn ws_upgrade_with_agent_pak_round_trips_messages() {
    let fixture = TestFixture::new();
    let (addr, registry) = spawn_broker(&fixture).await;

    // Provision an agent and bind a fresh PAK to it.
    let agent =
        fixture.create_test_agent("WS Test Agent".to_string(), "Test Cluster".to_string());
    let (agent_pak, agent_hash) = pak::create_pak().unwrap();
    fixture
        .dal
        .agents()
        .update_pak_hash(agent.id, agent_hash)
        .unwrap();

    let request = ws_request_with_pak(&ws_url(addr), &agent_pak);
    let (mut socket, response) = tokio_tungstenite::connect_async(request)
        .await
        .expect("upgrade should succeed for a valid agent PAK");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

    // Wait for the registry to reflect the registration. The server-side
    // upgrade callback runs after the handshake completes; poll briefly to
    // avoid a race.
    let connected = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wait_for_connection(&registry, agent.id),
    )
    .await
    .expect("agent should appear in registry within 2s");
    assert!(connected);

    // Agent → broker: send a heartbeat as a text frame.
    let heartbeat = WsMessage::Heartbeat(Heartbeat {
        agent_id: agent.id,
        sent_at: chrono::Utc::now(),
    });
    socket
        .send(Message::Text(serde_json::to_string(&heartbeat).unwrap()))
        .await
        .unwrap();

    // Broker → agent: push a synthetic Heartbeat back through the registry
    // (control lane) and assert the client receives it.
    registry
        .send_control(agent.id, heartbeat.clone())
        .expect("control-lane push should succeed");

    let received = tokio::time::timeout(std::time::Duration::from_secs(2), socket.next())
        .await
        .expect("frame should arrive within 2s")
        .expect("stream not closed")
        .expect("frame ok");
    match received {
        Message::Text(t) => {
            let decoded: WsMessage = serde_json::from_str(&t).unwrap();
            assert!(matches!(decoded, WsMessage::Heartbeat(_)));
        }
        other => panic!("expected text heartbeat, got {other:?}"),
    }

    // Clean disconnect: the close frame should propagate through, the
    // server-side select should fall through, and the registry entry must
    // disappear.
    socket.close(None).await.unwrap();
    drop(socket);

    let cleared = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wait_for_disconnection(&registry, agent.id),
    )
    .await
    .expect("agent should leave the registry within 2s after close");
    assert!(cleared);
}

async fn wait_for_connection(registry: &Arc<ConnectionRegistry>, agent_id: Uuid) -> bool {
    loop {
        if registry.is_connected(agent_id) {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

async fn wait_for_disconnection(registry: &Arc<ConnectionRegistry>, agent_id: Uuid) -> bool {
    loop {
        if !registry.is_connected(agent_id) {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

// =============================================================================
// WS-04: broker → agent push from REST mutation handlers
// =============================================================================
//
// These tests stand up the FULL broker router (v1 + internal_routes +
// shared ConnectionRegistry as an Extension) and exercise the three
// post-commit push paths end to end via real REST calls:
//
//   POST /api/v1/agents/{id}/targets             → WsMessage::TargetChanged
//   POST /api/v1/stacks/{id}/deployment-objects  → WsMessage::StackChanged
//   POST /api/v1/work-orders                     → WsMessage::WorkOrder

async fn spawn_full_broker(fixture: &TestFixture) -> (std::net::SocketAddr, Arc<ConnectionRegistry>) {
    use brokkr_broker::api;
    use brokkr_broker::ws::internal_routes;
    use brokkr_utils::config::Cors;

    let registry = ConnectionRegistry::new();
    let cors = Cors {
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
        ],
        allowed_headers: vec!["*".to_string()],
        max_age_seconds: 60,
    };

    // Mirror `api::configure_api_routes`: the v1 routes need the registry
    // as an Extension so the post-commit push helpers (`ws::push`) can
    // reach it.
    let app = axum::Router::new()
        .merge(api::v1::routes(fixture.dal.clone(), &cors, None))
        .merge(internal_routes(fixture.dal.clone(), registry.clone()))
        .layer(axum::extract::Extension(registry.clone()))
        .with_state(fixture.dal.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, registry)
}

/// Read frames from `socket` until one of the requested `WsMessage` shapes
/// arrives, or 3s elapses. Non-matching frames (heartbeats etc.) are
/// silently skipped so tests don't fail on incidental traffic.
async fn await_message<F>(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    mut matcher: F,
) -> WsMessage
where
    F: FnMut(&WsMessage) -> bool,
{
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let frame = tokio::time::timeout(remaining, socket.next())
            .await
            .expect("timed out waiting for matching WS frame")
            .expect("socket closed unexpectedly")
            .expect("read error");
        if let Message::Text(t) = frame {
            let msg: WsMessage = serde_json::from_str(&t).expect("decode WsMessage");
            if matcher(&msg) {
                return msg;
            }
        }
    }
}

#[tokio::test]
async fn rest_mutations_push_messages_over_ws() {
    use reqwest::Client;
    use serde_json::json;

    let fixture = TestFixture::new();
    let (addr, registry) = spawn_full_broker(&fixture).await;
    let http = Client::new();
    let base = format!("http://{}", addr);

    // 1. Provision agent + PAK and open the WS connection.
    let agent = fixture.create_test_agent("WS-04 agent".into(), "cluster".into());
    let (agent_pak, agent_hash) = pak::create_pak().unwrap();
    fixture.dal.agents().update_pak_hash(agent.id, agent_hash).unwrap();

    let ws_req = ws_request_with_pak(&ws_url(addr), &agent_pak);
    let (mut socket, _resp) = tokio_tungstenite::connect_async(ws_req).await.unwrap();
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wait_for_connection(&registry, agent.id),
    )
    .await
    .expect("registry registers agent within 2s");

    // 2. Create a stack (admin-owned via fixture's admin_generator) and
    //    POST a target — the agent should receive TargetChanged.
    let stack = fixture.create_test_stack(
        "ws04 stack".into(),
        Some("ws04 test".into()),
        fixture.admin_generator.id,
    );

    let resp = http
        .post(format!("{base}/api/v1/agents/{}/targets", agent.id))
        .header("Authorization", format!("Bearer {}", fixture.admin_pak))
        .json(&json!({ "agent_id": agent.id, "stack_id": stack.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let msg = await_message(&mut socket, |m| matches!(m, WsMessage::TargetChanged(_))).await;
    if let WsMessage::TargetChanged(t) = msg {
        assert_eq!(t.agent_id, agent.id);
        assert_eq!(t.stack_id, stack.id);
    }

    // 3. POST a deployment object on that stack — the agent should
    //    receive StackChanged (it now targets the stack from step 2).
    let resp = http
        .post(format!("{base}/api/v1/stacks/{}/deployment-objects", stack.id))
        .header("Authorization", format!("Bearer {}", fixture.admin_pak))
        .json(&json!({
            "yaml_content": "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: ws04\n",
            "is_deletion_marker": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let msg = await_message(&mut socket, |m| matches!(m, WsMessage::StackChanged(_))).await;
    if let WsMessage::StackChanged(s) = msg {
        assert_eq!(s.id, stack.id);
    }

    // 4. POST a work order targeted at the agent — the agent should
    //    receive WorkOrder.
    let resp = http
        .post(format!("{base}/api/v1/work-orders"))
        .header("Authorization", format!("Bearer {}", fixture.admin_pak))
        .json(&json!({
            "work_type": "build",
            "yaml_content": "kind: Build\nmetadata: { name: ws04 }\n",
            "targeting": { "agent_ids": [agent.id] }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let msg = await_message(&mut socket, |m| matches!(m, WsMessage::WorkOrder(_))).await;
    if let WsMessage::WorkOrder(wo) = msg {
        assert_eq!(wo.work_type, "build");
    }
}

#[tokio::test]
async fn push_to_disconnected_agent_is_a_clean_noop() {
    use reqwest::Client;
    use serde_json::json;

    let fixture = TestFixture::new();
    let (addr, _registry) = spawn_full_broker(&fixture).await;
    let http = Client::new();
    let base = format!("http://{}", addr);

    // Agent exists in DB but is NOT WS-connected. The REST mutation must
    // still succeed — push is fire-and-forget.
    let agent = fixture.create_test_agent("ws04 offline".into(), "cluster".into());
    let stack = fixture.create_test_stack(
        "ws04 offline stack".into(),
        None,
        fixture.admin_generator.id,
    );

    let resp = http
        .post(format!("{base}/api/v1/agents/{}/targets", agent.id))
        .header("Authorization", format!("Bearer {}", fixture.admin_pak))
        .json(&json!({ "agent_id": agent.id, "stack_id": stack.id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "REST POST must succeed even with no WS subscriber");
}

// =============================================================================
// WS-05: agent uplink dispatched into the DAL by the broker reader task
// =============================================================================

#[tokio::test]
async fn ws_uplink_persists_heartbeat_event_and_health() {
    use brokkr_models::models::deployment_objects::NewDeploymentObject;
    use brokkr_wire::{AgentEvent, DeploymentHealth, Heartbeat};

    let fixture = TestFixture::new();
    let (addr, registry) = spawn_full_broker(&fixture).await;

    // Provision agent + PAK and connect over WS.
    let agent = fixture.create_test_agent("WS-05 agent".into(), "cluster".into());
    let (agent_pak, agent_hash) = pak::create_pak().unwrap();
    fixture
        .dal
        .agents()
        .update_pak_hash(agent.id, agent_hash)
        .unwrap();

    let request = ws_request_with_pak(&ws_url(addr), &agent_pak);
    let (mut socket, _resp) = tokio_tungstenite::connect_async(request).await.unwrap();
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wait_for_connection(&registry, agent.id),
    )
    .await
    .expect("registry registers agent within 2s");

    // We need a deployment object to anchor agent_event / agent_health to.
    let stack = fixture.create_test_stack(
        "ws05 stack".into(),
        None,
        fixture.admin_generator.id,
    );
    let new_obj = NewDeploymentObject::new(
        stack.id,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: ws05\n".to_string(),
        false,
    )
    .unwrap();
    let object = fixture.dal.deployment_objects().create(&new_obj).unwrap();

    let now = chrono::Utc::now();

    // 1. Heartbeat over WS → broker writes via record_heartbeat. Read it
    //    back via the DAL after a brief settle.
    socket
        .send(Message::Text(
            serde_json::to_string(&WsMessage::Heartbeat(Heartbeat {
                agent_id: agent.id,
                sent_at: now,
            }))
            .unwrap(),
        ))
        .await
        .unwrap();

    let persisted = wait_until(std::time::Duration::from_secs(2), || async {
        fixture
            .dal
            .agents()
            .get(agent.id)
            .ok()
            .flatten()
            .and_then(|a| a.last_heartbeat)
            .is_some()
    })
    .await;
    assert!(persisted, "heartbeat over WS should land in agents.last_heartbeat");

    // 2. AgentEvent over WS → broker inserts an agent_events row.
    let event = AgentEvent {
        id: uuid::Uuid::nil(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        agent_id: agent.id,
        deployment_object_id: object.id,
        event_type: "DEPLOY".to_string(),
        status: "SUCCESS".to_string(),
        message: Some("ws05 over WS".to_string()),
    };
    socket
        .send(Message::Text(
            serde_json::to_string(&WsMessage::AgentEvent(event)).unwrap(),
        ))
        .await
        .unwrap();

    let event_seen = wait_until(std::time::Duration::from_secs(2), || async {
        let rows = fixture.dal.agent_events().list().unwrap_or_default();
        rows.iter().any(|e| {
            e.agent_id == agent.id
                && e.deployment_object_id == object.id
                && e.event_type == "DEPLOY"
                && e.status == "SUCCESS"
        })
    })
    .await;
    assert!(event_seen, "AgentEvent over WS should appear in agent_events");

    // 3. AgentHealth over WS → broker upserts deployment_health.
    let health = DeploymentHealth {
        id: uuid::Uuid::nil(),
        agent_id: agent.id,
        deployment_object_id: object.id,
        status: "healthy".to_string(),
        summary: None,
        checked_at: now,
        created_at: now,
        updated_at: now,
    };
    socket
        .send(Message::Text(
            serde_json::to_string(&WsMessage::AgentHealth(health)).unwrap(),
        ))
        .await
        .unwrap();

    let health_seen = wait_until(std::time::Duration::from_secs(2), || async {
        fixture
            .dal
            .deployment_health()
            .get_by_agent_and_deployment(agent.id, object.id)
            .ok()
            .flatten()
            .map(|h| h.status == "healthy")
            .unwrap_or(false)
    })
    .await;
    assert!(
        health_seen,
        "AgentHealth over WS should appear in deployment_health"
    );
}

/// Repeatedly poll `predicate` until it returns true or `timeout` elapses.
async fn wait_until<F, Fut>(timeout: std::time::Duration, mut predicate: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if predicate().await {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
