---
id: add-stack-id-to-deployment-applied
level: task
title: "Add stack_id to deployment.applied/failed webhook payloads so stack_id filters and consumers can use them"
short_code: "BROKKR-T-0305"
created_at: 2026-07-27T23:08:58.565436+00:00
updated_at: 2026-07-27T23:08:58.565436+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Add stack_id to deployment.applied/failed webhook payloads so stack_id filters and consumers can use them

## Objective

Put `stack_id` into the `deployment.applied` and `deployment.failed` webhook payloads at the emission site (`crates/brokkr-broker/src/api/v1/agents.rs::create_event`), where the deployment object is already in hand.

Those two events currently carry only `deployment_object_id`. Because webhook filtering (BROKKR-T-0288 item 1) uses the rule "an event that does not carry the filtered field never matches", a subscription filtered by `stack_id` silently receives no apply/failure notifications — arguably the two events an operator most wants scoped to a stack. The same gap affects any consumer trying to correlate an apply with its stack without a second API call.

Identified while implementing filter evaluation; deliberately not fixed there because `api/v1/agents.rs` was outside that workstream's file ownership.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (filters work as designed; this widens what they can usefully scope)

### Business Justification
- **User Value**: `stack_id` filters become useful for the deployment lifecycle, and consumers can correlate an apply to a stack directly from the payload.
- **Effort Estimate**: S

### Technical Approach

Deliberately rejected during the filter work: resolving `stack_id` lazily at match time inside `emit_event`. That would put a DAL round trip per subscription on the write path (`emit_event` runs synchronously inside `DAL::create`/`soft_delete` and the agent-event POST handler, which is hot for every apply report from every agent), and it would make the filter rule inconsistent — one event pair would resolve a field it does not carry while `workorder.*` and `agent.*` would not, so operators could no longer reason from the payload alone.

Fixing it at the emission site is one lookup that serves all subscriptions, and it happens where the deployment object is already being handled.

Note the soft-delete case: by the time an apply is reported the deployment object may already be gone, so `stack_id` may legitimately be unavailable. Emit JSON `null` in that case rather than omitting the key — the filter predicate already treats null as absent (`payload_uuid_eq`), so this stays consistent with `deployment.deleted`, which already emits a nullable `stack_id`.

## Acceptance Criteria

- [ ] `deployment.applied` and `deployment.failed` payloads carry `stack_id` (nullable when the object is no longer resolvable).
- [ ] A subscription filtered by `stack_id` receives apply and failure events for that stack, covered by an integration test.
- [ ] The existing test pinning the current exclusion (`test_webhook_filter_stack_id_excludes_deployment_applied`) is updated rather than deleted, so the change in behavior is explicit in the diff.
- [ ] The per-event-type filter table in `docs/src/reference/webhooks.md` is updated to match.

## Status Updates

*To be added during implementation*
