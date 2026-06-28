---
id: slice-6-work-orders-view-active
level: task
title: "Work orders view — active (live progress) + history"
short_code: "BROKKR-T-0261"
created_at: 2026-06-28T01:44:26.976822+00:00
updated_at: 2026-06-28T01:44:26.976822+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Work orders view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Work-orders view: Active (live progress) + History, per the handoff §Work orders.

### Type
- [x] Feature — view slice

## Acceptance Criteria

- [ ] **Active** section: rows — mono id, type chip, status pill, progress bar (ice; muted when
      pending), mono meta, mono "ago"; progress advances live (gated by Live/Paused).
- [ ] **History** section: rows — mono id, type chip, status pill (completed/failed), mono detail,
      mono "ago".
- [ ] Bound to the broker work-orders API; live progress via poll or WS work-order-progress frames
      (use poll if no stream).
- [ ] Loading/Empty/Error states.

## Dependencies

- Depends on [[BROKKR-T-0255]], [[BROKKR-T-0256]], slice 1.

## Implementation Notes

- Reference: handoff §5 Work orders; broker work-orders endpoints.

## Status Updates

*To be added during implementation*
