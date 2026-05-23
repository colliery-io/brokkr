---
id: ws-02-broker-ws-endpoint-axum
level: task
title: "WS-02: Broker WS endpoint — Axum handler, PAK auth on upgrade, per-agent connection registry"
short_code: "BROKKR-T-0157"
created_at: 2026-05-23T02:12:31.202066+00:00
updated_at: 2026-05-23T02:12:31.202066+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-02: Broker WS endpoint — Axum handler, PAK auth on upgrade, per-agent connection registry

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Add a WebSocket endpoint on the broker (e.g. `GET /internal/ws/agent`) using `axum::extract::ws`. Authenticate the agent's PAK during the upgrade handshake, register the connection in a per-agent connection registry, and run priority-aware read/write loops.

## Acceptance Criteria

- [ ] WS upgrade only succeeds with a valid agent PAK; bad/missing auth returns 401 without upgrading
- [ ] Endpoint is **not** part of the public OpenAPI spec (internal-only, won't appear in generated SDKs)
- [ ] `ConnectionRegistry` exposes lookup by agent id and is safe for concurrent use
- [ ] Per-connection writer applies priority: control-plane messages (WorkOrder, TargetChanged, StackChanged, Heartbeat) preempt log/event traffic
- [ ] Clean teardown on disconnect / drop / agent revoke; no leaked tasks
- [ ] Integration test: agent can upgrade, send heartbeat, receive a synthetic control message, disconnect cleanly

## Implementation Notes

- **Approach**: bounded mpsc per connection with two queues (high/low priority); writer task drains high first. Keep the registry behind a single shared `Arc<AppState>` field.
- **Dependencies**: WS-01. Unblocks WS-04, WS-05.
- **Risk**: PAK auth on upgrade has to integrate with existing middleware — verify it composes with the Axum WS extractor or extract the auth logic into a reusable helper.