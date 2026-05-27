---
id: ws-03-agent-ws-client-connect
level: task
title: "WS-03: Agent WS client — connect, reconnect/backoff, REST fallback path, force-REST config flag"
short_code: "BROKKR-T-0158"
created_at: 2026-05-23T02:12:32.745717+00:00
updated_at: 2026-05-23T03:26:15.692372+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-03: Agent WS client — connect, reconnect/backoff, REST fallback path, force-REST config flag

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Implement the WebSocket client side in `brokkr-agent`: dial the broker WS endpoint, authenticate with the agent's PAK, run read/write loops, reconnect with exponential backoff + jitter on disconnect, and continue REST polling while disconnected. Add a `--force-rest` config flag for environments where WS must not be used.

## Acceptance Criteria

## Acceptance Criteria

- [x] Agent connects to broker WS endpoint on startup using PAK
- [x] Receives + dispatches typed `WsMessage` variants (decoded in `run_socket`; pushed into the inbound mpsc for WS-04 / WS-05 consumers)
- [x] On disconnect, exponential backoff (capped, with jitter) controls reconnect attempts
- [x] While disconnected, the agent's existing REST polling continues to run (state watch exposes `Down`; existing REST polling tasks are unaffected by WS state)
- [x] `force_rest_only` config flag (env + CLI) prevents any WS dial attempt (config field `agent.ws_force_rest`; pinned `ForceRestOnly` state; no task ever calls `dial`)
- [x] Unit test: backoff schedule is sane; integration test: reconnects after broker restart

## Implementation Notes

- **Approach**: long-lived `tokio::spawn` task owning the connection; `tokio::sync::watch<WsState>` published for other agent components; bounded `mpsc` queues for inbound/outbound `WsMessage`.
- **Dependencies**: WS-01. Unblocks WS-04, WS-05, WS-06.
- **Risk**: REST polling must not double-process a message that also arrived via WS — verified: the existing REST work-order claim path is already idempotent (DB transaction + status field gating), so a duplicate arrival is a no-op.

## Status Updates

**2026-05-22** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `crates/brokkr-agent/src/broker_ws.rs`. Public API: `WsState`, `WsClient`, `spawn(&Settings) -> WsClient`, `ws_url_from_broker_url`.
- Config: added `Agent::ws_force_rest: bool` (default false via `#[serde(default)]`); `default.toml` documents the opt-out.
- Reconnect loop: exponential backoff starting at 1s, doubling to a 60s cap, with ±20% uniform jitter. `BackoffSchedule::reset()` is called after a successful connect so the next disconnect starts from 1s again.
- Force-rest mode: when `ws_force_rest=true`, the task is spawned but never calls `dial`. State is pinned at `ForceRestOnly`. Outbound sends are silently discarded so callers' `try_send` doesn't deceive them into thinking WS is live.
- URL translation: `http://broker:3000` → `ws://broker:3000/internal/ws/agent` (and the `https` → `wss` variant). Trailing slash on `broker_url` is stripped.
- Tests:
  - 4 unit tests: `ws_url_translates_scheme_and_appends_path`, `backoff_grows_exponentially_then_caps`, `backoff_reset_restores_initial`, `jitter_stays_within_twenty_percent`. All pass via `cargo test -p brokkr-agent --lib broker_ws`.
  - 3 integration tests (`tests/integration/broker_ws.rs`, no docker needed — in-test axum WS server):
    - `client_connects_and_reaches_up_state` — state goes Down → Up within 10s
    - `client_reconnects_after_broker_restart` — Up → Down on graceful shutdown of broker, Down → Up after re-binding on the same port
    - `force_rest_pins_state_and_skips_dial` — state stays `ForceRestOnly` indefinitely, no dial against an unreachable port
  - All 3 pass via `cargo test -p brokkr-agent --test integration broker_ws`.

**Lessons / notes for downstream tasks**:
- `JoinHandle::abort()` on an `axum::serve` task does NOT close already-upgraded WS connections — axum spawns each connection on its own task. The shutdown signal has to reach the per-connection handler too. The test uses a shared `Notify` plumbed into each WS handler; agent-side does not need this (the agent is the client and just sees EOF when broker tasks die).
- A silent failure mode I hit while writing this: I added `pub mod broker_ws;` to `lib.rs` via Edit, but the Edit had been rejected (needed a Read first) and I didn't notice. Symptom: `cargo build` succeeded but `cargo test --lib` ran 0 broker_ws tests. Always re-check that `lib.rs` actually contains the module declaration after editing.
- WS-04 / WS-05 should pull `WsClient` from agent main, call `take_inbound()` for the work-order receiver, and use `outbound()` for the heartbeat / event sink. The `WsState::is_up()` helper is the gate for the WS-vs-REST decision in WS-05's `BrokerSink`.
- Added `tokio-tungstenite`, `futures`, `rand`, `brokkr-wire` as agent deps; `axum/ws` as dev-dep for the in-test broker server.