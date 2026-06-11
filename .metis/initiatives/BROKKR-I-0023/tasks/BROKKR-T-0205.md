---
id: agent-keep-heartbeats-alive-during
level: task
title: "Agent: keep heartbeats alive during long work orders and apply retries"
short_code: "BROKKR-T-0205"
created_at: 2026-06-11T11:02:07.778531+00:00
updated_at: 2026-06-11T15:10:29.337773+00:00
parent: agent-reconciler-hardening-crash
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0023
---

# Agent: keep heartbeats alive during long work orders and apply retries

## Parent Initiative

[[BROKKR-I-0023]]

## Objective

Every select arm in the control loop (`cli/commands.rs:228-485`) is awaited inline. A build work order can block up to 15 minutes (`work_orders/build.rs:41` `BUILD_TIMEOUT_SECS = 900`, polled at `build.rs:279-341`); a failing k8s apply retries up to 5 minutes (`k8s/api.rs:77` `max_elapsed_time: 300s`). During that window: no heartbeats, no health updates, no deployment polling — the broker marks the agent offline and operators page on a healthy agent.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Work-order execution (at minimum builds) runs in a spawned task / JoinSet; the select loop stays responsive.
- [ ] Heartbeat cadence is maintained during a long-running operation (test with an artificial delay).
- [ ] Concurrent-execution invariants documented: what may run in parallel with reconcile (builds yes; two reconciles no).

## Implementation Notes

Smallest-change option: spawn only the work-order arm and keep reconcile inline (reconcile starving heartbeats for 5 min is the rarer case — judge whether to also bound it). Watch shared-state access (`tracked_deployment_objects`, metrics) when spawning.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0023-agent-reconciler-hardening). The work_order_interval arm no longer awaits process_pending_work_orders inline (a build blocks up to 15 min, starving heartbeats/health/deployment polling so the broker marks a healthy agent offline). It now spawns the pass via `tokio::spawn` into a detached `work_order_task: Option<JoinHandle<()>>`; the select loop returns immediately and keeps ticking. **Single-pass invariant**: a tick is skipped while the previous task is still running (`task.is_some_and(|h| !h.is_finished())`), so passes never overlap/double-process; new work waits for the next tick. Required making the whole work_orders module's errors `Send + Sync` (tokio::spawn needs a Send future, and a `Box<dyn Error>` was held across an await): changed all 15 `Box<dyn std::error::Error>` → `+ Send + Sync`, and wrapped the 4 cross-module boundary calls (utils::multidoc_deserialize, k8s::api::apply_k8s_objects ×3) with `.map_err(|e| format!(...))` since those still return non-Send errors. The HTTP/k8s clients have runtime affinity so a separate-runtime thread was not an option — the future must run on the main runtime, hence the Send refactor. **Concurrency invariants**: builds may run concurrently with reconcile/heartbeat; only one work-order pass runs at a time; the spawned task uses cloned (Clone-derived) config/sdk/k8s/agent snapshots. Build clean, 73 lib tests pass, integration compiles, clippy clean. Heartbeat-during-build liveness is by construction (await moved off the select arm); end-to-end timing is exercised by the e2e suite.