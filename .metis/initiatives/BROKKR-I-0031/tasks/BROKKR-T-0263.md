---
id: slice-8-webhooks-view
level: task
title: "Webhooks view — subscriptions + recent deliveries"
short_code: "BROKKR-T-0263"
created_at: 2026-06-28T01:44:27.093925+00:00
updated_at: 2026-06-28T01:44:27.093925+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Webhooks view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Webhooks view: subscriptions + recent deliveries, per the handoff §Webhooks. Read-only.

### Type
- [x] Feature — view slice (completes the seven-view monitor)

## Acceptance Criteria

- [ ] **Subscriptions** cards (auto-fit): name + state pill (enabled/disabled), mono url (ellipsized),
      event chips (mono, ice-tinted).
- [ ] **Recent deliveries** rows: mono id, mono event (ice), mono hook name, mono "try N", status pill
      (success/failed/dead/pending), mono "ago".
- [ ] Bound to the broker webhook subscriptions + deliveries APIs; Loading/Empty/Error states.
- [ ] Read-only — no create/edit/delete in v1 (sensitive webhook secrets are encrypted server-side and
      out of scope for the console).

## Dependencies

- Depends on [[BROKKR-T-0255]], [[BROKKR-T-0256]], slice 1.

## Implementation Notes

- Reference: handoff §7 Webhooks; broker webhooks endpoints.

## Status Updates

*To be added during implementation*
