---
id: ws-04-broker-agent-control-plane
level: task
title: "WS-04: Broker→agent control-plane push — work_order, target_changed, stack_changed"
short_code: "BROKKR-T-0159"
created_at: 2026-05-23T02:12:34.311865+00:00
updated_at: 2026-05-23T02:12:34.311865+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-04: Broker→agent control-plane push — work_order, target_changed, stack_changed

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Wire the broker's mutation paths to push inline-payload WS messages to the affected agent's connection. When a work order is created, a target changes, or a stack changes, look up the target agent in the `ConnectionRegistry` and enqueue the corresponding `WsMessage` variant.

## Acceptance Criteria

- [ ] New work orders trigger a `WorkOrder` push to the target agent
- [ ] Target changes trigger `TargetChanged`; stack changes trigger `StackChanged`
- [ ] Payloads use the same Rust types as the REST handlers (no parallel serialization path)
- [ ] If the target agent is not currently WS-connected, the push is a clean no-op (REST polling will pick it up on the next tick)
- [ ] Pushes happen post-commit, not pre-commit (agents must never see state the DB doesn't reflect yet)
- [ ] Integration test: create a work order, assert the connected agent receives the message via WS

## Implementation Notes

- **Approach**: emit pushes from a thin layer above the DAL (post-commit hook on the relevant API handlers), passing through the shared `ConnectionRegistry`. Avoid putting WS-push logic directly in DAL methods.
- **Dependencies**: WS-02.
- **Risk**: ordering — if WS push fires before commit visibility, the agent could GET an empty result. Always push after the DB transaction commits.