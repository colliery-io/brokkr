---
id: build-work-orders-use-a-fixed-15
level: task
title: "Build work orders use a fixed 15-minute watch not bounded by claim_timeout_seconds, so a short claim timeout re-dispatches mid-build"
short_code: "BROKKR-T-0310"
created_at: 2026-07-28T16:08:33.729209+00:00
updated_at: 2026-07-28T16:08:33.729209+00:00
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

# Build work orders use a fixed 15-minute watch not bounded by claim_timeout_seconds, so a short claim timeout re-dispatches mid-build

## Objective

Bound the BuildRun watch by the work order's own claim timeout, the way BROKKR-T-0303 just did for `batch/v1` Jobs.

`build::execute_build` watches a BuildRun with its own hard-coded ~15-minute constant (`crates/brokkr-agent/src/work_orders/build.rs`). That is well under the 3600s default `claim_timeout_seconds`, so the common case is safe — which is presumably why it has never bitten. But the bound is unrelated to the order's actual claim timeout: a work order created with `claim_timeout_seconds < 900` has its claim expire while the first agent is still watching, the broker's maintenance task reclaims it, and a second agent re-dispatches the build **while the first is still running**.

Same class of bug BROKKR-T-0303 fixed for Jobs, and the fix has the same shape: derive the budget from the claim timeout rather than a constant.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (unreachable at the default claim timeout; requires an order created with a short one)

### Technical Approach

`job_watch_budget(claim_timeout_seconds)` already exists in `work_orders/mod.rs` from BROKKR-T-0303 — `claim_timeout − max(10% of claim_timeout, 60s)`. Reuse it rather than inventing a second policy; two different watch-budget formulas in one module is how they drift.

**Critical interaction, do not miss it:** BROKKR-T-0303 found that `is_error_retryable` classifies any error message containing "timeout" as retryable, so reporting a watch timeout the obvious way makes the broker re-dispatch the order — causing exactly the double-execution the bound exists to prevent. `WorkOrderOutcomeError { message, retryable }` and the explicit downcast in `process_single_work_order` exist for this reason. The build path must use the same mechanism, not the text classifier.

Note the build watch should probably take `min(15 minutes, budget)` rather than simply adopting the budget — a build genuinely has an upper bound worth keeping independent of a very long claim timeout.

## Acceptance Criteria

- [ ] The BuildRun watch is bounded by both its own ceiling and the order's `claim_timeout_seconds`, using the shared budget helper.
- [ ] A build watch timeout is reported non-retryable via `WorkOrderOutcomeError`, not through the text classifier, and says the BuildRun was not cancelled.
- [ ] Unit coverage that a short `claim_timeout_seconds` shortens the build watch.
- [ ] `docs/src/explanation/work-orders.md` — the build path's watch-window description is updated if the ceiling changes; the build/custom contrast added by BROKKR-T-0303 must stay accurate.

## Status Updates

*To be added during implementation*
