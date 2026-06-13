---
id: slice-2-periodic-recompute-and
level: task
title: "Slice 2: periodic recompute-and-diff sweep for computed fleet signals"
short_code: "BROKKR-T-0230"
created_at: 2026-06-13T14:07:52.252307+00:00
updated_at: 2026-06-13T14:07:52.252307+00:00
parent: fleet-live-push
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0028
---

# Slice 2: periodic recompute-and-diff sweep for computed fleet signals

## Parent Initiative

[[BROKKR-I-0028]]

## Objective

The second half of the hybrid trigger: catch changes in the *computed* signals
(backpressure, health counts) that aren't tied to a discrete event, by
periodically recomputing and re-broadcasting only the agents whose computed
fields changed. Depends on Slice 1 ([[BROKKR-T-0229]]).

## Acceptance Criteria

- [ ] Background task (~15–30s, configurable; mirror existing background-task
      wiring) recomputes the fleet records and compares the *computed* fields
      (pending_object_count, pending_work_orders, claimed_work_orders,
      health_failing, health_degraded) against the last broadcast per agent.
- [ ] Re-broadcasts only agents whose computed fields changed (no-op when nothing
      changed — must not spam subscribers every tick).
- [ ] Reuses the bounded grouped queries from T-0226 (no N+1).
- [ ] Config key for the sweep interval (default ~15–30s); documented.
- [ ] Test: an agent whose backpressure/health changes between ticks gets a
      FleetUpdate; an unchanged fleet produces no frames.

## Implementation Notes

- Hold the last-broadcast computed snapshot per agent in memory (broker process
  state); seed it on startup so the first sweep doesn't broadcast the whole fleet.

## Status Updates

*To be added during implementation*
