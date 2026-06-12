---
id: follow-up-agent-events-retention
level: task
title: "Follow-up: agent_events retention/eviction policy (unbounded growth)"
short_code: "BROKKR-T-0228"
created_at: 2026-06-12T21:39:43.790408+00:00
updated_at: 2026-06-12T21:39:43.790408+00:00
parent: agent-fleet-legibility
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0027
---

# Follow-up: agent_events retention/eviction policy (unbounded growth)

## Parent Initiative

[[BROKKR-I-0027]]

## Objective

`agent_events` currently has no eviction — rows accumulate forever (soft-deleted
only on agent-delete cascade). Surfaced while scoping the fleet activity feed
(I-0027). Orthogonal to legibility but a real operational liability at fleet
scale. Add a retention policy.

## Backlog Item Details

### Type
- [x] Tech Debt

## Acceptance Criteria

- [ ] A retention policy for `agent_events` (configurable window; choose a sane
      default — note this is operator history, so likely a longer window than the
      6h telemetry ceiling, e.g. days/weeks). Decide hard-delete vs soft-delete.
- [ ] A background eviction task removes events older than the window (mirror the
      existing retention worker / diagnostic-cleanup patterns).
- [ ] Config key + default documented; `0`/unset disables (keep current behavior).
- [ ] Test: events older than the window are evicted; recent ones retained; the
      fleet activity feed (T-0226) still returns the latest N.

## Implementation Notes

- Reference the existing telemetry retention worker (WS 6h ceiling) and the
  diagnostic-cleanup task as patterns.
- Coordinate the default window with the fleet activity feed's "recent N" needs.

## Status Updates

*To be added during implementation*
