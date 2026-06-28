---
id: slice-4-deployments-view-per-stack
level: task
title: "Slice 4: Deployments view — per-stack deployment-object health"
short_code: "BROKKR-T-0259"
created_at: 2026-06-28T01:44:26.854707+00:00
updated_at: 2026-06-28T01:44:26.854707+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Slice 4: Deployments view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Per-stack deployment-object health view per the handoff §Deployments.

### Type
- [x] Feature — view slice

## Acceptance Criteria

- [ ] One panel per stack: header = mono stack name + health pill + label chips; right = mono
      `gen · {generator}`.
- [ ] Rows of deployment objects: mono object id, action pill (`apply` teal / `delete` bad), mono
      kind, per-agent rollup `N✓` (ok) `N⚠` (gold) `N✕` (bad), mono "ago".
- [ ] Bound to stacks + deployment objects + deployment-health APIs; Loading/Empty/Error states.
- [ ] Health colors via the central status→color helper ([[BROKKR-T-0255]]).

## Dependencies

- Depends on [[BROKKR-T-0255]], [[BROKKR-T-0256]], and slice 1.

## Implementation Notes

- Reference: handoff §3 Deployments; broker stacks/deployment-objects + deployment-health endpoints.

## Status Updates

*To be added during implementation*
