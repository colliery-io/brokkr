---
id: namespace-rollback-deletes-pre
level: task
title: "Namespace rollback deletes pre-existing namespaces on reconciliation failure"
short_code: "BROKKR-T-0193"
created_at: 2026-06-10T03:18:27.529625+00:00
updated_at: 2026-06-10T11:19:09.240802+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Namespace rollback deletes pre-existing namespaces on reconciliation failure

## Objective

`reconcile_target_state` (`crates/brokkr-agent/src/k8s/api.rs:686-810`) tracks the names of every Namespace object **declared in the deployment object** and, on failure during priority apply, validation, or the main apply, calls `rollback_namespaces`, which best-effort deletes them. It never checks whether the namespace existed before this reconciliation — so a deployment object that declares a namespace which was already present (created manually or by another stack) gets that namespace **deleted on rollback**, taking everything in it with it. Deleting a namespace deletes all resources inside it; this is a data-loss-grade footgun.

Fix: record whether each namespace was created by this reconciliation (e.g. check existence before apply, or compare creation timestamp/ownership annotation) and roll back only namespaces this pass actually created. Consider whether namespace rollback is wanted at all given server-side-apply idempotency.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Rollback deletes only namespaces created by the failing reconciliation pass
- [ ] Integration test: pre-existing namespace declared in a failing deployment object survives
- [ ] `docs/src/explanation/reconciliation.md` rollback description updated

## Status Updates

- 2026-06-09: Found while verifying rollback behavior for the docs run (the docs previously claimed only "namespaces created this reconciliation" are rolled back — the code is less careful than the docs were).
- 2026-06-10: IMPLEMENTED: namespace rollback now records only namespaces that did NOT exist before this reconciliation (existence check via `get_opt` before apply; lookup errors err on the side of not deleting) — `k8s/api.rs`. Pre-existing namespaces declared in a failing deployment object are no longer deleted. Remaining AC: integration test; update explanation/reconciliation.md rollback wording.
- 2026-06-10 (closure pass): integration tests written — k8s/api.rs gains rollback-spares-preexisting-namespace + rollback-deletes-new-namespace tests.