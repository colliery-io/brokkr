---
id: b3-followup-agent-stops-hammering-ws-on-auth-rejection
level: task
title: "B3-follow-up: agent stops hammering WS after repeated auth rejection"
short_code: "BROKKR-T-0182"
created_at: 2026-05-26T00:00:00+00:00
updated_at: 2026-05-26T00:00:00+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# B3-follow-up: agent stops hammering WS after repeated auth rejection

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Follow-up to [[BROKKR-T-0176]] (B3). The broker now force-closes an agent's WS
when its PAK is revoked. But the agent's `reconnect_loop` treats the resulting
`401/403` on the next upgrade like any other dial failure: it warns and retries
forever on the backoff schedule (capped at 60s). A decommissioned agent
therefore tight-loops a failing handshake every ~60s indefinitely, spamming
logs and burning cycles on a credential that will never work again (the same
PAK gates REST, so the agent is fully locked out — there's nothing to recover).

Make the agent recognise a **permanent auth rejection** and stop hammering.

## Acceptance Criteria

- [x] `reconnect_loop` tracks consecutive auth-rejection dial failures
      (HTTP 401/403 on the WS upgrade) separately from transient failures
- [x] After a small threshold (5) of consecutive auth rejections, the agent
      logs a clear fatal-level message ("broker rejected the agent PAK N times
      — credential likely revoked; stopping WS reconnect") and stops the
      reconnect loop instead of retrying forever
- [x] A new terminal `WsState::AuthRejected` is published so other components
      can distinguish "credential dead" from ordinary `Down`
- [x] A transient (non-auth) failure, or any successful connect, resets the
      counter — a blip never trips the permanent-stop path
- [x] Unit test(s) covering: auth-rejection detection from the tungstenite
      error, and the counter/threshold logic
- [x] No regression in existing broker_ws tests (backoff schedule, etc.)

## Implementation Notes

### Technical Approach

- `tokio_tungstenite::tungstenite::Error::Http(resp)` carries the handshake
  response; `resp.status()` is `401`/`403` for an auth rejection. A small
  `is_auth_rejection(&Error) -> bool` helper keeps the loop readable.
- Threshold of 5 keeps a one-off (e.g. a broker restart racing a token refresh)
  from tripping it, while still stopping promptly for a real revocation.
- Stopping = `return` from `reconnect_loop` after publishing
  `WsState::AuthRejected`. The agent's REST layer fails independently and
  surfaces its own auth error; the WS layer's job is to not hammer.

### Risk Considerations

- Don't trip on transient `Down`. Only HTTP 401/403 count; dial timeouts,
  connection-refused, etc. are transient and reset the counter.

## Status Updates

### 2026-05-26 — Done

`reconnect_loop` (`broker_ws.rs`) now counts consecutive WS-upgrade auth
rejections via `is_auth_rejection(&Error)` (matches `Error::Http` with status
401/403). After `MAX_CONSECUTIVE_AUTH_REJECTIONS = 5` it logs an `error!`
("…credential is almost certainly revoked. Stopping WS reconnect. Provision a
fresh PAK and restart the agent."), publishes the new terminal
`WsState::AuthRejected`, and returns from the loop. Any successful connect or
any non-auth (transient) dial error resets the counter, so a blip or a broker
restart never trips the permanent stop.

`WsState::AuthRejected` is additive — callers only ever matched `.is_up()`, so
nothing breaks; components can now distinguish "credential dead" from ordinary
`Down`.

**Tests:** `auth_rejection_detects_401_and_403_only` (401/403 → true; 500/503
and transport errors → false) + existing backoff tests unchanged. Agent unit
suite 65 passed.

This pairs with [[BROKKR-T-0176]]: the broker closes the socket on revoke, and
now the agent recognises the resulting permanent 401 and stops hammering
instead of retrying the dead credential every ~60s forever.
