---
id: diagnostic-requests-stuck-in
level: task
title: "Diagnostic requests stuck in 'claimed' forever when an agent dies mid-collection (never expired, never cleaned up)"
short_code: "BROKKR-T-0300"
created_at: 2026-07-27T19:10:49.702549+00:00
updated_at: 2026-07-28T00:42:10.246665+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Diagnostic requests stuck in 'claimed' forever when an agent dies mid-collection (never expired, never cleaned up)

## Objective

Close a permanent stuck-state and row leak in the diagnostics lifecycle. `expire_old_requests()` transitions only `pending → expired`, and `cleanup_old_requests()` deletes only `expired|completed|failed` (`crates/brokkr-broker/src/dal/diagnostic_requests.rs:171-215`, driven by `crates/brokkr-broker/src/utils/background_tasks.rs:46-90`). So a request an agent has **claimed** and then failed to submit — because the agent crashed, was evicted, lost its PAK, or its pod was rescheduled — sits in `claimed` forever: never expired, never failed, never deleted.

This is precisely the case a terminal failure state should cover, and it is an argument for keeping the `failed` status that BROKKR-T-0291 was otherwise considering deleting from the docs.

Found 2026-07-27 during the T-0291/T-0275 investigation.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (unbounded row growth is slow; the stuck state is invisible rather than damaging)

### Impact Assessment
- **Reproduction**: create a diagnostic request, let an agent claim it, kill the agent before it submits a result. The row remains `claimed` indefinitely; nothing reaps it.
- **Consequences**: monotonic row growth in `diagnostic_requests`; an operator watching the request sees it hang with no terminal state and no error.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Stale `claimed` requests past `expires_at` are swept to a terminal state (`expired`, or `failed` if that path is wired per BROKKR-T-0291) — minimum viable fix is one predicate change in `expire_old_requests()`.
- [ ] `cleanup_old_requests()` reaps whatever terminal state the sweep produces.
- [ ] Integration test: claim a request, advance past expiry, assert it reaches a terminal state and is eventually cleaned up.
- [ ] `docs/src/reference/diagnostics.md` state machine reflects the claimed-expiry path.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`.

**Decision: sweep abandoned claims to `failed`, not `expired`.** The two answer different operator questions and collapsing them destroys the only signal this sweep produces. `expired` means nobody picked the work up — the agent is offline or not polling, routine and often expected. `failed` means an agent *accepted* the request and did not survive it — crash, eviction, lost PAK, reschedule. A run of `failed` diagnostics against one agent is evidence about that agent's stability; a run of `expired` ones is evidence it isn't connected. Secondary: this gives `failed` a real producer, settling the BROKKR-T-0291 question in the direction this ticket's own filing argued for; the DB `CHECK` constraint and `VALID_STATUSES` already permit it, so no migration; and `cleanup_old_requests()` already reaped `failed`, so criterion 2 came free.

Cost stated plainly: this is **not** the "one predicate change" the ticket proposed. Distinguishing the outcomes needs two UPDATEs and a richer return type (`ExpirySweep { expired, failed }`). Judged worth it for the operator signal. The two counts log differently — `info!` for routine expiries, `warn!` for abandoned claims. `claimed_at` is preserved on the swept row so the operator can still see which agent took the work.

Tests: `test_expire_sweeps_abandoned_claimed_request_to_failed` (asserts the counts, the stamped `completed_at`, preserved `claimed_at`, and idempotency on a second sweep), `test_cleanup_reaps_abandoned_claimed_request` (asserts cleanup deletes **0** while still `claimed` — proving the state was genuinely unreachable before — then reaps after the sweep), and `test_abandoned_claimed_diagnostic_reports_failed` (API-level, including that a resurrected agent's late `POST /result` is refused with 409). `test_expire_old_requests` updated in place.

**Docs corrected in both directions.** `reference/diagnostics.md` had stated `failed` was never emitted and told readers to treat it as reserved — now false and removed; the lifecycle diagram, status table, and Collection Errors section all distinguish "agent reported an error" from "agent stopped answering". `how-to/diagnostics.md` carried the same claim in three places, including a troubleshooting entry steering operators *away* from the exact signal this ticket creates; corrected here as part of the task rather than left to drift.

**Deliberately not done:** no grace period for claims — a request claimed near its expiry gets swept almost immediately. Kept the deadline the operator already chose via `retention_minutes` rather than inventing a second timeout; if that proves too eager, a distinct claim timeout is the right fix, not a fudge factor. Also noted: the claim endpoint still doesn't reject already-expired requests (pre-existing race, now self-correcting under the sweep), and no `brokkr_diagnostic_requests_abandoned_total` metric — the `warn!` is the signal today.