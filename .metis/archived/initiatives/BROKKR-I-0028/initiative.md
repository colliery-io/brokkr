---
id: fleet-live-push
level: initiative
title: "Fleet Live Push"
short_code: "BROKKR-I-0028"
created_at: 2026-06-13T14:07:20.432841+00:00
updated_at: 2026-06-13T15:01:26.803465+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: fleet-live-push
---

# Fleet Live Push Initiative

## Context

[[BROKKR-I-0027]] (Agent Fleet Legibility) shipped the pull surface
`GET /api/v1/fleet` and deferred real-time push as optional. This initiative
delivers that push: a consumer-facing WebSocket stream of fleet-state changes so
a platform embedding Brokkr can keep its "this is your fleet" view live without
polling.

Brokkr already has the consumer-facing live-push machinery to reuse:
- `crates/brokkr-broker/src/ws/broadcaster.rs` — a `tokio::sync::broadcast`
  fan-out hub (today keyed per-stack for telemetry), with the ADR-0008
  slow-subscriber policy: a lagging subscriber gets `Lagged` and gap-marks,
  never blocking ingestion.
- `ws/subscribe.rs` — the `/stacks/{id}/live` subscribe handler pattern.
- The `WsMessage` wire enum (`crates/brokkr-wire`) carrying the live frames.

## Goals & Non-Goals

**Goals:**
- A consumer WS endpoint (`/api/v1/fleet/live`, admin-gated like `GET /fleet`)
  that streams per-agent fleet updates as `WsMessage::FleetUpdate(FleetAgentRecord)`.
- **Hybrid trigger** (maintainer decision): instant push on the discrete events
  the broker already sees (WS connect/disconnect, heartbeat receipt) + a
  low-frequency sweep that re-broadcasts an agent's record when its *computed*
  signals (backpressure, health counts) change.
- Measured-values-only frames (same FleetAgentRecord as the pull surface) — the
  consumer still owns severity.

**Non-Goals:**
- Replacing the pull surface — the consumer pulls `GET /fleet` once for the
  baseline, then subscribes for deltas (replace-by-agent_id). No snapshot-on-
  connect in v1.
- Alerting / verdicts (unchanged from I-0027 — Brokkr surfaces signals).
- Per-field diffing — frames carry the full per-agent record; the consumer
  replaces its row.

## Detailed Design

- **Fan-out:** a single fleet-wide `broadcast::Sender<WsMessage>` (not per-stack);
  every `/fleet/live` subscriber receives every agent's updates. Mirror
  broadcaster.rs's slow-subscriber handling.
- **Wire:** add `WsMessage::FleetUpdate(FleetAgentRecord)` (FleetAgentRecord moves
  to / is mirrored in brokkr-wire, or a wire-side twin). Optional/back-compat not
  required (new variant; consumers opt in by connecting).
- **Producers (event-driven):** on WS register/unregister (registry.rs) and on
  `record_heartbeat`, recompute *that one agent's* FleetAgentRecord and broadcast
  it. Needs a single-agent record builder (the rollup's FleetAggregates::load is
  whole-fleet; add a per-agent path or reuse get_agent_fleet_status's assembly).
- **Producer (periodic sweep):** a background task (~15–30s) recomputes fleet
  records, diffs the computed fields (pending_object_count, pending/claimed work
  orders, health_failing/degraded) against the last broadcast, and re-broadcasts
  only changed agents.
- **Endpoint:** `/api/v1/fleet/live` WS upgrade handler, admin-gated; on connect,
  subscribe to the fleet channel and forward frames. Metric
  `brokkr_fleet_live_subscribers` (mirror ws_live_subscribers).

## Alternatives Considered

- **Event-driven only** — misses backpressure/health (computed, not events) until
  the next heartbeat. Rejected: leaves the most operationally interesting signals
  laggy.
- **Periodic snapshot only** — simple but polling-in-disguise; coarse latency on
  connect/disconnect. Rejected.
- **Snapshot-on-connect** — deferred; baseline-via-GET is simpler and reuses the
  shipped surface.

## Implementation Plan

- **Slice 1 — Wire + fan-out + endpoint + event-driven push:** `FleetUpdate`
  variant, fleet-wide broadcast hub, `/fleet/live` handler (admin-gated),
  single-agent record builder, producers on connect/disconnect + heartbeat,
  subscriber metric, tests.
- **Slice 2 — Periodic recompute-and-diff sweep:** background task that
  re-broadcasts agents whose computed signals changed; tests.

## Status Updates

- 2026-06-13: Created from the I-0027 closeout's deferred WS-push. Trigger model
  = hybrid (maintainer). Design grounded in the existing ws/broadcaster fan-out.
## Closeout 2026-06-13

Completed. Both slices shipped in PR #66:
- T-0229 — GET /api/v1/fleet/live WS stream + FleetUpdate fan-out + event-driven
  push (connect/disconnect + heartbeat).
- T-0230 — periodic 20s recompute-and-diff sweep for the computed signals
  (backpressure/health).

The hybrid trigger is complete: instant pushes on discrete events + the sweep
for computed signals. A consumer pulls GET /fleet for the baseline, then
subscribes to /fleet/live and replaces records by agent_id. No deferred work.