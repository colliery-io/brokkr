---
id: ws-09-broker-log-event-persistence
level: task
title: "WS-09: Broker log/event persistence — migrations, DAL, ingestion, 6h continuous eviction worker"
short_code: "BROKKR-T-0164"
created_at: 2026-05-23T02:12:42.091501+00:00
updated_at: 2026-05-23T02:12:42.091501+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-09: Broker log/event persistence — migrations, DAL, ingestion, 6h continuous eviction worker

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Persist incoming `K8sEvent` and `PodLogLine` messages in Postgres, expose via DAL, and run a continuous eviction worker enforcing the **6-hour hard retention ceiling**. Per-stack config may shorten retention but cannot extend it past 6h.

## Acceptance Criteria

- [ ] New tables `agent_k8s_events` and `agent_pod_logs` created via Diesel migrations
- [ ] Migrations run cleanly up + down via `angreal models migrations`
- [ ] DAL modules in `crates/brokkr-broker/src/dal/` for both tables; CRUD + paginated time-range queries
- [ ] Ingestion path: WS handler receives messages → DAL insert (batched)
- [ ] Eviction worker runs on a continuous schedule (not on read), deleting rows older than the effective retention; ceiling capped at 6h regardless of caller config
- [ ] Per-stack retention override clamped: requests above 6h are silently capped at 6h, not rejected
- [ ] Storage metrics: rows per table, oldest-row-age, eviction-runs counter (per NFR-006)
- [ ] Integration test: insert rows older than ceiling, run worker, assert deleted
- [ ] Per [[feedback_use_angreal_for_tests]], tests run via `angreal tests integration`

## Implementation Notes

- **Approach**: row-per-event/line in Postgres (no S3) — the 6h ceiling makes the storage shape tractable per the ADR sub-decision. Partition by hour if rate volume justifies it during design.
- **Dependencies**: WS-02 (ingestion point). Parallelizable with WS-07/WS-08.
- **Risk**: write volume from a busy fleet. Use batched inserts; consider COPY for high-throughput paths if needed. Continuous eviction (not lazy) is non-negotiable per NFR-006.