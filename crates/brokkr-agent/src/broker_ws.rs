/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Agent side of the internal broker↔agent WebSocket channel.
//!
//! Wire protocol: [`brokkr_wire`]. Broker side: `brokkr_broker::ws`.
//!
//! This module owns:
//! - the dial-and-reconnect loop against `/internal/ws/agent`,
//! - a public [`WsState`] watch channel so other agent components (heartbeat,
//!   event emitter, work-order receiver) can choose between WS and REST in
//!   real time,
//! - the per-direction message queues that those components feed and drain.
//!
//! Per ADR-0008 the WS channel is **on by default**; REST polling is the
//! automatic fallback. Setting `agent.ws_force_rest = true` (env or config)
//! short-circuits the whole thing — no dial attempt is ever made and the
//! state stays [`WsState::ForceRestOnly`] for the lifetime of the agent.

use std::time::Duration;

use brokkr_utils::Settings;
use brokkr_wire::WsMessage;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    http::{header, HeaderValue},
    Message,
};
use tracing::{debug, info, warn};

/// Current state of the WS channel from the agent's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsState {
    /// Not connected. Other components must use REST.
    Down,
    /// Connected to the broker. Other components should prefer WS.
    Up,
    /// `ws_force_rest = true` — the agent will never attempt to use WS.
    /// Distinct from `Down` so callers can short-circuit "wait for WS"
    /// logic and avoid logging spurious "WS still down" warnings.
    ForceRestOnly,
}

impl WsState {
    pub fn is_up(self) -> bool {
        matches!(self, WsState::Up)
    }
}

/// Capacity of the outbound queue from the agent's emitters to the WS task.
/// Small but not tiny — bursts of agent events / heartbeats are absorbed,
/// but a wedged WS task surfaces as back-pressure to callers (who fall
/// back to REST) rather than as unbounded growth.
const OUTBOUND_CAPACITY: usize = 256;

/// Capacity of the inbound queue from the WS task to in-agent consumers.
/// Sized for a moderate burst of work-order / target / stack changes.
const INBOUND_CAPACITY: usize = 256;

/// Bounds on the reconnect backoff schedule.
const BACKOFF_INITIAL: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(60);

/// Public handle to the WS client. Constructed by [`spawn`]; the connection
/// task runs in the background until the returned [`JoinHandle`] is dropped
/// or the agent process exits.
pub struct WsClient {
    state: watch::Receiver<WsState>,
    outbound_tx: mpsc::Sender<WsMessage>,
    /// Inbound channel — single consumer. `take_inbound` moves it out so a
    /// downstream component (e.g. the work-order receiver) owns the only
    /// receiver and nothing else can race for messages.
    inbound_rx: Option<mpsc::Receiver<WsMessage>>,
    _task: JoinHandle<()>,
}

impl WsClient {
    /// Watch the connection state. Cheap to clone, safe to await.
    pub fn state(&self) -> watch::Receiver<WsState> {
        self.state.clone()
    }

    /// Sender for outbound messages (heartbeat, agent events, health,
    /// streamed kube events, log lines). If the lane is full or WS is
    /// down the send returns `Err`; callers should fall back to REST.
    pub fn outbound(&self) -> mpsc::Sender<WsMessage> {
        self.outbound_tx.clone()
    }

    /// Cheap clonable handle bundling the outbound sender with a current
    /// state view, intended for emitter call sites that want a single
    /// "try WS, else REST" decision point (see [`WsUplink::try_send`]).
    pub fn uplink(&self) -> WsUplink {
        WsUplink {
            state: self.state.clone(),
            outbound: self.outbound_tx.clone(),
        }
    }

    /// Take ownership of the inbound receiver. Single consumer; subsequent
    /// calls return `None`.
    pub fn take_inbound(&mut self) -> Option<mpsc::Receiver<WsMessage>> {
        self.inbound_rx.take()
    }
}

/// Send-side handle for agent components that want to prefer WS but fall
/// back to REST when WS is down (per ADR-0008). Cheap to `Clone` —
/// share one per emitter rather than locking the [`WsClient`].
#[derive(Clone)]
pub struct WsUplink {
    state: watch::Receiver<WsState>,
    outbound: mpsc::Sender<WsMessage>,
}

impl WsUplink {
    /// True iff the WS state is currently `Up`. `ForceRestOnly` and `Down`
    /// both return false, so call sites short-circuit cleanly when the
    /// agent is configured for REST-only.
    pub fn is_up(&self) -> bool {
        matches!(*self.state.borrow(), WsState::Up)
    }

    /// Try to send a message over WS. Returns `Err(msg)` (giving the
    /// caller their message back) if WS is down or the outbound lane is
    /// full or closed — that signals the caller to take the REST path.
    pub fn try_send(&self, msg: WsMessage) -> Result<(), WsMessage> {
        if !self.is_up() {
            return Err(msg);
        }
        match self.outbound.try_send(msg) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(m))
            | Err(tokio::sync::mpsc::error::TrySendError::Closed(m)) => Err(m),
        }
    }
}

/// Spawn the WS connection task and return a client handle.
///
/// When `settings.agent.ws_force_rest == true`, no task is spawned beyond
/// a no-op keep-alive that holds the watch sender; the state never leaves
/// `ForceRestOnly`. Outbound sends fail closed so callers reliably fall
/// back to REST.
pub fn spawn(settings: &Settings) -> WsClient {
    if settings.agent.ws_force_rest {
        info!("ws_force_rest=true; WebSocket channel disabled, REST-only mode");
        let (state_tx, state_rx) = watch::channel(WsState::ForceRestOnly);
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<WsMessage>(1);
        let (inbound_tx, inbound_rx) = mpsc::channel::<WsMessage>(1);
        // Hold the senders so the watch isn't poisoned and inbound rx
        // doesn't observe an EOF, but never produce any messages.
        let task = tokio::spawn(async move {
            let _state_tx = state_tx;
            let _inbound_tx = inbound_tx;
            // Drain outbound so callers' `try_send` doesn't fill up and
            // give them an illusion of WS-up; we just discard.
            while let Some(msg) = outbound_rx.recv().await {
                debug!(?msg, "discarding WS outbound message: ws_force_rest=true");
            }
        });
        return WsClient {
            state: state_rx,
            outbound_tx,
            inbound_rx: Some(inbound_rx),
            _task: task,
        };
    }

    let (state_tx, state_rx) = watch::channel(WsState::Down);
    let (outbound_tx, outbound_rx) = mpsc::channel::<WsMessage>(OUTBOUND_CAPACITY);
    let (inbound_tx, inbound_rx) = mpsc::channel::<WsMessage>(INBOUND_CAPACITY);

    // Honor ws_url override when set (e.g. WS through a separate ingress, or
    // through the I-0020 A2 chaos toxiproxy). Default: derive from broker_url.
    let url = settings
        .agent
        .ws_url
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| ws_url_from_broker_url(&settings.agent.broker_url));
    let pak = settings.agent.pak.clone();
    let task = tokio::spawn(reconnect_loop(
        url,
        pak,
        state_tx,
        inbound_tx,
        outbound_rx,
    ));

    WsClient {
        state: state_rx,
        outbound_tx,
        inbound_rx: Some(inbound_rx),
        _task: task,
    }
}

/// Convert `http(s)://broker/api/v1`-style URLs into the
/// `ws(s)://broker/internal/ws/agent` form the broker exposes. The agent's
/// `broker_url` setting points at the broker root (without `/api/v1`), so
/// we just swap the scheme and append the path.
pub fn ws_url_from_broker_url(broker_url: &str) -> String {
    let trimmed = broker_url.trim_end_matches('/');
    let with_ws_scheme = if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        // Unrecognised scheme — assume ws:// and let the connect fail
        // loudly rather than silently mangling the URL.
        format!("ws://{trimmed}")
    };
    format!("{with_ws_scheme}/internal/ws/agent")
}

async fn reconnect_loop(
    url: String,
    pak: String,
    state_tx: watch::Sender<WsState>,
    inbound_tx: mpsc::Sender<WsMessage>,
    mut outbound_rx: mpsc::Receiver<WsMessage>,
) {
    let mut backoff = BackoffSchedule::new();
    loop {
        match dial(&url, &pak).await {
            Ok(socket) => {
                info!(%url, "broker WS connected");
                backoff.reset();
                let _ = state_tx.send(WsState::Up);
                run_socket(socket, &inbound_tx, &mut outbound_rx).await;
                let _ = state_tx.send(WsState::Down);
                info!("broker WS disconnected; will reconnect with backoff");
            }
            Err(e) => {
                warn!(%url, error = %e, "broker WS dial failed");
            }
        }

        let delay = backoff.next();
        debug!(?delay, "sleeping before next WS reconnect attempt");
        tokio::time::sleep(delay).await;
    }
}

async fn dial(
    url: &str,
    pak: &str,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Error,
> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {pak}"))
            .map_err(|_| tokio_tungstenite::tungstenite::Error::Url(
                tokio_tungstenite::tungstenite::error::UrlError::UnableToConnect(
                    "invalid PAK header value".into(),
                ),
            ))?,
    );
    let (socket, _resp) = tokio_tungstenite::connect_async(request).await?;
    Ok(socket)
}

async fn run_socket(
    socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    inbound_tx: &mpsc::Sender<WsMessage>,
    outbound_rx: &mut mpsc::Receiver<WsMessage>,
) {
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            // Inbound frame from broker
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Text(t))) => {
                        match serde_json::from_str::<WsMessage>(&t) {
                            Ok(msg) => {
                                if inbound_tx.send(msg).await.is_err() {
                                    debug!("inbound consumer dropped; closing WS");
                                    return;
                                }
                            }
                            Err(e) => warn!(error = %e, "undecodable WS frame; dropping"),
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        warn!("binary WS frame from broker; only text is supported in v1");
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Frame(_))) => {
                        // Raw frame variant; tungstenite exposes this for
                        // re-encoding paths we don't use. Treat as a no-op.
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        debug!("broker closed WS stream");
                        return;
                    }
                    Some(Err(e)) => {
                        debug!(error = %e, "WS read error; reconnecting");
                        return;
                    }
                }
            }

            // Outbound message from agent components
            out = outbound_rx.recv() => {
                let Some(msg) = out else {
                    debug!("outbound channel closed; client shutting down");
                    let _ = sink.close().await;
                    return;
                };
                let text = match serde_json::to_string(&msg) {
                    Ok(t) => t,
                    Err(e) => {
                        warn!(error = %e, "failed to serialise outbound WS message");
                        continue;
                    }
                };
                if let Err(e) = sink.send(Message::Text(text)).await {
                    debug!(error = %e, "WS send failed; reconnecting");
                    return;
                }
            }
        }
    }
}

/// Exponential backoff with capped maximum and ±20% jitter. Sequence:
/// roughly 1, 2, 4, 8, 16, 32, 60, 60, 60, ... seconds, each with jitter.
#[derive(Debug)]
struct BackoffSchedule {
    current: Duration,
}

impl BackoffSchedule {
    fn new() -> Self {
        Self {
            current: BACKOFF_INITIAL,
        }
    }

    fn reset(&mut self) {
        self.current = BACKOFF_INITIAL;
    }

    fn next(&mut self) -> Duration {
        let base = self.current;
        // Advance for the next call before adding jitter, so a successful
        // reset on the next iteration restores the floor cleanly.
        self.current = (self.current * 2).min(BACKOFF_MAX);
        with_jitter(base)
    }
}

fn with_jitter(d: Duration) -> Duration {
    let ms = d.as_millis() as i64;
    let max_jitter = ms / 5;
    if max_jitter <= 0 {
        return d;
    }
    let mut rng = rand::thread_rng();
    let jitter: i64 = rng.gen_range(-max_jitter..=max_jitter);
    Duration::from_millis((ms + jitter).max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_translates_scheme_and_appends_path() {
        assert_eq!(
            ws_url_from_broker_url("http://broker:3000"),
            "ws://broker:3000/internal/ws/agent"
        );
        assert_eq!(
            ws_url_from_broker_url("https://broker.example.com"),
            "wss://broker.example.com/internal/ws/agent"
        );
        assert_eq!(
            ws_url_from_broker_url("http://broker:3000/"),
            "ws://broker:3000/internal/ws/agent"
        );
    }

    #[test]
    fn backoff_grows_exponentially_then_caps() {
        // Disable jitter for determinism by checking the underlying schedule
        // rather than the with_jitter output.
        let mut b = BackoffSchedule::new();
        let raw = std::iter::from_fn(|| {
            let base = b.current;
            b.current = (b.current * 2).min(BACKOFF_MAX);
            Some(base)
        })
        .take(10)
        .collect::<Vec<_>>();

        assert_eq!(raw[0], Duration::from_secs(1));
        assert_eq!(raw[1], Duration::from_secs(2));
        assert_eq!(raw[2], Duration::from_secs(4));
        // Eventually clamped at BACKOFF_MAX.
        assert_eq!(*raw.last().unwrap(), BACKOFF_MAX);
    }

    #[test]
    fn backoff_reset_restores_initial() {
        let mut b = BackoffSchedule::new();
        for _ in 0..5 {
            b.next();
        }
        assert!(b.current > BACKOFF_INITIAL);
        b.reset();
        assert_eq!(b.current, BACKOFF_INITIAL);
    }

    #[test]
    fn jitter_stays_within_twenty_percent() {
        let base = Duration::from_secs(10);
        for _ in 0..1000 {
            let j = with_jitter(base);
            let diff = (j.as_millis() as i64 - base.as_millis() as i64).abs();
            assert!(diff <= base.as_millis() as i64 / 5);
        }
    }

    // --- WS-06: WsUplink decision-point behaviour ---------------------------
    //
    // The "kill WS, observe REST resumes" property collapses to: when the
    // state watch is not `Up`, `try_send` returns the message back to the
    // caller so it can fall back to REST. These tests cover every relevant
    // path through that decision; the round-trip integration coverage that
    // proves the broker accepts the WS path lives in `tests/integration/`.

    fn uplink_with(state: WsState, capacity: usize) -> (WsUplink, watch::Sender<WsState>, mpsc::Receiver<WsMessage>) {
        let (state_tx, state_rx) = watch::channel(state);
        let (tx, rx) = mpsc::channel::<WsMessage>(capacity);
        let uplink = WsUplink {
            state: state_rx,
            outbound: tx,
        };
        (uplink, state_tx, rx)
    }

    fn heartbeat_msg() -> WsMessage {
        WsMessage::Heartbeat(brokkr_wire::Heartbeat {
            agent_id: uuid::Uuid::nil(),
            sent_at: chrono::Utc::now(),
        })
    }

    #[test]
    fn try_send_returns_message_when_down() {
        let (uplink, _state, _rx) = uplink_with(WsState::Down, 8);
        assert!(!uplink.is_up());
        let msg = heartbeat_msg();
        let returned = uplink.try_send(msg).unwrap_err();
        assert!(matches!(returned, WsMessage::Heartbeat(_)));
    }

    #[test]
    fn try_send_returns_message_when_force_rest_only() {
        let (uplink, _state, _rx) = uplink_with(WsState::ForceRestOnly, 8);
        assert!(!uplink.is_up());
        assert!(uplink.try_send(heartbeat_msg()).is_err());
    }

    #[tokio::test]
    async fn try_send_delivers_when_up() {
        let (uplink, _state, mut rx) = uplink_with(WsState::Up, 8);
        assert!(uplink.is_up());
        uplink.try_send(heartbeat_msg()).unwrap();
        let received = rx.recv().await.expect("frame delivered");
        assert!(matches!(received, WsMessage::Heartbeat(_)));
    }

    #[test]
    fn try_send_returns_message_when_lane_full() {
        // Capacity 1, never drain. First send fills the lane; second
        // returns Err with the unsent message so the caller can REST.
        let (uplink, _state, _rx) = uplink_with(WsState::Up, 1);
        uplink.try_send(heartbeat_msg()).unwrap();
        let bounced = uplink.try_send(heartbeat_msg()).unwrap_err();
        assert!(matches!(bounced, WsMessage::Heartbeat(_)));
    }

    #[test]
    fn ws_is_on_by_default_per_adr_0008() {
        // ADR-0008 declares "WS default, REST polling as fallback (opt-out)".
        // The shipped `default.toml` therefore must have `ws_force_rest = false`.
        // Loading without overrides reflects what real deployments inherit.
        let settings = brokkr_utils::Settings::new(None).expect("default settings load");
        assert!(
            !settings.agent.ws_force_rest,
            "agent.ws_force_rest must default to false (ADR-0008 opt-out)"
        );
    }

    #[tokio::test]
    async fn try_send_follows_state_flip_back_to_rest() {
        // Simulate the ADR-load-bearing flow: WS up, then dropped, then
        // up again. The same `WsUplink` instance must consult the live
        // state on every call, not a stale snapshot from construction.
        let (uplink, state_tx, mut rx) = uplink_with(WsState::Up, 8);
        uplink.try_send(heartbeat_msg()).unwrap();
        let _ = rx.recv().await.unwrap();

        state_tx.send(WsState::Down).unwrap();
        assert!(uplink.try_send(heartbeat_msg()).is_err(), "must REST after flip down");

        state_tx.send(WsState::Up).unwrap();
        uplink.try_send(heartbeat_msg()).unwrap();
        assert!(matches!(rx.recv().await, Some(WsMessage::Heartbeat(_))));
    }
}
