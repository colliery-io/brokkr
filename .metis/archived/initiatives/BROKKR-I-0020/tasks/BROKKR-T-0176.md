---
id: b3-pak-revocation-closes-open-ws
level: task
title: "B3: PAK revocation closes open WS connections"
short_code: "BROKKR-T-0176"
created_at: 2026-05-24T12:56:47+00:00
updated_at: 2026-05-24T12:56:47+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# B3: PAK revocation closes open WS connections

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Current behavior: PAK auth is checked once at WS upgrade. If an admin
revokes that PAK afterward, the existing WS connection stays open until
the agent's TCP socket dies — a real security gap for any "rotate the
agent PAK" incident-response workflow. Wire the PAK invalidation path
to close the matching WS socket.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ConnectionRegistry` gains a `close_for_agent(agent_id) -> usize`
      method that drops the connection handle and triggers per-connection
      cancel-signal close
- [ ] The PAK invalidation path (admin DELETE on a PAK, or whichever
      handler invalidates an agent PAK) calls `close_for_agent` after
      the DB delete commits
- [ ] New broker integration test: agent connects via WS → admin
      revokes the agent's PAK → asserts the socket closes within 1s
      and `brokkr_ws_connected_agents` decrements
- [ ] No regression in existing WS tests (connection lifecycle, normal
      disconnect, reconnect)
- [ ] Operator note added to `docs/src/explanation/internal-ws-channel.md`
      documenting the new behavior

## Implementation Notes

### Technical Approach

- Look at `crates/brokkr-broker/src/dal/agents.rs` or the PAK DAL for
  the existing invalidation path; trace upward to the HTTP handler
- The cancel-signal pattern already exists per the WS-03 work — reuse it
- For agents with multiple connections (multi-instance agents are rare
  but possible in theory), `close_for_agent` should close all of them.
  Return the count for logging

### Dependencies

None.

### Risk Considerations

- The close-on-revoke should NOT race with normal reconnect. The agent
  will try to reconnect after close; the next upgrade will re-check the
  (now-invalid) PAK and reject with 401 — that's the intended flow,
  but verify the agent doesn't tight-loop on reconnect attempts after
  hitting a permanent 401
- If the close path is invoked from within a DB transaction, defer it
  until after commit to avoid holding locks while doing socket I/O

## Status Updates

### 2026-05-26 — Implemented + green

**Mechanism:** no new cancel-token needed — the per-connection writer task
already exits when both its lane receivers close (`handler.rs` writer_task:
`let Some(msg) = next else { close socket; return }`). The lane *senders* live
only in the registry's `ConnectionHandle`. So dropping the handle from the
registry map IS the close signal: writer sees both lanes closed → closes the
socket → `run_connection`'s `select!` completes → connected-gauge decrements.

**Changes:**
- `ConnectionRegistry::close_for_agent(agent_id) -> usize` — removes the
  handle (drops its senders), returns 0/1. Map is one-connection-per-agent
  (`register` evicts a prior handle), so the count is 0 or 1 today; signature
  returns a count for forward-compat + logging.
- `rotate_agent_pak` and `delete_agent` handlers now take the
  `ConnectionRegistry` extension and call `close_for_agent(id)` **after** the
  DB commit + `invalidate_auth_cache` (never holding DB locks across socket
  teardown, per the risk note).
- Operator note added to `docs/src/explanation/internal-ws-channel.md`
  ("Credential revocation closes the socket") + tweaked the long-lived-
  connection caveat to mention revocation.

**Tests (all green):**
- Unit: `registry::tests::close_for_agent_removes_handle_and_drops_senders`.
- Integration: `rotating_agent_pak_closes_its_open_ws` and
  `deleting_agent_closes_its_open_ws` — connect agent WS, admin
  rotates/deletes, assert the registry clears AND the client socket observes
  the close within 1s.
- No regression: full `api::ws` module 18 passed (was 16 + these 2).

**Reconnect-after-401 risk (from the task):** the agent's reconnect uses
exponential backoff 1s→60s with jitter (documented in the WS channel doc's
Recovery semantics), so a permanent 401 after revocation backs off rather than
tight-looping. No code change needed; behavior confirmed against the existing
backoff. The deeper "agent should give up / surface a fatal auth error after N
401s" is a possible agent-side polish but out of scope here (the broker-side
security gap — the whole point of B3 — is closed).