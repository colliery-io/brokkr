---
id: test-release-expired-races-live-delivery-worker
level: task
title: "test_release_expired races the live delivery worker: #[serial] does not exclude the broker container"
short_code: "BROKKR-T-0324"
created_at: 2026-08-02T18:00:00+00:00
updated_at: 2026-08-02T18:00:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# test_release_expired races the live delivery worker: #[serial] does not exclude the broker container

## Objective

Make `dal::webhook_deliveries::test_release_expired` deterministic. It asserts an exact terminal status on a row that a live background worker is concurrently allowed to claim.

**It failed the `v0.9.1` release run** (531 passed, 1 failed), skipping e2e and every publish job. A re-run of the same commit was needed to ship. That is the cost: a flaky test in the release path turns a green release into a manual retry, and invites treating real failures as flake.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (test defect)

### Priority
- [x] P2 - Medium (no product defect; it intermittently blocks releases and erodes trust in a red run)

## 2026-08-02 — mechanism, confirmed

```
assertion `left == right` failed:
  our expired-claim delivery should be released back to pending
```

The sequence:

1. The test creates a delivery and claims it with a 0-second TTL, so it is immediately expired.
2. It calls `release_expired()`, returning the row to `pending`.
3. It asserts the row is `pending`.

Between 2 and 3, **the broker's webhook delivery worker can claim it** — flipping it to `acquired` and failing the assertion.

That worker is real and running: `.angreal/files/docker-compose.yaml` starts `broker: command: serve`, which spawns `start_webhook_delivery_task`, polling the same database every 5 seconds. Integration tests run against that stack with `--skip-docker`.

**`#[serial(webhook_queue)]` does not help.** It serializes this test against *other tests*; the broker container is not a test and does not take that lock. The exclusion the test believes it has does not extend to the process it is actually racing.

### This is the second race in the same test

The test already carries a comment describing the first one:

> `release_expired()` sweeps the GLOBAL expired set, so the returned *count* is not deterministic on a shared DB — an external delivery worker (or another queue test) can release our row first, making this call return 0 even though the sweep works. Asserting `released >= 1` on that count is what flaked in CI. The race-free invariant is checked below…

So a previous fix correctly identified that an external worker interferes, dropped the count assertion, and then labelled the remaining status assertion "the race-free invariant". **It is not race-free** — it is the same interference one step later. Worth noting because the comment actively reassures the next reader that the remaining assertion is safe.

### Not caused by BROKKR-T-0307

Checked, because that change touched this file's subject matter. T-0307 altered the *subscription-fetch error* arm to call `mark_failed`; it does not make the worker claim more eagerly, and the failing path involves no fetch error. The race predates it.

## Options

1. **Assert the transition, not the terminal state.** The invariant that actually holds is "the row no longer carries its *expired* claim" — either it is `pending`, or something legitimately re-claimed it with a *fresh* `acquired_until`. Accept both, and assert the old claim is gone (`acquired_until` in the future, or `acquired_by` changed).
2. **Take the row out of the worker's reach.** Give the test's subscription an event type no real subscription matches, or mark the delivery so `claim_for_broker` skips it. Keeps the strict assertion, at the cost of testing a slightly artificial row.
3. **Do not run a broker alongside DAL tests.** Cleanest in principle, largest change — the compose stack is shared by everything.

Option 1 is the smallest honest fix: it tests what `release_expired` guarantees, rather than what nothing guarantees.

## Acceptance Criteria

- [ ] The test no longer asserts an exact status that a concurrent worker may legitimately change.
- [ ] Whatever it asserts is stated in terms of `release_expired`'s actual guarantee, with the concurrency spelled out in a comment.
- [ ] The existing "race-free invariant" comment is corrected — it currently tells the next reader the opposite of the truth.
- [ ] Sibling tests in `dal/webhook_deliveries.rs` are checked for the same shape; `#[serial]` was trusted to exclude the worker once and may be elsewhere.
- [ ] Run the integration suite several times in a row and record that it is stable, rather than passing once and declaring it fixed.

## Status Updates

*To be added during implementation*
