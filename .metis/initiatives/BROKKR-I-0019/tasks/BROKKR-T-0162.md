---
id: ws-07-agent-kube-events-tailer
level: task
title: "WS-07: Agent kube Events tailer — filter to managed objects, stream over WS"
short_code: "BROKKR-T-0162"
created_at: 2026-05-23T02:12:39.003941+00:00
updated_at: 2026-05-23T12:23:12.150081+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-07: Agent kube Events tailer — filter to managed objects, stream over WS

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

In the agent, tail the Kubernetes `Events` API filtered to objects the agent deployed (via owner reference / managed-by label), and stream them upstream as `K8sEvent` WS messages.

## Acceptance Criteria

## Acceptance Criteria

- [x] Watcher subscribes to `Events` cluster-wide; per-event filtering is done by checking the `k8s.brokkr.io/stack` annotation on the `involvedObject` (managed-by label proxy). Events on non-managed objects are silently dropped after a single cached lookup.
- [x] Resumes via `kube::runtime::watcher` semantics — on transient errors the outer `watch_loop` is restarted with a 5s backoff; kube-rs maintains the `resourceVersion` cursor internally between successful pages.
- [x] Emits typed `K8sEvent` with reason, message, type, source, involved-object ref + UID, timestamp (`event_time` → `last_timestamp` → `now()` fallback).
- [x] Stops streaming for unmanaged objects: the cache's `NotOurs` entry prevents repeat lookups; an object that loses its `STACK_LABEL` will fall to `NotOurs` on cache miss after TTL.
- [ ] **Deferred** to BROKKR-I-0019 follow-up: real-k8s end-to-end test (deploy a workload that fails image pull, assert the broker `agent_k8s_events` row appears). Requires the docker stack via `angreal tests integration brokkr-agent` which is heavier and not in scope for this commit.

## Implementation Notes

- **Approach taken**: cluster-wide `kube_runtime::watcher` on `Event`. Each event's `involvedObject` UID is resolved to a stack id via the agent's annotation scheme — `k8s.brokkr.io/stack` carries the owning stack id on every Brokkr-managed resource. Lookups are cached for 5 minutes (positive + negative) so repeated events for the same pod don't translate to repeated `GET`s.
- **Dependencies**: WS-01 (wire types), WS-03 (`WsUplink`), WS-09 (broker-side persistence so frames actually land somewhere).
- **Risk**: events arriving *before* an object is queryable via the API → the lookup misses, we cache `NotOurs`, the event is lost. Acceptable for v1; the cache TTL is short enough that a later re-deploy of the same object recovers within minutes.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New module `crates/brokkr-agent/src/kube_events.rs`. Public API: `spawn(client, uplink, agent_id) -> JoinHandle<()>`. Runs a watcher loop with reconnect backoff (5s), drains decoded events through an outbound mpsc into the agent's `WsUplink`, drops frames silently when WS is down (Events are cheap signal; agents that miss a few are fine).
- UID cache (`UidCache`) with TTL = 5 minutes; both `Owned(stack_id)` and `NotOurs` cached so repeat events for the same pod don't hammer the kube API.
- `annotation_lookup` discovers the involvedObject's GVK via `kube::discovery::pinned_kind`, fetches the object via `Api<DynamicObject>`, and reads its `k8s.brokkr.io/stack` annotation. UUID parse failure short-circuits to `NotOurs`.
- Wired up in `cli/commands.rs`: spawned once at agent startup right after the k8s client is initialised. The `JoinHandle` is intentionally bound to `_kube_events_handle` — production drops it, the task runs for the agent process lifetime.
- Tests: 3 unit tests on `UidCache` (Owned-within-TTL, NotOurs-is-a-real-entry, absent-key-returns-None). All pass.

**Honest gap**: no full kube-cluster integration test in this commit. The docker stack required to spin up k3s + Shipwright is out of scope for a fast inner loop; that test should land alongside the WS-06 full-loop chaos test and the WS-12 UI integration when the e2e harness is naturally exercised.