---
id: ws-06-ws-default-cutover-chaos
level: task
title: "WS-06: WS-default cutover + chaos tests for REST fallback path"
short_code: "BROKKR-T-0161"
created_at: 2026-05-23T02:12:37.441501+00:00
updated_at: 2026-05-23T02:12:37.441501+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-06: WS-default cutover + chaos tests for REST fallback path

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Flip the agent's default to WS-on (opt-out, not opt-in), and harden the REST polling fallback with chaos integration tests that prove no work is missed when the WS channel misbehaves. This is the gate that lets the WS-by-default decision ship.

## Acceptance Criteria

- [ ] Agent config default: WS enabled; `force_rest_only=false`
- [ ] Chaos test: agent reconciles a new work order that was created while WS was forcibly dropped — it must surface on the next REST tick
- [ ] Chaos test: agent recovers from broker restart (reconnects, resumes WS, no missed work)
- [ ] Chaos test: agent recovers from a network blip mid-message (backoff schedule observed)
- [ ] Chaos test: agent with `force_rest_only=true` never opens a WS connection, behavior identical to pre-initiative
- [ ] All tests runnable via `angreal tests integration` per [[feedback_use_angreal_for_tests]]

## Implementation Notes

- **Approach**: extend the existing agent integration test harness with a controllable "WS kill switch" on the broker mock side. Use this to simulate drops and asserts that polling backfills.
- **Dependencies**: WS-04, WS-05 (the full agent path must be functional first).
- **Risk**: This task is **load-bearing** for the WS-default decision — if it can't be made green reliably, the default should stay opt-in (per ADR).