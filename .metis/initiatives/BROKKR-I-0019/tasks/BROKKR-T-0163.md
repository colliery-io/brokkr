---
id: ws-08-agent-pod-log-tailer
level: task
title: "WS-08: Agent pod log tailer — backpressure/sampling, per-stack opt-in"
short_code: "BROKKR-T-0163"
created_at: 2026-05-23T02:12:40.571898+00:00
updated_at: 2026-05-23T02:12:40.571898+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-08: Agent pod log tailer — backpressure/sampling, per-stack opt-in

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Tail container logs for pods in managed workloads. Per-stack opt-in via stack annotation (e.g. `brokkr.io/stream-logs: "true"`). Apply per-stream rate limiting / sampling and signal a `LogGap` when lines are dropped.

## Acceptance Criteria

- [ ] Log streaming is **off by default** for every stack; enabled only when the opt-in annotation is set
- [ ] Tails containers in pods belonging to opted-in stacks; emits `PodLogLine` messages with `{pod, container, ts, line}`
- [ ] Per-stream rate limiter (config'd default + override per stack) enforced; over-rate lines are dropped, not buffered indefinitely
- [ ] When lines are dropped, emit a `LogGap { dropped_count, since_ts }` marker so consumers know data is missing
- [ ] Container restarts resume tail at the new instance; pod removal stops the tail
- [ ] Integration test: opted-in stack produces logs visible at the broker; non-opted-in stack produces zero log messages

## Implementation Notes

- **Approach**: kube-rs `Pod::log_stream`; per-pod tasks; lines flow into a per-agent log channel with token-bucket rate limiting before WS send.
- **Dependencies**: WS-01, WS-03.
- **Risk**: a runaway log producer can saturate the WS channel. Rate limit must be enforced *agent-side*, before the WS send queue, and the priority writer (WS-02) ensures control plane is never starved even if logs are blocked.