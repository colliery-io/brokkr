---
id: agent-wire-ws-inbound-push-frames
level: task
title: "Agent: wire WS inbound push frames into the control loop"
short_code: "BROKKR-T-0204"
created_at: 2026-06-11T11:02:07.728771+00:00
updated_at: 2026-06-11T11:02:07.728771+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: wire WS inbound push frames into the control loop

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

`broker_ws.rs:122` `take_inbound()` has no callers (grep-verified): broker→agent push frames (`WorkOrder`/`TargetChanged`/`StackChanged`, actively sent by `brokkr-broker/src/ws/push.rs` — including the post-create push added in I-0021) are never processed. Worse, after 256 buffered inbound frames, `inbound_tx.send(msg).await` (`broker_ws.rs:335`) blocks forever (receiver alive, never drained), wedging `run_socket` while state stays `Up` — up to 256 queued uplink events are accepted by `try_send` and silently lost before callers fall back to REST.

## Acceptance Criteria

- [ ] The main control loop consumes `take_inbound()`; a `StackChanged`/`TargetChanged`/`WorkOrder` frame triggers an immediate poll/reconcile instead of waiting for the next tick.
- [ ] The 256-frame wedge is impossible: inbound send uses `try_send` (drop + count metric) or the consumer guarantees draining.
- [ ] Integration test: pushed frame causes reconcile without waiting for `polling_interval`; unit test: >256 frames without a consumer does not wedge the socket task.

## Implementation Notes

The push latency win is the whole point of the internal WS channel (I-0019/I-0020) — this closes the last gap. Coordinate with T-0205 so the inbound consumer lives in the right select arm.

## Status Updates

*To be added during implementation*
