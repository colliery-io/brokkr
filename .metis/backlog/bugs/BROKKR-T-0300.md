---
id: diagnostic-requests-stuck-in
level: task
title: "Diagnostic requests stuck in 'claimed' forever when an agent dies mid-collection (never expired, never cleaned up)"
short_code: "BROKKR-T-0300"
created_at: 2026-07-27T19:10:49.702549+00:00
updated_at: 2026-07-27T19:10:49.702549+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


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

- [ ] Stale `claimed` requests past `expires_at` are swept to a terminal state (`expired`, or `failed` if that path is wired per BROKKR-T-0291) — minimum viable fix is one predicate change in `expire_old_requests()`.
- [ ] `cleanup_old_requests()` reaps whatever terminal state the sweep produces.
- [ ] Integration test: claim a request, advance past expiry, assert it reaches a terminal state and is eventually cleaned up.
- [ ] `docs/src/reference/diagnostics.md` state machine reflects the claimed-expiry path.

## Status Updates

*To be added during implementation*
