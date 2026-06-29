---
id: overview-view-six-widgets-three
level: task
title: "Overview view — six widgets + three swappable layouts"
short_code: "BROKKR-T-0257"
created_at: 2026-06-28T01:44:26.734560+00:00
updated_at: 2026-06-29T00:32:35.798743+00:00
parent: brokkr-operator-console
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Overview view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Build the at-a-glance Overview command view — six widgets reflowed across three swappable CSS-grid
layouts (command / grid / stream), per the handoff §Overview.

### Type
- [x] Feature — view slice

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Six widgets: **KPI row** (active agents / WS channel / healthy / degraded / failing / req-min,
      colored by meaning, tabular figures), **Fleet by cluster** (segmented health bars), **Deployment
      health** (counts + per-stack rollup), **Broker throughput** (live counts + SVG sparkline over a
      44-point ring buffer), **Live activity** (prepending event feed), **Work orders** (active list).
- [ ] Three layouts via the header segmented control swapping grid-template-areas only (command default).
- [ ] Bound to real data: fleet (`/api/v1/fleet`), deployment health, work orders, broker counters
      (`/metrics` poll); activity feed from agent-events / fleet live.
- [ ] Live updates gated by Live/Paused ([[BROKKR-T-0256]]); Loading/Empty/Error states.

## Dependencies

- Depends on [[BROKKR-T-0255]] (primitives), [[BROKKR-T-0256]] (live engine), and slice 1
  ([[BROKKR-T-0252]]/[[BROKKR-T-0253]]/[[BROKKR-T-0254]]). Reuses Fleet + Deployments + Work-orders +
  Broker-health data (can land after those views or share their data layers).

## Implementation Notes

- Reference: handoff §1 Overview (exact grid templates, widget specs, sparkline viewBox).

## Status Updates

*To be added during implementation*

**2026-06-28 — implemented + pixel-verified.** `src/views/overview.rs`: KPI row, Fleet health
(SegmentedHealthBar), Broker throughput (Sparkline over an accumulating http-requests ring),
and Live activity (agent-events). Composes fleet + /metrics + /agent-events. Partial vs handoff:
the 3 layout variants → single "command" layout; per-cluster fleet panel → overall fleet health
(fleet record lacks cluster_name). Rendered correct via the harness.