---
id: slice-1-get-api-v1-fleet-broker
level: task
title: "Slice 1: GET /api/v1/fleet broker-computed fleet surface + gauge-staleness fix"
short_code: "BROKKR-T-0226"
created_at: 2026-06-12T21:39:43.651317+00:00
updated_at: 2026-06-13T13:49:12.678450+00:00
parent: agent-fleet-legibility
blocked_by: []
archived: true

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0027
---

# Slice 1: GET /api/v1/fleet broker-computed fleet surface + gauge-staleness fix

## Parent Initiative

[[BROKKR-I-0027]]

## Objective

Ship the broker-computed fleet legibility surface: a pull endpoint that returns,
per agent, the measured signals defined in the I-0027 v1 fleet record — using
only data the broker already has (no migrations, no agent changes). Also fold in
the pre-existing Prometheus gauge-staleness fix.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `GET /api/v1/fleet` returns an array of per-agent records with measured
      values (no verdicts): `agent_id`, `name`, `status`, `ws_connected`,
      `connected_since`, `last_heartbeat`, `heartbeat_age_seconds`,
      `pending_object_count`, `pending_work_orders`, `claimed_work_orders`,
      `last_event_at`, `seconds_since_last_event`, `health_failing`,
      `health_degraded`. (`k8s_reachable` is Slice 2 — omit here.)
- [ ] Per-agent detail view returns the same record plus the recent-events feed
      (latest N `agent_events`, newest first). Decide: new
      `GET /api/v1/agents/{id}/fleet-status` vs extending an existing handler.
- [ ] All fields computed from existing data on read:
      - `ws_connected`/`connected_since` from the WS `ConnectionRegistry`
        (`is_connected` / snapshot).
      - `heartbeat_age_seconds` = now − `agents.last_heartbeat` (computed on read,
        not the gauge).
      - `pending_object_count` = target-state objects for the agent with no
        `agent_event` yet (reuse `get_target_state_for_agent(.., include_deployed=false)`).
      - `pending_work_orders` (list_pending_for_agent count) + `claimed_work_orders`
        (`work_orders WHERE status='CLAIMED' AND claimed_by=agent`).
      - `last_event_at` = `max(agent_events.created_at)` per agent;
        `seconds_since_last_event` derived.
      - `health_failing`/`health_degraded` = `deployment_health` counts by status
        for the agent.
- [ ] Efficiency: the rollup must not be N+1 per agent — aggregate counts in
      bounded queries (e.g. grouped counts), not a per-agent query fan-out.
- [ ] **Gauge-staleness fix (folded in):** add a background task that periodically
      refreshes `brokkr_active_agents` and `brokkr_agent_heartbeat_age_seconds`
      from the DB, so they are correct independent of who calls `GET /agents`
      (today they only refresh inside the `list_agents` handler — agents.rs:113/117).
- [ ] OpenAPI updated + spec/SDKs regenerated (`angreal openapi export` + gen);
      drift checks pass.
- [ ] Reference docs updated (new endpoint in the API reference; mention in
      monitoring/agent-fleet docs).
- [ ] Tests: integration test asserts the rollup returns the expected fields and
      that pending/backpressure/activity values reflect seeded state; unit test
      for the gauge-refresh task.

## Implementation Notes

- Authz: same RBAC as `GET /agents` (admin/operator visibility) — confirm the
  generator/agent roles cannot read the whole fleet.
- Auth-cache / DAL: route DB access through the existing DAL `conn()` helper.
- Keep it read-only and verdict-free — measured values only (I-0027 philosophy).
- Watch cardinality on any new metrics (per-agent labels stay off hot-path
  counters, per the WS-metrics convention in metrics.rs).

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-13: IMPLEMENTED + verified. `GET /api/v1/fleet` (rollup) and
  `GET /api/v1/agents/{id}/fleet-status` (detail + recent events), admin-gated
  (matches list_agents). All v1 measured fields assembled O(N) from bounded
  grouped queries — NO N+1: new DAL methods last_event_at_by_agent,
  status_counts_by_agent, claimed_counts_by_agent, and set-based
  pending_counts_by_agent (deployment objects + work orders) that replicate the
  per-agent target/matching semantics. Prometheus gauge-staleness fix folded in
  (background refresh of active_agents + agent_heartbeat_age_seconds in
  background_tasks.rs). OpenAPI + Python/TS SDKs regenerated (all 3 drift checks
  pass). Correctness anchor: integration test
  test_fleet_grouped_methods_match_per_agent_ground_truth asserts the grouped
  methods == get_target_state_for_agent / list_pending_for_agent per agent —
  PASSES against a real DB. cargo build + clippy (warning-free) clean.