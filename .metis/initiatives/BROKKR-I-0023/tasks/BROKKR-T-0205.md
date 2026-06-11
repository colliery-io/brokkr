---
id: agent-keep-heartbeats-alive-during
level: task
title: "Agent: keep heartbeats alive during long work orders and apply retries"
short_code: "BROKKR-T-0205"
created_at: 2026-06-11T11:02:07.778531+00:00
updated_at: 2026-06-11T11:02:07.778531+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: keep heartbeats alive during long work orders and apply retries

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Every select arm in the control loop (`cli/commands.rs:228-485`) is awaited inline. A build work order can block up to 15 minutes (`work_orders/build.rs:41` `BUILD_TIMEOUT_SECS = 900`, polled at `build.rs:279-341`); a failing k8s apply retries up to 5 minutes (`k8s/api.rs:77` `max_elapsed_time: 300s`). During that window: no heartbeats, no health updates, no deployment polling — the broker marks the agent offline and operators page on a healthy agent.

## Acceptance Criteria

- [ ] Work-order execution (at minimum builds) runs in a spawned task / JoinSet; the select loop stays responsive.
- [ ] Heartbeat cadence is maintained during a long-running operation (test with an artificial delay).
- [ ] Concurrent-execution invariants documented: what may run in parallel with reconcile (builds yes; two reconciles no).

## Implementation Notes

Smallest-change option: spawn only the work-order arm and keep reconcile inline (reconcile starving heartbeats for 5 min is the rarer case — judge whether to also bound it). Watch shared-state access (`tracked_deployment_objects`, metrics) when spawning.

## Status Updates

*To be added during implementation*
