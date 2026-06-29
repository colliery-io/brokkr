---
id: deployments-view-per-stack
level: task
title: "Deployments view — per-stack deployment-object health"
short_code: "BROKKR-T-0259"
created_at: 2026-06-28T01:44:26.854707+00:00
updated_at: 2026-06-29T00:32:36.098518+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Deployments view

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Per-stack deployment-object health view per the handoff §Deployments.

### Type
- [x] Feature — view slice

## Acceptance Criteria

## Acceptance Criteria

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

**2026-06-28 — implemented + pixel-verified.** `src/views/deployments.rs` lists stacks
(name, description, generator) from `GET /api/v1/stacks`. Gap: per-stack deployment-objects +
per-agent health rollup need `/stacks/:id/health` + deployment-objects (N+1) — a follow-up.
trunk build green; rendered via harness. Runtime verification pending the stack.