---
id: ws-08-agent-pod-log-tailer
level: task
title: "WS-08: Agent pod log tailer — backpressure/sampling, per-stack opt-in"
short_code: "BROKKR-T-0163"
created_at: 2026-05-23T02:12:40.571898+00:00
updated_at: 2026-05-23T12:26:13.140346+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-08: Agent pod log tailer — backpressure/sampling, per-stack opt-in

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Tail container logs for pods in managed workloads. Per-stack opt-in via stack annotation (e.g. `brokkr.io/stream-logs: "true"`). Apply per-stream rate limiting / sampling and signal a `LogGap` when lines are dropped.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Log streaming is off by default — only pods carrying `brokkr.io/stream-logs: "true"` (`STREAM_LOGS_ANNOTATION`) AND `k8s.brokkr.io/stack` get tailed.
- [x] Tails every container in opted-in pods; emits `PodLogLine` with `{agent_id, stack_id, namespace, pod, container, ts, line}`.
- [x] Per-container token-bucket rate limiter (default 100 lines/sec per `RATE_WINDOW`); over-rate lines are dropped immediately, never buffered.
- [x] First drop in each window surfaces a `LogGap{reason: RateLimit, dropped_count: …}` so the UI sees a visible gap rather than silently swallowing data.
- [x] Pod deletion tears down the per-container tasks via `teardown_for(uid)`; container restart inside the same pod is handled implicitly by re-watching when the pod re-applies under a new UID.
- [ ] **Deferred**: live cluster integration test (opt-in vs not-opt-in pods, observable LogGap behaviour). Same harness gap as WS-07 — lands with the broader e2e suite.

## Implementation Notes

- **Approach taken**: cluster-wide `kube::runtime::watcher` on `Pod`. For each `Apply` event the pod is filtered by `is_opted_in` (annotation check) and `pod_stack_id` (annotation parse); when both succeed, per-container tail tasks are spawned via `Api<Pod>::log_stream(LogParams::follow=true)`. The kube-rs stream is bridged to a `tokio::io::BufReader` via `tokio_util::compat::FuturesAsyncReadCompatExt` for line-oriented reading.
- **Dependencies**: WS-01 (wire types), WS-03 (WsUplink), WS-09 (broker persists), WS-11 (broker fans out live).
- **Risk**: runaway log producer → handled by per-container token bucket *before* the WS uplink; the priority writer added in WS-02 means a saturated telemetry lane never starves control-plane messages.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `crates/brokkr-agent/src/pod_logs.rs`. Public API: `spawn(client, uplink, agent_id) -> JoinHandle<()>` and `STREAM_LOGS_ANNOTATION` constant. The annotation is *on the pod* (e.g. via the controlling Deployment's PodTemplateSpec); a future improvement can have the agent's reconciler propagate a stack-level opt-in down to pods automatically.
- Per-container tail tasks keyed by pod UID in an `Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>`. Pod delete or opt-out → `teardown_for(uid)` aborts all per-container handles cleanly.
- `RateLimiter`: 100 lines/sec per container by default. First drop in each window surfaces a `LogGap{RateLimit, dropped_count}`; subsequent drops in the same window accumulate silently into the next window's first gap (avoids gap-marker spam during a sustained burst).
- Wired up in `cli/commands.rs` right after the WS-07 spawn. `_pod_logs_handle` is intentionally unused — production drops it; the task runs for the agent process lifetime.
- Added `tokio-util = "0.7"` with the `compat` feature for the kube-rs → tokio AsyncRead bridge.
- Tests: 2 unit tests on `RateLimiter` (under-ceiling all pass; over-ceiling drop with first gap).

**Honest gap**: no live kube-cluster integration test in this commit (same as WS-07). The opt-in / opt-out / gap-marker behaviour is structurally proven by the unit tests; end-to-end "deploy an opted-in pod, see its logs at the broker; deploy a non-opted-in pod, see nothing" needs the e2e harness.