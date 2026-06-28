---
id: cross-cutting-live-paused-engine
level: task
title: "Cross-cutting: Live/Paused engine, toast system, prefers-reduced-motion"
short_code: "BROKKR-T-0256"
created_at: 2026-06-28T01:44:26.674406+00:00
updated_at: 2026-06-28T01:44:26.674406+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Cross-cutting: Live/Paused engine, toasts, reduced-motion

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Build the shared runtime the live views and write action depend on: a **Live/Paused** engine
(reactive signals driving polls + WS subscriptions + heartbeat aging + pulse), a **toast**
system, and global `prefers-reduced-motion` handling.

### Type
- [x] Feature — cross-cutting runtime

## Acceptance Criteria

- [ ] `live_on` signal gating all live updates; a 1s clock; pulse animation (`brk-pulse`) on live
      status dots; sweep animation (`brk-sweep`) for indeterminate progress (used by the diagnostic).
- [ ] Live engine abstraction: views register pollers (interval) and/or WS subscriptions; **pausing
      freezes streaming** (counters/clock may keep ticking, per the handoff).
- [ ] Toast system: bottom-right stack, 3px left border in the toast color (ok/bad/info), auto-dismiss
      ~3.4s; emitted on the diagnostic request/completion ([[BROKKR-T-0258]]).
- [ ] `prefers-reduced-motion` disables pulses/sweeps app-wide.
- [ ] WS client helper with reconnect/backoff (shared by `/fleet/live` and future streams).

## Dependencies

- Depends on [[BROKKR-T-0252]] (shell wires the Live/Paused toggle + clock) and [[BROKKR-T-0255]]
  (toast uses primitives). Used by the live views ([[BROKKR-T-0254]], [[BROKKR-T-0257]],
  [[BROKKR-T-0260]], [[BROKKR-T-0261]], [[BROKKR-T-0262]]) and the diagnostic write
  ([[BROKKR-T-0258]]).

## Implementation Notes

- Reference: handoff "Interactions & Behavior" (live engine `tick()`, pause semantics) and
  "Toasts". Map to Leptos signals/effects (not a single mutable component state).

## Status Updates

*To be added during implementation*
