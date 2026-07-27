---
id: agent-watch-batch-v1-job
level: task
title: "Agent: watch batch/v1 Job completion for custom work orders instead of reporting success at apply time"
short_code: "BROKKR-T-0303"
created_at: 2026-07-27T19:20:54.174359+00:00
updated_at: 2026-07-27T19:20:54.174359+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Agent: watch batch/v1 Job completion for custom work orders instead of reporting success at apply time

## Objective

Make "work order succeeded" mean the work actually completed, for the case where that is well-defined. Today `execute_custom_work_order` (`crates/brokkr-agent/src/work_orders/mod.rs:269-331`) server-side-applies the YAML and returns `Ok(Some("Successfully applied N resource(s)"))`, and `process_single_work_order` (`:192-224`) immediately completes the order with `success = true`. A migration Job that lands in `ImagePullBackOff` or exits non-zero is recorded as a success — exactly the signal operators build deployment gates on.

Split from BROKKR-T-0291 on 2026-07-27 per Dylan's decision to **implement monitoring before writing the docs**, so `explanation/work-orders.md` is written once against final behavior. That doc correction is blocked on this ticket.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (false success signal on the feature's documented headline use case)

### Business Justification
- **User Value**: `work_order_log` becomes a real audit trail rather than an apply log; CI/CD gates on work-order success stop being decorative.
- **Effort Estimate**: M

### Technical Approach

**Reuse the proven in-repo pattern, don't invent one.** `build::execute_build` already does exactly this shape for BuildRuns — discover API → poll status conditions on an interval → interpret → bounded timeout (`crates/brokkr-agent/src/work_orders/build.rs:103-195, 260-342`). Generalize that loop to a `Job`'s `status.succeeded` / `status.failed`.

Three constraints established during investigation:

1. **Bound the wait below the work order's claim timeout.** `claim_timeout_seconds` defaults to 3600 (`brokkr-models/src/models/work_orders.rs:158-160`) and the broker's maintenance task reclaims stale CLAIMED orders (`brokkr-broker/src/utils/background_tasks.rs:138-146`). A watch that outlives the claim gets the order re-dispatched to another agent **while the first Job is still running**. Builds sidestep this with a 15-minute constant well under the hour; derive the bound from `claim_timeout_seconds` minus a margin rather than hard-coding.
2. **`deployment_health.rs` is NOT reusable here.** It resolves ownership via `brokkr.io/deployment-object-id` (label → annotation → ownerReference chain), and custom work orders stamp **no Brokkr annotations at all** (`work_orders/mod.rs:290-325` builds DynamicObjects straight from user YAML). It is a pattern reference only. `execute_custom_work_order` already holds the applied `Vec<DynamicObject>`, so kind/namespace/name are in hand — no label plumbing needed.
3. **Kind-aware, with an apply-only fallback.** Only `batch/v1 Job` has unambiguous terminal semantics. `Deployment`, `CronJob`, `ConfigMap` either never "complete" or mean something different. Non-Job kinds stay apply-only and must say so in the result message. Do **not** attempt generic "is this resource healthy yet" monitoring — that is the deployment-object reconciler's job and does not belong in the work-order path.

Blocking is safe: work-order processing already runs in a spawned task, one order per pass (`brokkr-agent/src/cli/commands.rs:543-570`, `.take(1)` at `work_orders/mod.rs:146`), and builds already block for up to 15 minutes this way.

## Acceptance Criteria

- [ ] A custom work order containing a `batch/v1 Job` is completed with `success = false` and a useful reason when the Job fails or times out, and `success = true` only when it reports succeeded.
- [ ] The watch is bounded below `claim_timeout_seconds` so the stale-claim reaper cannot re-dispatch a still-running order; timeout is reported distinguishably from failure.
- [ ] Non-Job kinds remain apply-only and their result message says so explicitly.
- [ ] Integration coverage: succeeding Job, failing Job, and a timeout.
- [ ] `docs/src/explanation/work-orders.md:139` rewritten against the shipped behavior, **preserving the build/custom distinction** — the build path's "watches to completion" claim at :102 is already accurate and must not be flattened. (This closes the last open item of BROKKR-T-0291.)

## Status Updates

*To be added during implementation*
