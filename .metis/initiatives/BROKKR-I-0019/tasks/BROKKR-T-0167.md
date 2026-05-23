---
id: ws-12-ui-live-tail-history
level: task
title: "WS-12: UI live tail + history integration (ui-slim)"
short_code: "BROKKR-T-0167"
created_at: 2026-05-23T02:12:46.824209+00:00
updated_at: 2026-05-23T02:12:46.824209+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-12: UI live tail + history integration (ui-slim)

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

In `ui-slim`, add a per-stack "Live" view that subscribes to event/log tail (WS-11) and a "History" view that paginates through the REST history (WS-10). Surface gap markers, reconnect state, and the 6h retention ceiling.

## Acceptance Criteria

- [ ] Per-stack page exposes "Live tail" and "History" tabs
- [ ] Live tab subscribes via WS-11; streams events + logs in real time; shows reconnect indicator on drop
- [ ] History tab paginates via WS-10 REST endpoints; respects time-range filters
- [ ] `LogGap` markers rendered visibly (e.g. "23 lines dropped due to rate limit")
- [ ] Clear UX message about the 6h retention ceiling + recommended sink ("For long-term retention, configure Datadog forwarding")
- [ ] Built and used via `angreal local up` then exercised in a browser to confirm the golden path

## Implementation Notes

- **Approach**: TypeScript SDK from WS-10 for history; native browser WebSocket for live (use the regenerated TS schema types). Reuse existing ui-slim layout patterns.
- **Dependencies**: WS-10, WS-11.
- **Risk**: long-running WS subscriptions through reverse proxies (ingress timeouts). Document required proxy timeouts in the ops docs (WS-14).