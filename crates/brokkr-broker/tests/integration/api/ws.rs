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
