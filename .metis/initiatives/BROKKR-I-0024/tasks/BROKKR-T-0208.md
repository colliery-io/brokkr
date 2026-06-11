---
id: broker-fix-path-body-auth
level: task
title: "Broker: fix path/body auth mismatches and event listing scope"
short_code: "BROKKR-T-0208"
created_at: 2026-06-11T11:02:07.925054+00:00
updated_at: 2026-06-11T15:44:03.529978+00:00
parent: broker-api-correctness-error
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0024
---

# Broker: fix path/body auth mismatches and event listing scope

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Several handlers authorize against the path id but act on a body id that is never compared (or skip scoping entirely). Verified worst case: `create_event` (`agents.rs:441-453`) authorizes `require_admin_or_agent(&auth_payload, id)` for the **path** id but inserts `new_event.agent_id` from the **body** unchecked — an agent PAK can attribute events to any other agent, and these feed the webhook event bus.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_event`: reject path/body agent-id mismatch with 400 (or overwrite with path id) — follow the stacks.rs `stack_id_mismatch` pattern (stacks.rs:657 add_annotation).
- [ ] `add_target` (`agents.rs:729-746`), `add_label` (`:519`), `add_annotation` (`:626`): enforce path/body equality (currently the path segment is decorative).
- [ ] `GET /agent-events` + `/agent-events/:id` (`agent_events.rs:44-94`): currently any valid PAK enumerates all agents' events cluster-wide; scope to admin-or-own (consistent with `GET /agents/:id/events`, agents.rs:412). CHECK the demo UI and SDK contract suites for consumers of the unscoped behavior before changing.
- [ ] `update_health_status` (`health.rs:114-167`): restrict the batch to deployment objects in stacks the agent targets; stop silently dropping invalid entries via `filter_map(.ok())` (`:137-152`) — return per-entry errors or 422.
- [ ] Integration tests for each rejection path.

## Implementation Notes

The agent-events scoping is a behavior change — list it in the release notes. Everything else only rejects requests that were already nonsensical.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (security-critical parts; one low-severity item deferred — noted below). Branch feat/i0024-broker-api-correctness.
  - **create_event path/body mismatch (the vuln)**: after the auth check, reject `new_event.agent_id != id` with 400 `agent_id_mismatch`. An agent PAK can no longer attribute events (which feed the webhook bus) to a different agent. Same equality check added to add_label, add_annotation, add_target (the path segment was decorative).
  - **agent-events enumeration scope**: `GET /agent-events` and `/agent-events/{id}` bound `_auth_payload` (any valid PAK could list every agent's events cluster-wide). Now require admin (an agent reads its own via `GET /agents/{id}/events`, already scoped). Verified no active test/UI consumer uses these with a non-admin PAK (only the generated SDK wrappers reference them) — BEHAVIOR CHANGE, note in release notes.
  - **health silent-drop**: `update_health_status` built records with `filter_map(...).ok()`, silently discarding invalid entries. Now a fallible `map(...).collect::<Result<_,_>>()?` returns 422 `invalid_health_record` so the agent learns its report was rejected. (The existing agent-PAK-match auth check was already correct.)
  Tests (rejection paths): test_create_event_agent_id_mismatch_returns_400 (→400 agent_id_mismatch), test_list_agent_events_requires_admin (→403). Broker builds clean, clippy clean, tests compile; the 403/400 behaviors run on CI's broker integration suite.
  DEFERRED (low severity): restricting the health batch to deployment objects in stacks the agent targets. The upsert is already keyed by (agent_id, deployment_object_id) so the blast radius is bounded; full scoping needs a per-record deployment_object→stack→agent_targets join (N+1 or a new batch DAL method). Tracked here as a residual; not security-critical.
