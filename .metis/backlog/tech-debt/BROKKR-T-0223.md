---
id: broker-stack-annotations-unique
level: task
title: "Broker: stack_annotations UNIQUE (stack_id, key) migration + 409 routing — deferred from T-0210"
short_code: "BROKKR-T-0223"
created_at: 2026-06-11T15:58:13.624382+00:00
updated_at: 2026-06-11T15:58:13.624382+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#task"
  - "#phase/backlog"


exit_criteria_met: false
---

# Broker: stack_annotations UNIQUE (stack_id, key) migration + 409 routing — deferred from T-0210

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

`stack_annotations` lacks `UNIQUE (stack_id, key)` (crates/brokkr-models/migrations/03_stacks/up.sql), unlike `agent_annotations` and `template_annotations`. Duplicate keys accumulate silently and template matching then sees an arbitrary one-of-N value. Split from [[BROKKR-T-0210]] because it's a schema migration that needs DB-verified up/down + a deterministic dedupe rule — best done as focused, separately-verified work.

## Backlog Item Details

### Type
- [x] Tech Debt (low severity)

## Acceptance Criteria

- [ ] New migration: dedupe existing `stack_annotations` rows on (stack_id, key) with a documented keep-rule (e.g. latest `created_at`/highest id wins), then `ALTER TABLE stack_annotations ADD CONSTRAINT unique_stack_annotation UNIQUE (stack_id, key)`. Down migration drops the constraint.
- [ ] `stacks.rs add_annotation` create call routed through `ApiError::from_diesel` (already the T-0207 pattern) so a duplicate now returns 409 unique_violation (currently it silently inserts a duplicate, since there's no constraint to violate).
- [ ] `angreal models migrations` (up + redo/down) passes; `angreal models schema` regenerated if needed.
- [ ] Integration test: adding the same stack annotation key twice returns 409.

## Implementation Notes

Couple the migration and the from_diesel routing — the 409 only works once the constraint exists. Verify on a real Postgres (the agent could not run `angreal models migrations` locally without docker; CI's broker integration suite applies migrations). Mind the dedupe keep-rule so template matching gets a stable value.

## Status Updates

*To be added during implementation*
