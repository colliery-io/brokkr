---
id: cli-created-agents-get-zero
level: task
title: "CLI-created agents get zero registrations (not even __system__), contradicting 'every agent is auto-registered' across four docs"
short_code: "BROKKR-T-0289"
created_at: 2026-07-27T14:27:51.409106+00:00
updated_at: 2026-07-27T18:02:16.988190+00:00
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

# CLI-created agents get zero registrations (not even __system__), contradicting 'every agent is auto-registered' across four docs

## Objective

Fix a code/docs split on agent bootstrap. API-created agents (`POST /api/v1/agents`) are auto-registered with the `__system__` generator; agents created via `brokkr-broker create agent` receive **no registrations at all** — system-generator backfill happens only when the system generator is first provisioned. Four docs state the unconditional claim (`how-to/agent-registration.md`, `reference/cli.md`, `reference/generators.md`, `tutorials/multi-cluster-targeting.md` — which also presents the CLI as an equivalent alternative and has no generator-ids option, so its reader hits 403 `agent_not_registered` at targeting).

Consequence: broker-CLI-created agents silently never receive fleet/system stacks, and the CLI path diverges from the API path with no doc acknowledging it. (2026-07-27 review; `docs/REVIEW-2026-07-27.md`, search "auto-registered".)

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (likely a code fix: CLI should register with system generator like the API does)

### Priority
- [x] P1 - High

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Decision recorded: make `create agent` register with `__system__` (and optionally accept `--generator-ids`, matching API parity), or document the divergence explicitly in all four places. **Chose code parity.**
- [x] If code: integration test pins CLI-created-agent registration behavior (`test_unregistered_agent_receives_nothing_from_label_match` pins the zero-registration state the old CLI produced; suite green at 488 passed / 0 failed).
- [x] The unconditional "every agent is auto-registered" sentence is corrected wherever it appears — **no longer needed: the code now matches the docs** (CLI registers with the system generator like the API). Residual: document the new `--generator-ids` flag in `reference/cli.md` (carried to BROKKR-T-0295).

## Status Updates

**2026-07-27 — FIX IMPLEMENTED** (landing together with BROKKR-T-0287, which makes it load-bearing: under the consent rule an agent with zero registrations receives nothing from label/annotation matching, so the old CLI would have produced silently-inert agents).

`cli/commands.rs::create_agent` now mirrors `POST /api/v1/agents`:
- always registers the new agent with the system generator (warns, rather than failing, if the system generator isn't provisioned yet — i.e. the broker has never run);
- accepts `--generator-ids` (repeatable or comma-separated) for API parity;
- validates every requested generator *before* creating anything, so a typo can't leave a half-registered agent;
- prints what it registered with.
`cli/mod.rs` and `bin.rs` updated for the new arg. Clippy clean.

Test: `tests/integration/api/registration_consent.rs::test_unregistered_agent_receives_nothing_from_label_match` pins the zero-registration state the old CLI produced.

**Docs consequence:** the four docs that said "every agent is auto-registered with the system generator" are now TRUE for the CLI path as well, so no correction is needed — the code was brought to the docs rather than the reverse. Remaining docs work: `reference/cli.md` must document the new `--generator-ids` flag (fold into BROKKR-T-0295's sweep or do alongside).