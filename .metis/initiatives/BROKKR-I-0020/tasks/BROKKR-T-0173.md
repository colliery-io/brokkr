---
id: a4-push-poll-race-test-for-target
level: task
title: "A4: Push/poll race test for target_changed pushes"
short_code: "BROKKR-T-0173"
created_at: 2026-05-24T12:56:41.000000+00:00
updated_at: 2026-05-24T12:56:41.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# A4: Push/poll race test for target_changed pushes

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Close the post-commit push race that ADR-0008 flagged as a known risk: a
REST GET of an agent's targets concurrent with a WS `target_changed` push
for the same agent. Asserts the agent state machine doesn't double-reconcile
and the push isn't silently dropped because of a competing read lock or
stale-data race.

## Acceptance Criteria

- [ ] New broker integration test in `crates/brokkr-broker/tests/`
      exercising concurrent REST GET `/agents/{id}/targets` + WS push of
      `target_changed{agent_id, stack_id}` for the same agent
- [ ] Test runs N concurrent iterations (N ≥ 50) with the order of GET/push
      randomized per iteration
- [ ] Asserts every push is delivered (not silently dropped) — verifiable
      via the broker-side `brokkr_ws_messages_total{direction="downlink",type="target_changed"}`
      counter incrementing by N
- [ ] Asserts no duplicate target row is created (the post-push REST GET
      returns the same target list as the WS push payload references)
- [ ] Runs under `angreal tests integration --crate brokkr-broker` and stays
      green in CI

## Implementation Notes

### Technical Approach

- Use the existing broker integration-test harness (real Postgres via
  testcontainers, real axum server)
- Spawn a test WS client that captures `target_changed` frames; spawn a
  concurrent REST GET in a `tokio::spawn`
- The race window we care about: the broker writes to the `agent_targets`
  table → the post-commit `push_target_changed` fires. A REST GET that
  lands between commit and push must still see the new target

### Dependencies

None.

### Risk Considerations

- This is a narrow race test, not a chaos test. Keep it tight and
  deterministic-where-possible — flaky integration tests waste more time
  than they catch bugs
- If we find the race actually exists (i.e. push gets dropped under load),
  the fix is likely to wrap registry send in a retry loop or move the push
  to inside the transaction — both decisions worth a quick design check
  before implementing

## Status Updates

*To be added during implementation*
