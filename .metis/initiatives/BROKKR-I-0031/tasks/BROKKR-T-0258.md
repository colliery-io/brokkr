---
id: agent-detail-slide-over-run
level: task
title: "Agent detail slide-over + run-diagnostic action (the v1 write)"
short_code: "BROKKR-T-0258"
created_at: 2026-06-28T01:44:26.794739+00:00
updated_at: 2026-06-29T00:27:50.930106+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Agent detail slide-over + run-diagnostic

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Add the right-anchored agent-detail slide-over (opened from any Fleet row) and the **v1 write** —
**run diagnostic** — wired to the existing `POST /api/v1/diagnostics`.

### Type
- [x] Feature — view slice + the single v1 write action

## Acceptance Criteria

## Acceptance Criteria

- [ ] Slide-over per handoff: 430px right panel over a scrim; header (agent name + id, ✕); 2×2 grid
      (cluster / last-heartbeat / status pill / health pill); label chips; **recent events** list
      (ago / type / result pill, from agent-events).
- [ ] **Run diagnostic** button → `POST /api/v1/diagnostics`; while pending show the indeterminate
      sweep + "collecting pod statuses, events, log tails…"; poll `GET /diagnostics/:id` and render the
      result block (pod statuses / events / log tails) on completion.
- [ ] Toasts on diagnostic request + completion ([[BROKKR-T-0256]]).
- [ ] The **Activate/Deactivate** button from the design is **omitted/disabled** in v1 (no endpoint;
      deferred per [[BROKKR-A-0010]]) — leave a clear seam.
- [ ] Auth: the diagnostic write uses the operator's credential; surface 4xx as a toast.

## Dependencies

- Depends on [[BROKKR-T-0254]] (Fleet rows open the slide-over), [[BROKKR-T-0255]], [[BROKKR-T-0256]].

## Implementation Notes

- Reference: handoff "Agent detail slide-over" + diagnostics API (`crates/brokkr-broker/src/api/v1/diagnostics.rs`:
  `POST /diagnostics`, `GET /diagnostics/:id`).

## Status Updates

*To be added during implementation*
**2026-06-28 — implemented + pixel-verified.** Fleet rows clickable → `SlideOver` agent detail
(name, id, status/health pills, ws, heartbeat) + **Run diagnostic** button → `POST /api/v1/diagnostics`
(`api::create_diagnostic`) with request/queued/failed toasts. Activate/Deactivate intentionally
omitted (deferred per ADR-0010). Diagnostic-result polling is a follow-up. Verified via harness
(slide-over screenshot).
