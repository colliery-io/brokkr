---
id: ws-13-ws-diagnostics-observability
level: task
title: "WS-13: WS diagnostics + observability metrics (connected agents, by-type rates, dropped lines)"
short_code: "BROKKR-T-0168"
created_at: 2026-05-23T02:12:48.376126+00:00
updated_at: 2026-05-23T12:36:54.236654+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-13: WS diagnostics + observability metrics (connected agents, by-type rates, dropped lines)

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]]

## Objective

Surface WS connection state for operators, and add Prometheus metrics covering the WS channel and log/event pipeline. Builds on existing Prometheus instrumentation (BROKKR-T-0087).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Admin/diagnostics endpoint `GET /api/v1/admin/ws/connections` returns per-agent state (`agent_id`, `connected_since`, `messages_in`, `messages_out`) + aggregate live-subscriber count. Admin-only.
- [ ] UI agent-list "WS" badge — UI wiring belongs to WS-12 (intentionally cross-task: backend exposes the data here, UI consumes there).
- [x] Metrics added (Prometheus):
  - `brokkr_ws_connected_agents` (gauge) — bumped on register/unregister in `handler.rs`
  - `brokkr_ws_messages_total{direction, type}` (counter) — bumped in `dispatch_uplink` (inbound) and `writer_task` (outbound)
  - `brokkr_ws_live_subscribers` (gauge) — synced from the admin endpoint and on broadcaster activity
  - `brokkr_ws_log_eviction_runs_total` (counter) — bumped in `eviction::run_once`
  - `brokkr_ws_telemetry_evicted_total{table}` (counter) — bumped per eviction with the row count
- [x] Cardinality discipline: deliberately no `agent_id` or `stack_id` label on the high-cardinality counters. Per-agent breakdown lives in the diagnostics endpoint, not Prometheus.
- [ ] **Deferred** to follow-up: `brokkr_ws_reconnects_total` and per-stack dropped-log-lines counter — both require agent-side instrumentation to populate, and `brokkr-agent` doesn't have a Prometheus scrape endpoint today. WS-13 covers the broker-side counters; the agent-side counters land alongside an agent metrics endpoint in a future task.

## Implementation Notes

- **Approach**: extended `metrics.rs` with WS-specific counters/gauges, all registered via the existing `REGISTRY`. Cardinality kept low. New admin endpoint reuses the existing PAK middleware + admin auth check.
- **Dependencies**: WS-02 (registry), WS-09 (eviction worker), WS-11 (broadcaster).
- **Risk**: cardinality on labels — mitigated by *not* labelling counters with `agent_id`/`stack_id`. Operators that need per-agent insight use the diagnostics endpoint.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- `metrics.rs`: 5 new metrics (`WS_CONNECTED_AGENTS`, `WS_MESSAGES_TOTAL`, `WS_LIVE_SUBSCRIBERS`, `WS_LOG_EVICTION_RUNS_TOTAL`, `WS_TELEMETRY_EVICTED_TOTAL`). All registered with the global registry; `init()` forces lazy eval of each.
- `ws/handler.rs`:
  - `ws_connected_agents` inc on register / dec on unregister
  - `ws_variant_name(&WsMessage)` helper returns the snake_case wire tag for the metric label (kept in sync by hand with `brokkr-wire` — golden fixture catches drift)
  - `dispatch_uplink` increments `ws_messages_total("in", variant)` on every received frame
  - `writer_task` increments `ws_messages_total("out", variant)` on every successful send
- `ws/eviction.rs::run_once` increments `ws_log_eviction_runs_total` once per pass and `ws_telemetry_evicted_total{table=…}` by row count when any rows are evicted.
- `api/v1/admin.rs::list_ws_connections` — new admin endpoint. Reads from the shared `ConnectionRegistry::snapshot()` and `LiveBroadcaster::subscriber_count()`. Updates the `ws_live_subscribers` gauge as a side-effect so operators can scrape it without traffic on the channel.
- OpenAPI regenerated; Python + TypeScript SDKs regenerated. All three drift checks green.
- Integration tests added:
  - `admin_ws_connections_endpoint_reports_live_state` — baseline (0/0), connect an agent, see the counter and per-connection entry.
  - `admin_ws_connections_endpoint_rejects_non_admin` — agent PAK gets 403.
- 15/15 `api::ws` integration tests now green.

**Deferred**:
- `brokkr_ws_reconnects_total` (agent-side): needs an agent metrics endpoint that doesn't exist today.
- Per-stack dropped-log-lines counter: requires agent-side aggregation in `pod_logs::RateLimiter`. The `LogGap` frame already carries `dropped_count`, so a future broker-side `dispatch_uplink` could sum it cheaply if needed — left out of v1 to avoid the stack-id label cardinality decision.
- UI "WS" badge — that's WS-12's job.