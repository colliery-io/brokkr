---
id: broker-webhook-delivery-path-has
level: task
title: "Broker webhook delivery path has the same unbounded reclaim shape on subscription fetch error"
short_code: "BROKKR-T-0307"
created_at: 2026-07-28T00:42:12.388484+00:00
updated_at: 2026-07-31T05:23:34.324963+00:00
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

# Broker webhook delivery path has the same unbounded reclaim shape on subscription fetch error

## Objective

Close the last arm of the reclaim-loop family on the **broker** delivery path, matching what BROKKR-T-0302 and BROKKR-T-0304 fixed on the agent path.

In `deliver_pending_webhooks` (`crates/brokkr-broker/src/utils/background_tasks.rs:344-365`), each already-claimed delivery fetches its subscription. The two failure arms are not symmetric:

```rust
Ok(None) => {
    // ... mark_dead(delivery.id, "Subscription not found");
    continue;                      // correct: terminal, row reaches `dead`
}
Err(e) => {
    error!("Failed to get subscription {} for delivery {}: {:?}", ...);
    continue;                      // no mark_dead, no attempt increment, no release
}
```

The `Err` arm `continue`s **after the delivery has already been claimed**. The row sits `acquired` until its TTL lapses, is released by `release_expired`, is re-claimed on the next poll, and fails again. `attempts` never increments, so it never reaches `dead`.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (no data loss; a persistent fetch failure burns poll cycles indefinitely and the delivery never resolves either way)

## 2026-07-29 — verified against the code

Confirmed by reading `background_tasks.rs:344-365`. The title was this ticket's only content — an unedited template otherwise — and the claim holds.

**This is the third in a family, and the last known one.** BROKKR-T-0302 fixed the two decryption arms on the agent path; BROKKR-T-0304 fixed the `subscription not found` and fetch-error arms, also on the agent path. Both of those tickets cite the **broker** path as the well-behaved contrast — T-0302 explicitly notes it "marks such a delivery `dead` on first touch with an audit row (`background_tasks.rs:356-370`)". That is true of the `Ok(None)` arm and not of the `Err` arm three lines below it, which is presumably how it was missed.

**Marking it `dead` is the wrong fix here**, and this is where it differs from its siblings. `Ok(None)` means the subscription is genuinely gone — terminal, so `dead` is right. `Err(e)` is a *database* failure: almost always transient (connection exhaustion, a restarting Postgres, a network blip). Killing a delivery because the broker briefly could not read its own subscription table would discard events for a reason that has nothing to do with the event or the subscriber.

The right shape is bounded retry: increment `attempts` so the existing retry/backoff machinery applies and the delivery eventually reaches `dead` on its own terms, or explicitly release the claim so it is retried without burning the TTL window. Whichever is chosen, the invariant to restore is the one the family shares — **no path may leave a claimed delivery untouched.**

Worth checking during the fix whether other arms in the same loop (`background_tasks.rs:371-460`) share the shape; the decrypt and send arms appear to mark dead or update correctly, but they were not audited as closely as the subscription fetch.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A subscription-fetch error on a claimed delivery either increments `attempts` or releases the claim — it never leaves the row untouched.
- [ ] A persistently failing fetch terminates (reaches `dead` via the normal attempt ceiling) rather than looping forever.
- [ ] A *transient* fetch error still delivers the webhook on a later poll — the fix must not turn a blip into a discarded event.
- [ ] The `Ok(None)` arm still marks dead immediately; its behaviour is correct and must not regress.
- [ ] The remaining arms in the same loop are audited for the same shape, and any found are fixed or explicitly recorded as correct.

## Status Updates

**2026-07-30 — FIXED** on branch `docs/tenancy-review-2026-07` (commit `737c7dd`). 531 integration tests pass, 151 unit (4 new). Details in the commit message and in BROKKR-T-0315's neighbours; this ticket was one of two unedited template shells whose titles carried the entire finding, and both turned out to be real.