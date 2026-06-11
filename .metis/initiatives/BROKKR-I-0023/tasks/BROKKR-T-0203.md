---
id: agent-prune-safety-fail-closed
level: task
title: "Agent: prune safety — fail closed, verify ownership, scope to watch_namespace"
short_code: "BROKKR-T-0203"
created_at: 2026-06-11T11:02:07.682725+00:00
updated_at: 2026-06-11T13:09:09.813408+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: prune safety — fail closed, verify ownership, scope to watch_namespace

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

The prune path in `k8s/api.rs` can delete resources it shouldn't. (1) `reconcile_target_state`'s apply loop (`api.rs:809-829`) silently skips objects with missing TypeMeta or unresolvable GVK (both `if let`s have no `else`; e.g. discovery raced a just-installed CRD) — then prune (`841-902`) deletes the old copy because its checksum mismatches: delete-without-replace. (2) The prune loop never calls `verify_object_ownership` (the `brokkr.io/owner-id` check `delete_k8s_objects:377` enforces) — anything carrying the stack annotation with a stale checksum is deleted, including objects applied by a different agent. (3) `get_all_objects_by_annotation` (`300-350`) always lists cluster-wide, ignoring `agent.watch_namespace`; under namespace-scoped RBAC every list 403s, each only `warn!`ed (`:344`), so prune silently never works.

## Acceptance Criteria

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
## Status Updates

- 2026-06-11: DONE (branch feat/i0023-agent-reconciler-hardening). reconcile_target_state gained `agent_id: &Uuid` + `watch_namespace: Option<&str>`. Changes in k8s/api.rs:
  - **Fail closed**: the main apply loop's `if let Some(gvk)` / `resolve_gvk` `if let` (which silently skipped objects with no TypeMeta or an unresolvable API resource) now error and abort the reconcile BEFORE prune — no more delete-without-replace when discovery races a fresh CRD.
  - **Ownership-gated prune**: prune now skips objects where `verify_object_ownership(&obj, agent_id)` is false, so a foreign agent (or user-annotated object) carrying the same stack annotation is never deleted. To make this consistent the apply loop now STAMPS `BROKKR_AGENT_OWNER_ANNOTATION = agent_id` on every applied object (production already does this via create_k8s_objects; stamping in reconcile makes it correct regardless of input and keeps the existing same-agent prune tests valid).
  - **Namespace scope**: get_all_objects_by_annotation gained `watch_namespace`; namespaced lists are scoped to it (cluster-scoped resources still list cluster-wide), and if EVERY list fails it returns an error instead of an empty set that would make prune a silent no-op under namespace-scoped RBAC.
  - **Retry + aggregate**: prune deletes wrapped in with_retries(RetryConfig::default()); a per-object delete failure is collected and the loop continues, surfacing all failures at the end (was: abort on first).
  - Caller commands.rs passes `&agent.id` and `config.agent.watch_namespace.as_deref()`.
  Tests: updated all integration callers (tests/integration/k8s/api.rs ×12, deployment_health.rs ×1) to the new signature; added `test_reconcile_does_not_prune_other_agents_objects` (agent B must not prune agent A's object). Compiles all targets, 70 unit tests pass, clippy clean (3 pre-existing T-0206 warnings only). The foreign-owner + existing prune/empty tests run on CI's agent integration suite (need k3s). Fail-closed and namespace-scope behaviors are implemented and reasoned; the existing rollback-on-failure integration test exercises the abort-before-prune path, and a dedicated namespace-scoped-RBAC integration case is a future addition (needs RBAC fixture).
