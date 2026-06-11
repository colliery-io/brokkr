---
id: ts-sdk-remove-retry-from-non
level: task
title: "TS SDK: remove retry from non-idempotent POSTs"
short_code: "BROKKR-T-0212"
created_at: 2026-06-11T11:02:08.123938+00:00
updated_at: 2026-06-11T11:02:08.123938+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# TS SDK: remove retry from non-idempotent POSTs

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

TS wraps non-idempotent POSTs in `this.retry`: `submitManifests` (client.ts:169-174), apply's create-stack (:205-210) and create-deployment-object (:238-243). A lost response (proxy 502/504 after the broker committed) double-submits a revision — spurious agent redeploy — or duplicate-creates a stack. Rust (wrapper.rs:344-353, 396-454) and Python issue single attempts, and all three wrappers' own docs say not to retry non-idempotent operations.

## Acceptance Criteria

- [ ] The three POST sites call `this.api.POST` directly (single attempt), surfacing errors via `BrokkrError.fromOpenapiFetch`/`fromResponse` as elsewhere.
- [ ] Reads inside `apply` (verify_pak, list stacks, list deployment objects) keep their existing behavior; decide and document whether POST /auth/pak counts as idempotent (it is a pure read — keeping retry there is fine if documented).
- [ ] Unit tests assert no retry on the create paths (mock a 502: exactly one request observed).
- [ ] Contract suite green.

## Status Updates

*To be added during implementation*
