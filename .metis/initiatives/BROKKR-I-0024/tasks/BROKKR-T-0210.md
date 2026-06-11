---
id: broker-input-validation-and-stack
level: task
title: "Broker: input validation and stack_annotations unique index"
short_code: "BROKKR-T-0210"
created_at: 2026-06-11T11:02:08.021805+00:00
updated_at: 2026-06-11T11:02:08.021805+00:00
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

# Broker: input validation and stack_annotations unique index

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Small validation and schema gaps from the sweep.

## Acceptance Criteria

- [ ] Webhooks: `create_webhook` (`webhooks.rs:309-311`) and `update_webhook` accept client-supplied `timeout_seconds`/`max_retries` with no validation and no DB CHECK; `test_webhook` (`:663`) does `timeout_seconds as u64`, so a negative sign-extends to an absurd timeout. Validate `>= 1` (and a sane upper bound) at create/update; clamp at use.
- [ ] `list_audit_logs` (`admin.rs:342-343`): `limit` clamped only by `.min(1000)` — `limit=-1` reaches Postgres (`LIMIT must not be negative`) → 500. Use `.clamp(1, 1000)`; clamp `offset` to `>= 0` (cf. `clamp_limit`, stacks.rs:934).
- [ ] Migration: `stack_annotations` lacks `UNIQUE (stack_id, key)` (migrations/03_stacks/up.sql:103-109), unlike `agent_annotations`/`template_annotations` — duplicate keys accumulate silently and template matching sees arbitrary one-of-N values. Add the unique index (dedupe existing rows in the migration), route `stacks.rs add_annotation` (:663-666) through `from_diesel` → 409.
- [ ] `agents.rs:425` `serde_json::to_value(e).unwrap()` in list_events → return typed `Json(events)` or map the error.
- [ ] Migration up/down verified via `angreal models migrations`; integration tests for the new 409 and the validation rejections.

## Implementation Notes

The annotation dedupe needs a deterministic keep-rule (e.g. latest `created_at` wins) — document it in the migration comment.

## Status Updates

*To be added during implementation*
