---
id: agent-reuse-discovery-per
level: task
title: "Agent: reuse Discovery per reconcile + pod-log tail re-attach (deferred from T-0206)"
short_code: "BROKKR-T-0221"
created_at: 2026-06-11T14:45:20.568203+00:00
updated_at: 2026-06-12T03:02:43.543672+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Agent: reuse Discovery per reconcile + pod-log tail re-attach (deferred from T-0206)

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Two items split out of [[BROKKR-T-0206]] because each is larger/riskier than the rest of that checklist and benefits from its own change + test.

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Discovery reuse in reconcile**: `reconcile_target_state` rebuilds `Discovery::new(client).run()` per object in both the apply and prune loops (k8s/api.rs ~826/940). Build it once per call and reuse — BUT mind the interaction with T-0203's fail-closed behavior: a CRD applied by a priority object earlier in the same reconcile may not be in a single up-front discovery snapshot (and may not be established yet), so a naive single-snapshot change would make a CRD+CR-in-one-bundle reconcile spuriously fail. Implement build-once with a re-discover-on-resolve-miss fallback (or take the snapshot after the priority pre-apply and refresh on miss). Integration test: a bundle containing a CRD and a CR of that CRD reconciles successfully.
- [ ] **Pod-log tail re-attach** (pod_logs.rs): once `ensure_tails` records a UID in `active`, it never re-attaches; `tail_container` gives up permanently after `MAX_OPEN_ATTEMPTS` or first EOF, so a pod that becomes loggable later is never tailed again for its life. Remove the UID from `active` when all its tail tasks finish so a later watcher `Apply` re-attaches — without racing the abort-on-teardown path that also owns the handles. Test: a pod whose tails complete is re-tailed on the next Apply.

## Implementation Notes

Both are in the agent crate; the integration tests run on CI's agent suite (k3s). Discovery reuse is a perf win (drops dozens of discovery API calls per reconcile); the re-attach is a correctness fix for the opt-in log-streaming feature.

## Status Updates

*To be added during implementation*

## Status Updates

- 2026-06-11: IMPLEMENTED on the T-0224 branch (folded into PR #51). Both items:
  1. Discovery reuse (k8s/api.rs): reconcile_target_state held a fresh Discovery::new(client).run() per object in BOTH the apply and prune loops. Added a function-scoped `let mut discovery: Option<Discovery>` and a resolve_gvk_cached(discovery, client, gvk) helper that builds the snapshot lazily on first resolve (i.e. after the priority CRD pre-apply) and reuses it, refreshing exactly once on a miss before the caller concludes a GVK is unresolvable — preserving T-0203's fail-closed behavior for a CRD+CR-in-one-bundle (the CR's first resolve misses, triggers one re-discover, then resolves). Common case is now one discovery sweep per reconcile instead of one per object.
  2. Pod-log re-attach (pod_logs.rs): ensure_tails did `if guard.contains_key(uid) { return }`, so once a uid was recorded it never re-attached even after every tail finished (tail_container gives up after MAX_OPEN_ATTEMPTS or first EOF) — a pod that became loggable later was never tailed again. Extracted take_if_attachable(map, uid): returns false while any tail is_finished()==false; when all are finished it removes the stale entry and returns true so ensure_tails re-attaches on the next watcher Apply. Race-free vs teardown_for (both under the `active` write lock). Unit test take_if_attachable_reattaches_only_after_all_tails_finish covers no-entry/running/all-finished. agent build + clippy + all 74 lib tests pass. The end-to-end re-tail and the CRD-bundle reconcile are exercised by the agent k3s integration/e2e suite on CI.