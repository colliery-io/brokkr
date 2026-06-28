---
id: slice-7-broker-health-view
level: task
title: "Slice 7: Broker health view — Prometheus metric cards + internal WS connections"
short_code: "BROKKR-T-0262"
created_at: 2026-06-28T01:44:27.035882+00:00
updated_at: 2026-06-28T01:44:27.035882+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Slice 7: Broker health view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Broker health view: Prometheus metric cards + internal WS connections panel, per the handoff §Broker health.

### Type
- [x] Feature — view slice

## Acceptance Criteria

- [ ] Metric cards (auto-fit): Active agents, WS connected (teal), Http req/min (ice), DB queries/min
      (violet), Stacks, Deploy objects — each with the Prometheus metric name as mono sub
      (`brokkr_active_agents`, `brokkr_ws_connected_agents`, `brokkr_http_requests_total`,
      `brokkr_database_queries_total`, `brokkr_stacks_total`, `brokkr_deployment_objects_total`).
- [ ] Bound to the broker **`/metrics`** endpoint (Prometheus text), parsed + polled on the live interval.
- [ ] **Internal WS connections** panel: rows — pulsing teal dot, mono agent name, mono cluster,
      mono `N msg/s`, mono "up {uptime}" — from `GET /api/v1/admin/ws/connections`.
- [ ] Loading/Empty/Error states.

## Dependencies

- Depends on [[BROKKR-T-0255]], [[BROKKR-T-0256]], slice 1.

## Implementation Notes

- Reference: handoff §6 Broker health; broker `metrics.rs` (`/metrics`) + `admin/ws/connections`.
- Parsing Prometheus text in-wasm: a small parser for the named counters/gauges (don't pull a heavy dep).

## Status Updates

*To be added during implementation*
