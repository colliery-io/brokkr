---
id: ws-04-broker-agent-control-plane
level: task
title: "WS-04: Broker→agent control-plane push — work_order, target_changed, stack_changed"
short_code: "BROKKR-T-0159"
created_at: 2026-05-23T02:12:34.311865+00:00
updated_at: 2026-05-23T03:50:28.998652+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-04: Broker→agent control-plane push — work_order, target_changed, stack_changed

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Wire the broker's mutation paths to push inline-payload WS messages to the affected agent's connection. When a work order is created, a target changes, or a stack changes, look up the target agent in the `ConnectionRegistry` and enqueue the corresponding `WsMessage` variant.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] New work orders trigger a `WorkOrder` push to the target agent (`create_work_order` → `push_work_order` against `targeting.agent_ids`)
- [x] Target changes trigger `TargetChanged`; stack changes trigger `StackChanged` (`add_target` → `push_target_changed`; `create_deployment_object` → `push_stack_changed_to_targets`)
- [x] Payloads use the same Rust types as the REST handlers (no parallel serialization path) — `brokkr-wire` re-exports the `brokkr-models` types verbatim
- [x] If the target agent is not currently WS-connected, the push is a clean no-op (REST polling will pick it up on the next tick) — `push::deliver` swallows `SendError::NotConnected` with a debug log; covered by `push_to_disconnected_agent_is_a_clean_noop`
- [x] Pushes happen post-commit, not pre-commit — every `push_*` call site is in the success branch after the DAL returns Ok
- [x] Integration test: create a work order, assert the connected agent receives the message via WS — covered by `rest_mutations_push_messages_over_ws`

## Implementation Notes

- **Approach**: helpers live in `crate::ws::push`; called from the REST handlers' success branches. Registry exposed to v1 handlers via a single `.layer(axum::Extension(ws_registry))` on the parent router in `configure_api_routes`.
- **Dependencies**: WS-02.
- **Risk**: ordering — handled by placing push calls *after* the DAL operations return Ok; the DB transaction has therefore committed before the push frame is enqueued.

## Status Updates

**2026-05-22** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `crates/brokkr-broker/src/ws/push.rs` with three fire-and-forget helpers: `push_work_order(reg, work_order, &agent_ids)`, `push_target_changed(reg, target)`, `push_stack_changed_to_targets(reg, dal, stack)`. Errors (`NotConnected`, `LaneUnavailable`) are logged and dropped — REST polling remains the source of truth per ADR-0008.
- `configure_api_routes` now does `.layer(axum::Extension(ws_registry))` on the parent router, so every v1 handler can grab the registry as `Extension<Arc<ConnectionRegistry>>` without changing the router state type.
- Three handler hooks added:
  - `api/v1/work_orders.rs::create_work_order` — pushes `WorkOrder` to each explicitly-targeted agent
  - `api/v1/agents.rs::add_target` — pushes `TargetChanged`
  - `api/v1/stacks.rs::create_deployment_object` — pushes `StackChanged` to every agent currently targeting the stack
- Tests (`tests/integration/api/ws.rs`):
  - `rest_mutations_push_messages_over_ws` — stands up the **full** broker router (v1 + internal + shared registry Extension), opens a real WS connection as an agent, then drives all three push paths via reqwest. Each push frame is asserted on the WS within a 3s deadline (`await_message` helper skips incidental frames).
  - `push_to_disconnected_agent_is_a_clean_noop` — POSTs a target for an agent that never WS-connected; REST returns 201, no errors. Proves fire-and-forget semantics.
- Total WS integration test count is now 6/6 green via `angreal tests integration brokkr-broker api::ws`.

**Choices worth noting for downstream tasks**:
- Remove-target push was deliberately skipped in v1 (REST polling surfaces the deletion on next tick). The wire body for `TargetChanged` is a created-target shape; signalling "your target was removed" is a v2 wire change.
- Label / annotation targeting on work orders is **not** pushed yet — only explicit `targeting.agent_ids` triggers a push. Selector-based targeting requires resolving the matching agent set, which is more involved DAL work. WS-04 follow-up.
- New stacks (no targets yet) are intentionally not pushed; nothing to push to. `update_stack` is also unhooked because it's a metadata update; the meaningful change for agents is `create_deployment_object`, which IS hooked.
- A `spawn_full_broker(fixture)` helper was added to the test file so future tests (WS-13 diagnostics, WS-09 ingestion via WS uplink) can reuse the full-router wiring.