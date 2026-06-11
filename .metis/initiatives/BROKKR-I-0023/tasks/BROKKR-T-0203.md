---
id: agent-prune-safety-fail-closed
level: task
title: "Agent: prune safety — fail closed, verify ownership, scope to watch_namespace"
short_code: "BROKKR-T-0203"
created_at: 2026-06-11T11:02:07.682725+00:00
updated_at: 2026-06-11T11:02:07.682725+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: prune safety — fail closed, verify ownership, scope to watch_namespace

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

The prune path in `k8s/api.rs` can delete resources it shouldn't. (1) `reconcile_target_state`'s apply loop (`api.rs:809-829`) silently skips objects with missing TypeMeta or unresolvable GVK (both `if let`s have no `else`; e.g. discovery raced a just-installed CRD) — then prune (`841-902`) deletes the old copy because its checksum mismatches: delete-without-replace. (2) The prune loop never calls `verify_object_ownership` (the `brokkr.io/owner-id` check `delete_k8s_objects:377` enforces) — anything carrying the stack annotation with a stale checksum is deleted, including objects applied by a different agent. (3) `get_all_objects_by_annotation` (`300-350`) always lists cluster-wide, ignoring `agent.watch_namespace`; under namespace-scoped RBAC every list 403s, each only `warn!`ed (`:344`), so prune silently never works.

## Acceptance Criteria

- [ ] Unresolvable-GVK / missing-TypeMeta in the apply loop is an error (matching `apply_k8s_objects:230-244` behavior); reconcile aborts before prune.
- [ ] Prune deletes are gated on `verify_object_ownership`.
- [ ] Listing scopes to `watch_namespace` when set; all-lists-failed surfaces as an error, not a warn-and-continue.
- [ ] Prune deletes wrapped in `with_retries`, continue past per-object failures, aggregate errors (`890-895` currently aborts remaining prunes on first failure).
- [ ] Tests cover: skipped-apply-then-prune (must NOT delete), foreign-owner object (must NOT delete), namespace-scoped mode.

## Implementation Notes

Consider the stricter invariant: only prune when every desired object confirmed applied this cycle. Self-healing relies on checksum mismatch persisting, so failing closed costs only one tick of delay.

## Status Updates

*To be added during implementation*
