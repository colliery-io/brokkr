---
id: console-per-agent-pause-resume
level: task
title: "Console: make pause actually work, per-agent and admin-authorized, and drop the global toggle"
short_code: "BROKKR-T-0322"
created_at: 2026-07-31T01:00:00+00:00
updated_at: 2026-07-31T01:00:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Console: make pause actually work, per-agent and admin-authorized, and drop the global toggle

## Objective

> *"regardless of the mechanism - can we make the pause button actually WORK from the UI"* — Dylan, 2026-07-31

Replace the inert global `Live/Paused` header control with a working per-agent pause in the Fleet modal, authorized by an admin PAK supplied per action.

Implements the decision recorded in BROKKR-T-0319, which established that the header toggle should be removed rather than relocated, and that a real pause belongs per-agent with a target and a credential.

## Backlog Item Details

### Type
- [x] Feature - New functionality

### Priority
- [x] P2 - Medium (removes a control that misrepresented fleet state, and makes the capability real)

## 2026-07-31 — no broker changes needed

The capability already existed; nothing in the console reached it.

| Piece | Where |
|---|---|
| Writable state | `PUT /api/v1/agents/{id}` sets `status` (`api/v1/agents.rs:398`), `require_admin_or_agent` |
| Partial update | the handler applies only fields present, so sending `status` alone cannot clobber `name`/`cluster_name` |
| Agent honours it | `brokkr-agent/src/cli/commands.rs:427` skips deployment-object fetches, `:545` skips work orders |
| Takes effect live | the agent re-reads its own record each heartbeat (`commands.rs:409-412`) |

The console's injected token is `admin: true, readonly: true`, so the readonly middleware rejects the PUT before the handler — which is exactly why the operator supplies an admin PAK for the one request, the same shape as tenant minting (BROKKR-T-0318).

## Acceptance Criteria

- [x] The global `Live/Paused` control and its `live` signal are gone from `app.rs` and `Main`.
- [x] The Fleet agent modal can pause and resume that agent, authorized by a per-action admin PAK.
- [x] The control states plainly that pausing does not roll back already-applied resources.
- [x] The admin PAK is memory-only and cleared on both the success and error paths.
- [x] The modal reflects the new state immediately rather than waiting for the fleet refetch.
- [x] Harness scenes cover the control and the paused state, with the no-persistence assertion.
- [x] Docs describe the new control and that pausing is agent-side self-restraint, not broker enforcement.

## Status Updates

**2026-07-31 — DONE.** clippy clean on `wasm32-unknown-unknown` (zero warnings), `trunk build` passes, 25 harness scenes green with both no-persistence assertions passing.

**Built:** `api::put_json_with_token` (mirroring the POST variant — explicit bearer, never stored) and `api::set_agent_status`. The Fleet modal gained an **Agent state** section above diagnostics: current-state copy, a `PasswordInput` for the admin PAK, and a button that reads **Pause** or **Resume** depending on state. Two new harness scenes, `fleet-pause` and `fleet-paused`, the latter driven end to end through the PUT.

**The copy is the part worth keeping.** "Pausing does not roll anything back" is stated in the control itself, not just the docs. An operator reaching for pause during an incident is likely to want *undo*, and the two are easy to conflate — the control should say which one it is at the moment of use.

**Removed rather than relocated**, per T-0319. The header toggle drove nothing and read as fleet-wide; the docs had gone as far as calling it "decoration for now", which is not something to ship.

**Recorded honestly in the docs:** pausing is the agent honouring its own status, not the broker enforcing it — the broker will still serve a paused agent that asks. The how-to says so and warns against using pause as containment for an agent you do not trust. BROKKR-T-0321 tracks whether the broker should enforce it too; this ticket deliberately did not pre-empt that decision, because the UI is correct either way.

**Not done:** no integration test asserts the console's read-only credential is rejected on `PUT /agents/{id}`. The equivalent guard exists for tenant minting (`test_ui_pak_cannot_mint_a_generator`), and `test_ui_pak_cannot_mutate` already covers `PUT /agents/{id}` generically — so the behaviour is pinned, but not by a test named for this feature. Worth adding if the pause path grows.