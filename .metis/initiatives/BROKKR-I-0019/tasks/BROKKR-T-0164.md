---
id: ws-09-broker-log-event-persistence
level: task
title: "WS-09: Broker log/event persistence — migrations, DAL, ingestion, 6h continuous eviction worker"
short_code: "BROKKR-T-0164"
created_at: 2026-05-23T02:12:42.091501+00:00
updated_at: 2026-05-23T10:40:03.966598+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-09: Broker log/event persistence — migrations, DAL, ingestion, 6h continuous eviction worker

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

Persist incoming `K8sEvent` and `PodLogLine` messages in Postgres, expose via DAL, and run a continuous eviction worker enforcing the **6-hour hard retention ceiling**. Per-stack config may shorten retention but cannot extend it past 6h.

## Acceptance Criteria

## Acceptance Criteria

- [x] New tables `agent_k8s_events` and `agent_pod_logs` created via Diesel migrations (`18_agent_telemetry/up.sql` + `down.sql`)
- [x] Migrations run cleanly **up** — proven via the integration tests' `embed_migrations!` path which exercises every migration in order on a fresh schema. Down was hand-validated (simple `DROP TABLE IF EXISTS`); the angreal `models migrations` redo step requires a `diesel-cli` installed with the `postgres` feature locally, which is an env issue, not a code issue.
- [x] DAL modules: `agent_k8s_events.rs` and `agent_pod_logs.rs` in `crates/brokkr-broker/src/dal/`; each exposes `create`, `list_for_stack(stack_id, since, limit)`, `evict_older_than(cutoff)`, `count` (the surface WS-10 + WS-11 + WS-13 will consume).
- [x] Ingestion path: WS handler's `dispatch_uplink` now writes `K8sEvent` → `agent_k8s_events`, `PodLogLine` → `agent_pod_logs`, with mismatched-`agent_id` rejection. `LogGap` is acknowledged but not persisted (gap markers are pure metadata).
- [x] Eviction worker runs on a continuous tokio interval (default 60s), keying on `created_at` (server-side) so backdated agent timestamps cannot extend retention. Spawned at startup in `configure_api_routes`.
- [x] Per-stack retention override clamped: `RetentionConfig::new(requested, tick)` silently clamps anything above `HARD_RETENTION_CEILING` (6h) back down. Covered by `retention_above_ceiling_is_clamped` unit test. (Per-stack policy *plumbing* is deferred — today every stack uses the global default; the clamp is the load-bearing invariant.)
- [x] Integration test: backdate inserted rows by 7h, `run_once` with default policy, assert tables are empty (`eviction_worker_drops_rows_past_retention`).
- [x] Tests run via `angreal tests integration brokkr-broker api::ws` (9/9 green) and `cargo test -p brokkr-broker --lib ws::eviction` (3/3 green).
- [ ] Storage metrics (`brokkr_ws_log_eviction_runs_total`, table sizes) deferred to WS-13.

## Implementation Notes

- **Approach**: row-per-event/line in Postgres; the 6h ceiling keeps storage shape `O(fleet_rate × 6h)` regardless of caller config, per ADR sub-decision. No S3.
- **Dependencies**: WS-02 (ingestion point). Parallelisable with WS-07/WS-08.
- **Risk**: write volume — batched inserts are a follow-up if rate justifies; today's single-frame insert is fine for v1. Continuous eviction (not lazy on read) is non-negotiable per NFR-006 and shipped here.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- Migration `18_agent_telemetry/{up,down}.sql`: two tables (`agent_k8s_events`, `agent_pod_logs`) with stack+created_at composite indexes (for the future WS-10 history query) and standalone `created_at` indexes (for the eviction worker).
- `brokkr-models`: new modules `models/agent_k8s_events.rs` + `models/agent_pod_logs.rs` (Queryable / Insertable structs); schema.rs entries inserted by hand (no migration of the `angreal models schema` task needed for review).
- `brokkr-broker`:
  - DAL: `dal/agent_k8s_events.rs` + `dal/agent_pod_logs.rs` — minimal surface (`create` / `list_for_stack` / `evict_older_than` / `count`).
  - `ws/handler.rs`: `dispatch_uplink` extended — `K8sEvent` and `PodLogLine` now `.create()` in their respective tables; mismatched-agent-id bodies are dropped with a warning to prevent cross-agent forgery.
  - `ws/eviction.rs`: new `RetentionConfig::new` (clamps to `HARD_RETENTION_CEILING = 6h`), `spawn(dal, config)` for the production worker, `run_once(dal, config)` exposed for deterministic tests.
  - `api/mod.rs`: `configure_api_routes` spawns the eviction worker at startup with `RetentionConfig::default_policy()`.
- Tests:
  - 3 unit tests in `ws::eviction::tests` (clamping, preservation, defaults). All pass.
  - 2 new integration tests in `tests/integration/api/ws.rs`:
    - `ws_telemetry_ingestion_lands_in_agent_telemetry_tables` — agent sends K8sEvent + PodLogLine over WS, asserts both land in their tables.
    - `eviction_worker_drops_rows_past_retention` — inserts rows, backdates `created_at` by 7h, runs `run_once`, asserts tables are empty.
  - 9/9 `api::ws` integration tests pass after this task.

**Follow-ups for downstream tasks**:
- WS-10 (REST history) consumes `list_for_stack(stack_id, since, limit)` directly; response should include the retention ceiling per ADR NFR-007.
- WS-11 (fan-out subscription) needs a tap *before* the DAL insert so subscribers see frames at ingest latency. The cleanest spot is in `dispatch_uplink` itself.
- WS-13 (metrics) should add `brokkr_ws_log_eviction_runs_total`, `brokkr_agent_k8s_events_rows`, `brokkr_agent_pod_logs_rows` gauges/counters around the eviction worker.