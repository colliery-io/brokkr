---
id: broker-fix-path-body-auth
level: task
title: "Broker: fix path/body auth mismatches and event listing scope"
short_code: "BROKKR-T-0208"
created_at: 2026-06-11T11:02:07.925054+00:00
updated_at: 2026-06-11T11:02:07.925054+00:00
parent: broker-api-correctness-error
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0024
---

# Broker: fix path/body auth mismatches and event listing scope

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Several handlers authorize against the path id but act on a body id that is never compared (or skip scoping entirely). Verified worst case: `create_event` (`agents.rs:441-453`) authorizes `require_admin_or_agent(&auth_payload, id)` for the **path** id but inserts `new_event.agent_id` from the **body** unchecked — an agent PAK can attribute events to any other agent, and these feed the webhook event bus.

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
