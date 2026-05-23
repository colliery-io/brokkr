---
id: ws-11-live-fan-out-subscription
level: task
title: "WS-11: Live fan-out subscription (read-only WS/SSE) for UI tail — PAK-scoped"
short_code: "BROKKR-T-0166"
created_at: 2026-05-23T02:12:45.257748+00:00
updated_at: 2026-05-23T02:12:45.257748+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-11: Live fan-out subscription (read-only WS/SSE) for UI tail — PAK-scoped

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Add a read-only WS/SSE subscription endpoint on the broker that lets UI / SDK clients tail live events and logs for stacks. The broker fans out from an in-memory hub fed by ingestion (WS-09). PAK-scoped: subscribers receive only streams from stacks their PAK can read.

## Acceptance Criteria

- [ ] Endpoint accepts a subscription filtered by `stack_id` (or set of stack_ids)
- [ ] Only streams from stacks the PAK has read access to are delivered; unauthorized stacks are silently filtered
- [ ] Subscriber receives messages within RTT of broker ingestion (no extra DB round-trip)
- [ ] Clean disconnect: subscriber drop releases server resources
- [ ] Heartbeat / ping keeps idle subscriptions alive through proxies
- [ ] Integration test: subscribe via test client; agent emits an event; subscriber receives it

## Implementation Notes

- **Approach**: tokio broadcast channel per stack inside the broker; ingestion path fan-outs to (a) DAL insert (WS-09) and (b) the broadcast hub. Subscribers filter via the PAK scope at subscribe time.
- **Dependencies**: WS-09 (ingestion is the source of truth).
- **Risk**: a slow subscriber must not slow ingestion. Use a bounded per-subscriber buffer; drop subscribers that lag past the threshold.