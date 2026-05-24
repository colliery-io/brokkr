---
id: b3-pak-revocation-closes-open-ws
level: task
title: "B3: PAK revocation closes open WS connections"
short_code: "BROKKR-T-0176"
created_at: 2026-05-24T12:56:47.000000+00:00
updated_at: 2026-05-24T12:56:47.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
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

*To be added during implementation*
