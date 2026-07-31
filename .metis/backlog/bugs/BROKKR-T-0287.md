---
id: stack-matching-ignores
level: task
title: "Stack matching ignores registration: unregistered agents receive any tenant's stacks via label/annotation match, contradicting security-hardening.md"
short_code: "BROKKR-T-0287"
created_at: 2026-07-27T14:27:47.760452+00:00
updated_at: 2026-07-27T18:02:04.630605+00:00
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

# Stack matching ignores registration: unregistered agents receive any tenant's stacks via label/annotation match, contradicting security-hardening.md

## Objective

Investigate and resolve a tenant-isolation gap surfaced by the 2026-07-27 review. `docs/src/how-to/security-hardening.md` claims "An unregistered agent can only serve system/fleet-scoped stacks." Reviewer finding (accuracy pass against `crates/brokkr-broker/src/utils/matching.rs`): stack-to-agent association is the OR-union of agent labels, annotations, and explicit targets, with **no registration check** on the label/annotation paths — registration gates only explicit target creation (403 `agent_not_registered`). An agent never registered with tenant B's generator still receives tenant B's stacks if labels/annotations happen to match.

This intersects directly with the current tenant-isolation work: label/annotation matching is a cross-tenant delivery channel that bypasses the registration subscription model of PRs #79/#80.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (isolation semantics; needs a code-vs-docs decision)

### Priority
- [x] P0 - Critical (potential cross-tenant workload delivery; at minimum a false isolation claim)

### Impact Assessment
- **Reproduction sketch**: create tenant-B stack with label `env=prod`; give an agent registered only to tenant A (or to nothing) the matching label; observe the agent's deployment-object poll include tenant B's stack.
- **Decision needed**: (a) enforce registration as a filter in the matching union (behavioral change, aligns with I-0030 subscription model), or (b) keep matching as-is and rewrite security-hardening.md + multi-tenancy docs to state plainly that labels/annotations cross tenant boundaries.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Reproduction confirmed or refuted with an integration test (`angreal tests integration brokkr-broker`). **CONFIRMED 2026-07-27**: `registration_consent` filter run → 1 passed (registered path), 2 failed (label + annotation legs) — an unregistered generator's colliding stack appears in the agent's target state. Consent tests now `#[ignore]`-tagged (compile-verified) pending the fix; un-ignore them when enforcement lands.
- [x] Decision (code fix vs docs fix) recorded here; if code, registration gates all three match paths and a regression test pins it; if docs, security-hardening.md and reference/multi-tenancy.md describe the real boundary. **Fix implemented 2026-07-27** (see Status Updates); full-suite verification in flight.
- [x] Either way, the false sentence in security-hardening.md is gone.

## Status Updates

**2026-07-27** — Docs-side interim landed: security-hardening.md now states that registration gates explicit-target creation while label/annotation matching delivers stacks independently of registration, with tenant-distinct-selector guidance (phrased to survive a future code tightening); "Configuring Agent Scopes" steps corrected to match.

**Citation correction from verification**: the agent-facing matching union lives in `dal/deployment_objects.rs:283-326` (and `dal/stacks.rs:320-375`), not `utils/matching.rs` (that file is template-to-stack matching). Use those paths for the reproduction test.

Open: the code decision (gate label/annotation matching on registration vs. accept and document) and the integration-test reproduction — both still needed before this closes.

**2026-07-27 — DECISION (Dylan): option 2 — registration is the consent boundary for ALL association paths.** Semantics: a generator says "these are the labels I push to"; an agent's registrations say "these are the generators I accept stacks from". Label/annotation matching must select *within* registered generators, never across them. Supporting facts: agent labels/annotations are admin-only (`require_admin`, agents.rs) while stack labels belong to the stack's owner (`fetch_owned_stack`, stacks.rs) — so the exposure is a tenant-chosen stack label colliding with an admin-applied agent label.

**Repro/regression tests added**: `tests/integration/api/registration_consent.rs` — positive test (registered generator's labeled stack delivered) plus two consent tests (label + annotation legs) asserting an unregistered generator's colliding stack must NOT appear in GET /agents/:id/target-state. Consent tests expected red against current code; will be `#[ignore]`-tagged with this ticket until the enforcement lands, then un-ignored as the regression guard. Enforcement site: registration filter on the label/annotation legs of the union in `dal/deployment_objects.rs` (target state + `pending_counts_by_agent`) and `dal/stacks.rs::get_associated_stacks` — keep all mirrors consistent.

**2026-07-27 — FIX IMPLEMENTED.** Two sites (the target-state path inherits the first, so there were two real mirrors, not three):
1. `dal/stacks.rs::get_associated_stacks` — loads the agent's registered generator ids once, then filters the label-matched and annotation-matched stack sets to `registered_generators.contains(&s.generator_id)`. Explicit targets are left unfiltered (already gated at creation with 403 `agent_not_registered`). An agent with no registrations now gets nothing from the matching legs. `get_target_state_for_agent` delegates here, so the agent-facing endpoint is covered automatically.
2. `dal/deployment_objects.rs::pending_counts_by_agent` — the set-based mirror gains an `inner_join` on `agent_generator_registrations` (agent_id + stacks.generator_id) for both the label and annotation legs, keeping its documented "exactly mirrors get_target_state_for_agent(...).len()" contract true.

Doc comments on both functions now state the consent rule. Regression tests un-ignored. Clippy clean (remaining warnings are pre-existing `sort_by` nits on untouched lines).

**VERIFIED — full broker integration suite green: 488 passed, 0 failed** (`angreal tests integration brokkr-broker`, exit 0). All four `registration_consent` tests pass.

**Pre-existing tests updated (5 assertions across 4 tests)** — each created a stack owned by a fresh test generator, gave the agent a matching label/annotation, and asserted delivery *without any registration*, i.e. they encoded the leak. Each now calls `register_agent_with_generator(...)`, which is what a real operator must do under the consent rule:
- `dal::stacks::test_get_associated_stacks` (agent1 and, further down, agent4 in the annotations-only case)
- `dal::deployment_objects::test_target_state_label_targeting_after_deployment_exists`
- `dal::deployment_objects::test_target_state_annotation_targeting_after_deployment_exists`
- `api::agents::test_get_agent_stacks`
Target-only cases were untouched and still pass, confirming explicit targets remain exempt as designed.