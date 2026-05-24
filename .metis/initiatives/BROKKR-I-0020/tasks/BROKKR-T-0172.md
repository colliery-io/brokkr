---
id: a3-real-k3s-test-for-kube-events
level: task
title: "A3: Real-k3s test for kube-events + pod-logs tailers"
short_code: "BROKKR-T-0172"
created_at: 2026-05-24T12:56:39.000000+00:00
updated_at: 2026-05-24T12:56:39.000000+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# A3: Real-k3s test for kube-events + pod-logs tailers

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Prove the kube-events tailer (WS-07) and pod-logs tailer (WS-08) actually
emit when a real Kubernetes cluster does. The existing unit tests cover
the `UidCache` and `RateLimiter` in isolation; this task wires both through
a real k3s cluster, a real failing pod, and a real noisy pod, and asserts
rows land in `agent_k8s_events` / `agent_pod_logs` (and that opt-out
annotation is honored).

## Acceptance Criteria

- [ ] New k3s test invokable via `angreal helm test` (extend the existing harness
      with a `--scenario telemetry` flag or add a separate task)
- [ ] Test deploys a stack annotated with `k8s.brokkr.io/stack` and
      `brokkr.io/stream-logs: "true"`
- [ ] Forces a pod failure (e.g. `image: does-not-exist:latest`); asserts a
      `Warning` event with `reason=Failed` (or `ImagePullBackOff`) lands in
      `agent_k8s_events` within 30s
- [ ] Drives a pod that logs > 100 lines/sec for ≥ 5s; asserts a `LogGap{RateLimit}`
      row appears in `agent_pod_logs`
- [ ] Deploys a second stack WITHOUT the `brokkr.io/stream-logs: "true"`
      annotation; asserts zero rows for that stack in `agent_pod_logs`
      (opt-in enforcement)
- [ ] Deploys a third stack WITHOUT the `k8s.brokkr.io/stack` annotation
      on the pod; asserts no events captured for it (managed-object filter
      enforcement)
- [ ] Test runs green in helm-test CI

## Implementation Notes

### Technical Approach

- Reuse the existing `angreal helm test` k3s spin-up; the broker + agent
  already deploy via helm chart. Only the test scenario file changes
- For the "100 lines/sec" pod, a simple busybox `while true; do echo ...; done`
  with a small `sleep` is enough — don't overcomplicate
- Query the broker via the REST history endpoints (`GET /stacks/{id}/events`
  and `GET /stacks/{id}/logs`) rather than going directly to Postgres —
  exercises the full ingestion path

### Dependencies

None directly, though shares the broader hardening goal of [[BROKKR-T-0170]]
/ [[BROKKR-T-0171]]. Can run in parallel with the A1/A2 work.

### Risk Considerations

- k3s test takes longer; may need to bump CI timeouts
- Event arrival is asynchronous and depends on k8s reconcile loops; use
  polling-with-timeout (e.g. 30s deadline) rather than fixed sleep

## Status Updates

*To be added during implementation*
