---
id: ws-07-agent-kube-events-tailer
level: task
title: "WS-07: Agent kube Events tailer — filter to managed objects, stream over WS"
short_code: "BROKKR-T-0162"
created_at: 2026-05-23T02:12:39.003941+00:00
updated_at: 2026-05-23T02:12:39.003941+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-07: Agent kube Events tailer — filter to managed objects, stream over WS

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

In the agent, tail the Kubernetes `Events` API filtered to objects the agent deployed (via owner reference / managed-by label), and stream them upstream as `K8sEvent` WS messages.

## Acceptance Criteria

- [ ] Watcher subscribes to `Events` filtered to managed objects only (no general cluster event shipping per initiative non-goals)
- [ ] Resumes from last seen `resourceVersion` on reconnect (no duplicate events on restart)
- [ ] Emits typed `K8sEvent` messages containing event reason, message, involved-object ref, timestamp
- [ ] Stops streaming for objects no longer managed (after stack delete / target removal)
- [ ] Integration test: deploy a workload that produces a known Event (e.g. ImagePullBackOff), assert the broker receives it

## Implementation Notes

- **Approach**: kube-rs `watcher` against the Events API; filter in-process by owner-ref UID set, which is kept in sync by the agent's reconciler.
- **Dependencies**: WS-01, WS-03.
- **Risk**: events for an object can arrive *before* the agent has registered the owner UID. Buffer briefly or re-check after a small delay.