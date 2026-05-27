---
id: flaky-test-release-expired-webhook
level: task
title: "Flaky test_release_expired: webhook-delivery queue tests race on the shared DB"
short_code: "BROKKR-T-0184"
created_at: 2026-05-27T00:00:00+00:00
updated_at: 2026-05-27T00:00:00+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: NULL
---

# Flaky test_release_expired: webhook-delivery queue tests race on the shared DB

## Objective

`dal::webhook_deliveries::test_release_expired` flaked in CI (the v0.5.0
Release run, 2026-05-27), failing `assert!(released >= 1)` at
`webhook_deliveries.rs:278` while passing locally and on the PR. Make it
deterministic.

## Root cause

`TestFixture::new()` connects every test to a **single shared database**, and
cargo runs integration tests in parallel. The webhook-delivery queue
operations are **global**, not subscription-scoped:

- `release_expired()` sweeps *all* expired-claim rows.
- `claim_for_broker(N, …)` / `claim_for_agent(…)` claim from the *global*
  pending pool.

So a concurrently-running webhook test can release or steal another test's
rows. `test_release_expired` claims its row with TTL 0, sleeps 100 ms, then
calls `release_expired()` and asserts it released ≥ 1 — but a parallel test's
`release_expired()` could already have swept that row, so the count comes back
0. Product code is correct; the test isolation is wrong.

## Fix

Serialize the webhook-delivery DAL tests against each other with
`#[serial(webhook_queue)]` (serial_test, already a workspace dep). They run
one-at-a-time relative to each other — eliminating the shared-queue
contention — while still parallelising against the rest of the suite. Added
`serial_test` to `brokkr-broker` dev-dependencies and a module comment
explaining why.

## Acceptance Criteria

## Acceptance Criteria

- [x] `test_release_expired` (and the sibling claim/release tests) no longer
      race on the shared DB
- [x] `#[serial(webhook_queue)]` on all webhook-delivery DAL tests; module
      comment explains the shared-global-queue hazard
- [x] Integration suite green

## Status Updates

### 2026-05-27 — Fixed

Serialized the 21 webhook-delivery DAL tests. Compiles clean; webhook module +
full integration suite green (recorded in the fix commit). No product change.