---
id: ws-03-agent-ws-client-connect
level: task
title: "WS-03: Agent WS client — connect, reconnect/backoff, REST fallback path, force-REST config flag"
short_code: "BROKKR-T-0158"
created_at: 2026-05-23T02:12:32.745717+00:00
updated_at: 2026-05-23T02:12:32.745717+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-03: Agent WS client — connect, reconnect/backoff, REST fallback path, force-REST config flag

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Implement the WebSocket client side in `brokkr-agent`: dial the broker WS endpoint, authenticate with the agent's PAK, run read/write loops, reconnect with exponential backoff + jitter on disconnect, and continue REST polling while disconnected. Add a `--force-rest` config flag for environments where WS must not be used.

## Acceptance Criteria

- [ ] Agent connects to broker WS endpoint on startup using PAK
- [ ] Receives + dispatches typed `WsMessage` variants
- [ ] On disconnect, exponential backoff (capped, with jitter) controls reconnect attempts
- [ ] While disconnected, the agent's existing REST polling continues to run
- [ ] `force_rest_only` config flag (env + CLI) prevents any WS dial attempt
- [ ] Unit test: backoff schedule is sane; integration test: reconnects after broker restart

## Implementation Notes

- **Approach**: a long-lived task in the agent runtime owning the WS connection; a watch/mpsc channel publishes connection state ("WS-up" / "WS-down") so other agent components can decide whether to use WS or REST.
- **Dependencies**: WS-01. Unblocks WS-04, WS-05, WS-06.
- **Risk**: REST polling must not double-process a message that also arrived via WS — idempotency on the agent's work-order handler is required (likely already true since REST polling is idempotent today; verify).