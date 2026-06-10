/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for the agent's broker WebSocket client (WS-03,
//! BROKKR-T-0158).
//!
//! Stands up a minimal in-test broker WS server (no DAL, no PAK lookup —
//! the agent's contract with the broker is just "WS upgrade at
//! `/internal/ws/agent` with a `Bearer <pak>` header") so we can:
//! - assert the state watch transitions Down → Up on a successful dial,
//! - kill the server and assert Up → Down,
//! - restart it and assert Down → Up again (reconnect with backoff works),
//! - assert `ws_force_rest = true` short-circuits all dialling and pins
//!   the state at `ForceRestOnly`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use brokkr_agent::broker_ws::{self, WsState};
use brokkr_utils::config::Settings;
use tokio::net::TcpListener;
use tokio::sync::Notify;

const SHORT_TIMEOUT: Duration = Duration::from_secs(10);

/// Per-connection cancellation: shared with all in-flight WS handlers so a
/// test-side `notify_waiters()` drops every live socket. This is what
/// `JoinHandle::abort()` alone can't do — axum spawns each connection on
/// its own task, and the serve loop's task being aborted doesn't propagate.
type ShutdownNotify = Arc<Notify>;

async fn ws_accept_with_cancel(
    upgrade: WebSocketUpgrade,
    cancel: ShutdownNotify,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |mut socket: WebSocket| async move {
        loop {
            tokio::select! {
                _ = cancel.notified() => {
                    let _ = socket.close().await;
                    return;
                }
                frame = socket.recv() => match frame {
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return,
                    Some(Ok(_)) => {}
                }
            }
        }
    })
}

/// Spawn a test broker bound to a specific address. Returns a graceful-
/// shutdown notifier — calling `notify_waiters()` causes both the accept
/// loop AND every live connection handler to exit, which is what makes
/// the agent observe `WsState::Down`.
async fn spawn_test_broker_on(addr: SocketAddr) -> ShutdownNotify {
    let listener = TcpListener::bind(addr).await.unwrap();
    let shutdown = Arc::new(Notify::new());
    let shutdown_serve = shutdown.clone();
    let shutdown_conn = shutdown.clone();
    let app = Router::new().route(
        "/internal/ws/agent",
        get(move |u| ws_accept_with_cancel(u, shutdown_conn.clone())),
    );
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move { shutdown_serve.notified().await })
            .await;
    });
    shutdown
}

async fn spawn_test_broker() -> (SocketAddr, ShutdownNotify) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let shutdown = spawn_test_broker_on(addr).await;
    (addr, shutdown)
}

fn settings_for_broker(addr: SocketAddr, force_rest: bool) -> Settings {
    let mut s = Settings::new(None).expect("default settings load");
    s.agent.broker_url = format!("http://{addr}");
    s.agent.pak = "brokkr_test_pak".to_string();
    s.agent.ws_force_rest = force_rest;
    s
}

async fn wait_for_state(
    mut watch: tokio::sync::watch::Receiver<WsState>,
    want: WsState,
) -> WsState {
    tokio::time::timeout(SHORT_TIMEOUT, async {
        loop {
            if *watch.borrow() == want {
                return *watch.borrow();
            }
            if watch.changed().await.is_err() {
                return *watch.borrow();
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for WS state {:?}", want))
}

#[tokio::test]
async fn client_connects_and_reaches_up_state() {
    let (addr, shutdown) = spawn_test_broker().await;
    let settings = settings_for_broker(addr, false);
    let client = broker_ws::spawn(&settings);

    let observed = wait_for_state(client.state(), WsState::Up).await;
    assert_eq!(observed, WsState::Up);

    shutdown.notify_waiters();
}

#[tokio::test]
async fn client_reconnects_after_broker_restart() {
    // First lifecycle: broker up, agent connects, broker shuts down.
    let (addr, shutdown1) = spawn_test_broker().await;
    let settings = settings_for_broker(addr, false);
    let client = broker_ws::spawn(&settings);

    wait_for_state(client.state(), WsState::Up).await;
    shutdown1.notify_waiters();
    wait_for_state(client.state(), WsState::Down).await;

    // Second lifecycle: bring a new broker up on the SAME address so the
    // agent's existing reconnect loop can find it.
    let _shutdown2 = spawn_test_broker_on(addr).await;
    wait_for_state(client.state(), WsState::Up).await;
}

#[tokio::test]
async fn force_rest_pins_state_and_skips_dial() {
    // Bind nothing — if the client tried to dial it would just keep
    // failing. Instead it must short-circuit at construction time.
    let unreachable_addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
    let settings = settings_for_broker(unreachable_addr, true);
    let client = broker_ws::spawn(&settings);
    let state = client.state();

    // No need to await a transition — the construction-time guarantee is
    // that ForceRestOnly is the initial value.
    assert_eq!(*state.borrow(), WsState::ForceRestOnly);

    // Wait a moment to ensure the loop doesn't somehow flip it. (If it
    // ever does, this catches the regression cheaply.)
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(*state.borrow(), WsState::ForceRestOnly);
}
