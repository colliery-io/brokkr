---
id: agent-webhook-path-subscription
level: task
title: "Agent webhook path: 'subscription not found' and fetch-error arms leak the same unbounded reclaim loop"
short_code: "BROKKR-T-0304"
created_at: 2026-07-27T19:32:42.345754+00:00
updated_at: 2026-07-27T19:32:42.345754+00:00
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

- [ ] A delivery whose subscription no longer exists reaches a terminal state instead of cycling.
- [ ] The transient fetch-error case has a deliberate, documented policy (retry with attempts vs immediate death) and follows it.
- [ ] Integration coverage for the not-found case, in the shape of the T-0302 tests.
- [ ] The helper is shared, not copy-pasted, across all four arms.

## Status Updates

*To be added during implementation*
