---
id: a2-followup-work-order-completion-chaos
level: task
title: "A2-follow-up: work-order completion chaos — REST fallback drains the queue with WS severed"
short_code: "BROKKR-T-0183"
created_at: 2026-05-26T00:00:00+00:00
updated_at: 2026-05-26T00:00:00+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# A2-follow-up: work-order completion chaos — REST fallback drains the queue with WS severed

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Deliver the A2 criterion that [[BROKKR-T-0171]] deferred: prove that **work
orders still get claimed and completed via REST polling while the WS channel is
severed** — the load-bearing "WS is additive, REST never stops working" claim,
at the full work-order lifecycle (not just heartbeat).

The original deferral was because the only shipped `work_type` was `build`,
which needs Shipwright + ttl.sh (a flaky external dependency, already
`allow_fail` in the e2e suite). The unlock: the agent also handles
`work_type: "custom"`, which just applies the work order's YAML to k3s and
completes — **no external build infra**, so completion is deterministic.

## Acceptance Criteria

- [x] New scenario `angreal tests e2e --scenario ws-workorders`
- [x] Severs WS via toxiproxy, then seeds N (≥ 5) `custom` work orders targeting
      the real docker-compose agent, each applying a unique trivial object
      (ConfigMap) so completions are independent
- [x] Asserts every work order reaches the `work_order_log` as `success: true`,
      `claimed_by = the agent`, within a timeout — i.e. the agent discovered,
      claimed, executed, and completed them over REST while WS was down
- [x] Asserts no duplicate completion: each work-order id appears once in the
      log and is gone from the active `/work-orders` queue
- [x] Restores WS at the end (clean stack for teardown)
- [x] Scenario runs green against the live stack

## Implementation Notes

### Technical Approach

- Reuse the A2 chaos plumbing from [[BROKKR-T-0171]]: `toxiproxy_set_enabled`,
  the `brokkr-integration-test-agent` lookup, the `brokkr_ws_connected_agents`
  gauge waits.
- `custom` work orders apply their YAML via `execute_custom_work_order`
  (`brokkr-agent/src/work_orders/mod.rs`) — a ConfigMap per work order, unique
  name, applies instantly with no scheduling/image pull.
- Completion is observed by polling `GET /work-order-log/{id}` (200 + `success`
  once the agent calls `complete_work_order`). Until then it 404s / errors —
  poll-with-timeout.
- Keep N modest (8) for CI runtime; the point is "all of them, with WS down,"
  not load (that's B4).

### Risk Considerations

- The agent discovers work orders via REST polling on its `polling_interval`
  when WS is down, so the timeout must comfortably exceed several poll cycles +
  apply time. Use ~150s.
- Do NOT use `build` work orders here — that reintroduces the Shipwright/ttl.sh
  flakiness the original deferral was avoiding.

## Status Updates

### 2026-05-26 — Green; closes the A2 deferral

`angreal tests e2e --scenario ws-workorders` (`test_ws_workorders`). With WS
severed via toxiproxy, seeded 8 `custom` work orders (unique ConfigMaps)
targeting the real `brokkr-integration-test-agent`; all 8 were claimed,
applied to k3s, and completed (`success: true`, `claimed_by` = the agent) over
REST polling within the window, and the active queue drained clean.

```
WS severed (gauge < 1) ✓
seeded 8 work orders ✓
completed 1/8 … 8/8 ✓
all 8 work orders completed with WS severed ✓
active queue clean (no leftovers) ✓
✅ PASSED
```

**Finding during bring-up (not a bug, but worth recording):** the first run
got 0/8. Cause: the agent skips work-order processing unless its status is
`ACTIVE` (`commands.rs:308`), and the docker-compose agent boots `INACTIVE`.
The scenario now sets the agent `ACTIVE` first (mirroring Part 5
`test_work_orders`). The agent refreshes its status over REST each cycle, so it
picked up `ACTIVE` and drained the queue **even with WS severed** — which is
itself extra evidence for the REST-fallback claim (status refresh + work-order
discovery + claim + complete all rode REST).

This delivers the A2 criterion [[BROKKR-T-0171]] deferred ("every work order
reaches completed"), using `custom` work orders to keep it deterministic and
free of the Shipwright/ttl.sh flakiness that drove the original deferral.
