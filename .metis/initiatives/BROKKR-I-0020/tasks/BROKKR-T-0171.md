---
id: a2-full-loop-chaos-test-rest
level: task
title: "A2: Full-loop chaos test — REST fallback never drops a reconciliation"
short_code: "BROKKR-T-0171"
created_at: 2026-05-24T12:56:37.000000+00:00
updated_at: 2026-05-24T12:56:37.000000+00:00
parent: BROKKR-I-0020
blocked_by:
  - BROKKR-T-0170
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# A2: Full-loop chaos test — REST fallback never drops a reconciliation

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Prove the load-bearing design claim of I-0019: **WS is an additive
optimization with automatic REST fallback that can never cause an outage.**
WS-06 covered the structural decision branch; this task drives the full
agent runtime against a chaos-tested broker connection and asserts zero
dropped reconciliations.

## Acceptance Criteria

- [ ] Chaos scenario invokable via `angreal tests e2e --scenario ws-chaos`
- [ ] Seeds N work orders (N ≥ 20) through the broker
- [ ] Network proxy (toxiproxy or `tc netem`) sits between agent and broker;
      WS connection is severed at randomized intervals while REST stays reachable
- [ ] Asserts every work order reaches `completed` state
- [ ] Asserts duplicate-reconcile detection holds (no work order reconciled twice)
- [ ] Captures `brokkr_ws_messages_total{direction,type}` deltas; asserts the
      WS drop is observable in metrics (reconnects counted, not silent)
- [ ] Asserts the REST-poll fallback path actually fires (logs or a counter
      shows REST GETs increasing during WS-down windows)
- [ ] Scenario runs green in CI

## Implementation Notes

### Technical Approach

- Build on the harness from [[BROKKR-T-0170]]; add toxiproxy or a similar
  TCP-layer chaos tool between agent and broker so we can cut WS without
  killing the broker
- Randomized severance intervals (e.g. uniform in [2s, 15s]) over a 60s window
  exercise the reconnect/backoff state machine more aggressively than a single
  kill
- Use a small N (20) for CI; the scenario should accept a `--n` arg so we can
  drive it harder locally
- For "duplicate-reconcile detection holds": query the broker's `work_orders`
  table at the end and assert `state_transitions` count per work order matches
  the expected (acknowledged → in_progress → completed) without extras

### Dependencies

- [[BROKKR-T-0170]] (A1) for the base e2e harness

### Risk Considerations

- This test is the closest thing we have to a production sanity check for the
  whole feature. If it's flaky, treat the flake as a bug in the WS/REST
  contract, not a test problem — that's the whole point
- Don't muddy the test by also exercising real k8s; that's [[BROKKR-T-0172]]'s job.
  Here, "reconciliation" means the work order state machine on the broker,
  not actual k8s deploy

## Status Updates

*To be added during implementation*
