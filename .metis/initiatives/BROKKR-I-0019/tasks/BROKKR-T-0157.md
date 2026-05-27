---
id: ws-02-broker-ws-endpoint-axum
level: task
title: "WS-02: Broker WS endpoint — Axum handler, PAK auth on upgrade, per-agent connection registry"
short_code: "BROKKR-T-0157"
created_at: 2026-05-23T02:12:31.202066+00:00
updated_at: 2026-05-23T03:03:09.140285+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-02: Broker WS endpoint — Axum handler, PAK auth on upgrade, per-agent connection registry

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Add a WebSocket endpoint on the broker (e.g. `GET /internal/ws/agent`) using `axum::extract::ws`. Authenticate the agent's PAK during the upgrade handshake, register the connection in a per-agent connection registry, and run priority-aware read/write loops.

## Acceptance Criteria

## Acceptance Criteria

- [x] WS upgrade only succeeds with a valid agent PAK; bad/missing auth returns 401 without upgrading
- [x] Endpoint is **not** part of the public OpenAPI spec (internal-only, won't appear in generated SDKs)
- [x] `ConnectionRegistry` exposes lookup by agent id and is safe for concurrent use
- [x] Per-connection writer applies priority: control-plane messages (WorkOrder, TargetChanged, StackChanged, Heartbeat) preempt log/event traffic
- [x] Clean teardown on disconnect / drop / agent revoke; no leaked tasks
- [x] Integration test: agent can upgrade, send heartbeat, receive a synthetic control message, disconnect cleanly

## Implementation Notes

- **Approach**: bounded mpsc per connection with two queues (high/low priority); writer task drains high first. Registry held behind `axum::Extension` instead of pushed into DAL/AppState.
- **Dependencies**: WS-01. Unblocks WS-04, WS-05.
- **Risk**: PAK auth on upgrade has to integrate with existing middleware — handled by reusing the v1 auth middleware on the internal-route branch.

## Status Updates

**2026-05-22** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `crates/brokkr-broker/src/ws/{mod,registry,handler}.rs`.
- `/internal/ws/agent` mounted *outside* `/api/v1` (so it never reaches the OpenAPI spec), behind the same PAK middleware as v1. Agent-only restriction enforced in the handler — admin / generator PAKs get `403 Forbidden`.
- `ConnectionRegistry` (`Arc<RwLock<HashMap<Uuid, ConnectionHandle>>>`) injected via `axum::Extension` rather than added to DAL — keeps DAL focused and gives WS-04 push code a clean way to grab the same registry.
- Each connection runs a paired reader/writer task. Writer uses `tokio::select! { biased; control_rx, telemetry_rx }` — the `biased` keyword is what guarantees the ADR's "control plane is never starved by log/event traffic" property. Cleanup uses `unregister_if_matches(agent_id, connected_since)` so a stale writer can't evict a fresh reconnect.
- Lane capacities: control 64, telemetry 1024. Lane-full returns `SendError::LaneUnavailable`; callers should treat this the same as "not connected" and let REST polling backfill.
- Tests:
  - 5 unit tests in `ws::registry::tests`: unknown-agent send, register+route, eviction on reconnect, `unregister_if_matches` generation check, lane-full back-pressure. All pass via `cargo test -p brokkr-broker --lib ws::registry::tests`.
  - 4 integration tests in `tests/integration/api/ws.rs`:
    - `ws_upgrade_rejects_unauthenticated` — 401 via `oneshot` (auth middleware bails before WS extractor)
    - `ws_endpoint_is_not_in_openapi_spec` — asserts `/docs/openapi.json` paths object does not contain `/internal/ws/agent`
    - `ws_upgrade_rejects_admin_pak` — real TCP listener + tokio-tungstenite → 403
    - `ws_upgrade_with_agent_pak_round_trips_messages` — full upgrade, agent sends Heartbeat upstream, broker pushes synthetic Heartbeat back through `registry.send_control`, client receives it, client closes, registry entry clears. All within 2s timeouts.
  - All 4 pass via `angreal tests integration brokkr-broker api::ws`.

**Design notes for downstream tasks**:
- Why no in-process WS upgrade test? Axum's `WebSocketUpgrade` requires `hyper::upgrade::OnUpgrade` in request extensions, which only the real HTTP/1.1 server path installs. `tower::ServiceExt::oneshot` returns 426 `ConnectionNotUpgradable` regardless of header validity. The hybrid (oneshot for auth gating, real listener for handshake) is correct here.
- WS-04 broker push code: grab `Extension<Arc<ConnectionRegistry>>` from request, call `send_control`. The post-commit hook pattern is required — see ADR risk.
- WS-05 agent uplink: agent sends WsMessage text frames; reader_task currently just counts them. WS-05 will add dispatch (to AgentEvents / DeploymentHealth DAL writes).
- Added `tokio-tungstenite = "0.24"` and `brokkr-wire` as broker dev-dependencies for the integration test.