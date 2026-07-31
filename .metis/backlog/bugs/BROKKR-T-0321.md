---
id: agent-inactive-is-agent-side-not-broker-enforced
level: task
title: "Docs say the broker withholds work from INACTIVE agents; the agent withholds it from itself"
short_code: "BROKKR-T-0321"
created_at: 2026-07-30T06:00:00+00:00
updated_at: 2026-07-30T06:00:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Docs say the broker withholds work from INACTIVE agents; the agent withholds it from itself

## Objective

Correct where `status: INACTIVE` is enforced, and decide whether it should also be enforced broker-side.

Two getting-started pages state that the **broker** gates delivery on agent status:

- `getting-started/evaluate.md:63` — "A freshly registered agent starts with `status` `INACTIVE` — **the broker only hands it deployment objects once you mark it `ACTIVE`**."
- `getting-started/installation.md:130` — "every new agent starts inactive and **applies nothing until you activate it**."

The **observable behaviour matches** — an INACTIVE agent does apply nothing — but the enforcement point is the opposite of what is described.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (documentation correctness; possibly behaviour)

### Priority
- [x] P2 - Medium (no live exposure by itself, but it misdescribes an access boundary on the primary onboarding path)

## 2026-07-30 — verified

Found while deciding what to do about the console's Live/Paused toggle (BROKKR-T-0319).

**The gate is agent-side.** `crates/brokkr-agent/src/cli/commands.rs:427` skips deployment-object fetches and `:545` skips work-order processing when `agent.status != "ACTIVE"`. The agent re-reads its own record each heartbeat (`:409-412`), so a status change lands within a poll cycle.

**The broker does not gate on it at all.** `agent.status` is read in exactly two places server-side: the `brokkr_active_agents` gauge (`api/v1/agents.rs:136`) and serialization into audit/event payloads. `get_associated_stacks` and the target-state path have no status filter — the broker will serve deployment objects to an INACTIVE agent that asks.

So `INACTIVE` is **agent-side self-restraint, not access control**. A modified, replaced, or simply older agent binary that does not implement the check will be served normally. The second doc sentence ("applies nothing until you activate it") is true of the shipped agent; the first ("the broker only hands it...") is false about the mechanism, and it is the one a reader would rely on when reasoning about the trust boundary.

Worth noting the default makes this load-bearing: the column defaults to `INACTIVE` (`migrations/01_agents/up.sql:10`), so *every* agent depends on this check from creation until an operator activates it.

## Decisions needed

1. **Fix the docs regardless** — say plainly that the agent honours its own status and the broker does not enforce it, so nobody treats `INACTIVE` as a containment control.
2. **Decide whether the broker should also enforce it.** Adding the filter to the served-stack union would make the docs' original claim true and turn a convention into a boundary. It pairs naturally with the registration-consent work (BROKKR-T-0287), which put the equivalent check on the read path for generators. Against: it is a behaviour change that could surprise anyone relying on an agent applying work while marked INACTIVE — though that combination is not something the docs ever sanctioned.

## Acceptance Criteria

- [ ] Both getting-started statements describe the actual enforcement point.
- [ ] `reference/` and `explanation/security-model.md` state whether agent status is a boundary or a convention — currently neither mentions it.
- [ ] A decision is recorded on broker-side enforcement, with reasoning, whether or not it is implemented.
- [ ] If implemented, a test asserts an INACTIVE agent is served no deployment objects even when it asks directly.

## Status Updates

*To be added during implementation*
