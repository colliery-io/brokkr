---
id: ws-13-ws-diagnostics-observability
level: task
title: "WS-13: WS diagnostics + observability metrics (connected agents, by-type rates, dropped lines)"
short_code: "BROKKR-T-0168"
created_at: 2026-05-23T02:12:48.376126+00:00
updated_at: 2026-05-23T02:12:48.376126+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-13: WS diagnostics + observability metrics (connected agents, by-type rates, dropped lines)

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Surface WS connection state for operators, and add Prometheus metrics covering the WS channel and log/event pipeline. Builds on existing Prometheus instrumentation (BROKKR-T-0087).

## Acceptance Criteria

- [ ] Admin/diagnostics endpoint returns per-agent WS connection state (`connected_since`, `last_message_at`, `messages_in/out`)
- [ ] UI agent list shows a "WS" badge reflecting current connection state
- [ ] Metrics added (Prometheus):
  - `brokkr_ws_connected_agents` (gauge)
  - `brokkr_ws_messages_total{direction, type}` (counter)
  - `brokkr_ws_reconnects_total{agent_id}` (counter)
  - `brokkr_ws_dropped_log_lines_total{stack_id}` (counter)
  - `brokkr_ws_log_eviction_runs_total` (counter, paired with WS-09)
- [ ] Metrics covered by tests where reasonable; visible via existing Prometheus scrape

## Implementation Notes

- **Approach**: extend the existing metrics module in `crates/brokkr-broker/src/metrics.rs`. Connection-state query reads from the `ConnectionRegistry` (WS-02).
- **Dependencies**: WS-02 for registry, WS-09 for log/eviction metrics.
- **Risk**: cardinality on `agent_id` and `stack_id` labels. If fleet is large, consider stripping `agent_id` from counters and keeping it only on gauges.