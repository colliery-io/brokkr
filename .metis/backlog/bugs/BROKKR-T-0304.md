---
id: agent-webhook-path-subscription
level: task
title: "Agent webhook path: 'subscription not found' and fetch-error arms leak the same unbounded reclaim loop"
short_code: "BROKKR-T-0304"
created_at: 2026-07-27T19:32:42.345754+00:00
updated_at: 2026-07-28T00:42:11.094402+00:00
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

# Agent webhook path: 'subscription not found' and fetch-error arms leak the same unbounded reclaim loop

## Objective

Finish closing the reclaim-loop class of bug in `get_pending_agent_webhooks` (`crates/brokkr-broker/src/api/v1/webhooks.rs`). BROKKR-T-0302 fixed the two decryption arms; two sibling arms in the same loop still `continue` **after** the delivery has been claimed:

- `Ok(None)` — subscription not found (around line 812). Identical unbounded cycle: the row sits `acquired`, its 60s TTL lapses, `release_expired` frees it, the next poll re-claims it, and it fails again — with `attempts` pinned at 0 so it never reaches `dead`. The broker delivery path marks this case dead; the agent path does not.
- `Err(e)` — subscription fetch failure. Same shape. Note this one may be a transient database error rather than a permanent condition, so it likely deserves ordinary retry semantics (increment attempts, honour `max_retries`) rather than immediate death — decide deliberately rather than copying the not-found treatment.

Deliberately scoped out of BROKKR-T-0302, whose acceptance criteria covered decryption only.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (a deleted subscription with in-flight deliveries produces a permanently cycling row; no data loss or security impact)

### Impact Assessment
- **Reproduction**: create a subscription with a pending agent-targeted delivery, hard-delete the subscription, then poll as the agent. The delivery cycles claimed → expired → claimed indefinitely with `attempts` never incrementing.

### Technical Approach
`fail_delivery_undecryptable` (added by BROKKR-T-0302) is already the right shape for the not-found arm — generalise its name and reason parameter rather than duplicating it. Roughly a two-line change for the not-found case; the fetch-error case needs the retry-vs-dead decision above.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A delivery whose subscription no longer exists reaches a terminal state instead of cycling.
- [ ] The transient fetch-error case has a deliberate, documented policy (retry with attempts vs immediate death) and follows it.
- [ ] Integration coverage for the not-found case, in the shape of the T-0302 tests.
- [ ] The helper is shared, not copy-pasted, across all four arms.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`.

`fail_delivery_undecryptable` generalised to **`fail_claimed_delivery(dal, delivery, reason, disposition)`** with a `DeliveryDisposition { Terminal, Retryable { max_retries } }` enum — `Terminal` → `mark_dead`, `Retryable` → `mark_failed`. Shared across all four arms as required. The decrypt metric moved out of the helper to the two decryption call sites, since it is decryption-specific and the helper no longer is; that mirrors how the broker path does it. Both BROKKR-T-0302 call sites keep their distinct `last_error` strings and terminal outcome.

**Fetch-error decision: retryable, not terminal.** A `get(subscription_id)` failure is a database error that says nothing about this delivery — the subscription is almost certainly still there next poll, so killing it on one blip would silently discard a valid webhook. But bare `continue` was not an option either, since the row is already claimed and would cycle with `attempts` never moving. `mark_failed` counts the attempt, releases the claim, schedules backoff, and bounds the cycle by the retry budget. Uses a `FALLBACK_MAX_RETRIES = 5` constant mirroring the schema default, because the subscription's own `max_retries` is precisely what could not be read. Nice property recorded in the code: if the database is down hard the `mark_failed` write fails too, the row falls back to TTL recovery with `attempts` unmoved, and it self-heals when the DB returns.

**The ticket's reproduction is WRONG and the severity was generous.** `webhook_deliveries.subscription_id` is `NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE`, so hard-deleting a subscription deletes its deliveries too — there is no orphan row and the poll returns nothing. The `Ok(None)` arm is only reachable in a narrow race: the delete landing *between* `claim_for_agent` (which returns rows already in memory) and the per-delivery fetch in the same poll. The fix is still correct and worth having, but this is a rare race rather than the routine cycle described.

Test: `test_pending_agent_webhooks_missing_subscription_marks_delivery_dead`, shaped on the T-0302 model including the second poll that proves the loop is over. It has to manufacture the state via `delete_subscription_without_cascade`, which suppresses the FK trigger with `SET LOCAL session_replication_role = replica` inside one transaction (reverts at commit, so the pooled connection is returned unmodified). **This assumes a superuser test role** — true under the angreal compose file; if that ever changes the helper fails loudly at the `SET LOCAL` rather than silently passing. No test for the fetch-error case: forcing a DAL error needs a database fault with no injection seam, and every method of forcing one is process- or schema-global and would corrupt the other tests sharing the DB.

**Follow-up filed as BROKKR-T-0307:** the identical `Err(e) => continue` shape exists on the *broker* delivery path in `utils/background_tasks.rs`, outside this ticket's file ownership.