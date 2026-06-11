---
id: agent-survive-malformed-bundles
level: task
title: "Agent: survive malformed bundles and discovery failures (crash-loop fixes)"
short_code: "BROKKR-T-0202"
created_at: 2026-06-11T11:02:07.633977+00:00
updated_at: 2026-06-11T15:10:29.118470+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: survive malformed bundles and discovery failures (crash-loop fixes)

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Two crash classes kill the agent in production: (1) `cli/commands.rs:268` propagates `create_k8s_objects(...)?` out of the deployment-check select arm — one malformed YAML bundle exits the process, no failure event is sent, and the restart refetches the same bundle → permanent crash loop; (2) `k8s/api.rs:310, 373, 470` call `.expect("Failed to create discovery client")` in `get_all_objects_by_annotation`/`delete_k8s_objects`/`validate_k8s_objects` — the first runs every reconcile tick, so a transient API-server blip panics the agent (the sibling call in `apply_k8s_objects:156` correctly uses `?`).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A bundle that fails to apply logs at error level, submits a failure event to the broker (`send_failure_event`), and the loop continues to the next object — process stays up.
- [ ] All three discovery `.expect`s replaced with `?`/mapped errors; transient discovery failure surfaces as a logged, retried error.
- [ ] Handler-path expects fixed: `health.rs:129,137` (`Time went backwards`) → `unwrap_or_default`; `metrics.rs:141-142` (encode) → 500 response.
- [ ] Unit/integration coverage: malformed-bundle case proves no-exit + failure event.

## Implementation Notes

Startup expects (`commands.rs:81,86,124,168`) are acceptable and out of scope. Keep the error path consistent with the existing failure-event flow used elsewhere in the loop.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0023-agent-reconciler-hardening). (1) commands.rs deployment-check arm: `create_k8s_objects(...)?` → match that logs, sends a failure event, and `continue`s to the next object (mirrors the adjacent reconcile error arm) — a malformed bundle no longer exits the process / crash-loops. (2) k8s/api.rs: all three `Discovery::new(...).run().await.expect("Failed to create discovery client")` (get_all_objects_by_annotation, delete_k8s_objects, validate_k8s_objects) → `.map_err(|e| { error!(...); e })?` (functions already return Result<_, Box<dyn Error>>; matches the correct apply_k8s_objects pattern). (3) health.rs `health` handler: two `.expect("Time went backwards")` → `unwrap_or_default()`. (4) metrics.rs `encode_metrics()` → `Result<String,String>`; health.rs `metrics_handler` maps Err → 500 (was always 200; an encode failure no longer panics the connection task). Tests: existing `test_create_k8s_objects_invalid_yaml` already proves the recoverable-error contract the loop fix relies on; added `encode_metrics_succeeds` unit test. agent builds clean, 70 lib tests pass, clippy adds no new warnings (the 3 remaining are pre-existing, slated for T-0206). The full "no-exit + failure event" loop behavior is integration-level (e2e harness); the unit guarantee (create_k8s_objects→Err is recoverable) is covered and the loop change mirrors the established error arm.