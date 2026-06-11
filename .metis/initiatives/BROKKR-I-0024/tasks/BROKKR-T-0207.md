---
id: broker-route-blanket-500-dal-sites
level: task
title: "Broker: route blanket-500 DAL sites through from_diesel (15 sites)"
short_code: "BROKKR-T-0207"
created_at: 2026-06-11T11:02:07.878785+00:00
updated_at: 2026-06-11T11:02:07.878785+00:00
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

# Broker: route blanket-500 DAL sites through from_diesel (15 sites)

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Apply the add_label fix pattern (stacks.rs:557, BROKKR PR #44) to the 15 remaining sites where a blanket `ApiError::internal` masks a realistic constraint violation. UNIQUE → 409 `unique_violation`, FK → 422, via `ApiError::from_diesel` (error.rs:147).

## Acceptance Criteria

- [ ] Duplicate-entity sites → 409: `agents.rs:527` add_label (`UNIQUE (agent_id,label)`), `agents.rs:637-643` add_annotation (`UNIQUE (agent_id,key)`), `agents.rs:738-741` add_target (`UNIQUE (agent_id,stack_id)` — the idempotent re-target case), `templates.rs:440-443` add_label, `templates.rs:552-555` add_annotation.
- [ ] Rename-collision sites → 409: `stacks.rs:245-248` update_stack (`unique_stack_name`), `agents.rs:326-329` update_agent (`unique_agent_cluster`), `generators.rs:246-249` update_generator (`unique_generator_name`).
- [ ] Client-supplied-FK sites → 422: `agents.rs:450-453` create_event (FK deployment_objects), `health.rs:158-167` update_health_status batch, `work_orders.rs:229-248` create_work_order targeting (classify before wrapping in `targeting_failed`; keep the cleanup).
- [ ] Race-only: `templates.rs:214-217, 308-311` create_new_version → from_diesel (free improvement).
- [ ] Integration regression tests for the duplicate paths, mirroring `test_add_stack_label_duplicate_returns_409` (tests/integration/api/stacks.rs).
- [ ] SDK contract suites still green (the apply 409-tolerance paths must keep working).

## Implementation Notes

Coverage context from the audit: 129 `ApiError::internal` sites reviewed; the other ~103 are genuinely internal (reads, server-generated values, pre-validated constraints, count-based 404s) — do NOT convert them. Constraint locations: crates/brokkr-models/migrations/.

## Status Updates

*To be added during implementation*
