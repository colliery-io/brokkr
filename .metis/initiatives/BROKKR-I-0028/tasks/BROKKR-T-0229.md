---
id: slice-1-api-v1-fleet-live-ws
level: task
title: "Slice 1: /api/v1/fleet/live WS endpoint + FleetUpdate fan-out + event-driven push"
short_code: "BROKKR-T-0229"
created_at: 2026-06-13T14:07:52.201407+00:00
updated_at: 2026-06-13T14:07:52.201407+00:00
parent: fleet-live-push
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0028
---

# Slice 1: /api/v1/fleet/live WS endpoint + FleetUpdate fan-out + event-driven push

## Parent Initiative

[[BROKKR-I-0028]]

## Objective

The consumer-facing live stream + the instant (event-driven) half of the hybrid
trigger: push a per-agent FleetAgentRecord whenever the broker observes a WS
connect/disconnect or a heartbeat.

## Acceptance Criteria

- [ ] `WsMessage::FleetUpdate(FleetAgentRecord)` wire variant added (brokkr-wire);
      FleetAgentRecord available wire-side (mirror or move it).
- [ ] Fleet-wide fan-out: a single `broadcast::Sender<WsMessage>` (mirror
      ws/broadcaster.rs's slow-subscriber policy — Lagged → drop, never block).
- [ ] `GET /api/v1/fleet/live` WS upgrade handler, **admin-gated** (same as
      GET /fleet). On connect, subscribe to the fleet channel and forward frames.
- [ ] Single-agent record builder (the rollup's FleetAggregates::load is
      whole-fleet; add a per-agent path or reuse get_agent_fleet_status assembly).
- [ ] Event-driven producers broadcast the affected agent's record on: WS
      register/unregister (ws/registry.rs) and `record_heartbeat`
      (api/v1/agents.rs). Best-effort — a push failure must never affect the
      triggering operation.
- [ ] Metric `brokkr_fleet_live_subscribers` (mirror ws_live_subscribers); doc it.
- [ ] OpenAPI/docs note the new endpoint (WS upgrade — document shape + the
      FleetUpdate frame). SDKs regenerated if the wire/schema surface changes;
      drift checks pass.
- [ ] Tests: a connected subscriber receives a FleetUpdate when an agent
      heartbeats / connects / disconnects; admin gate enforced; a slow subscriber
      doesn't stall ingestion.

## Implementation Notes

- Reuse ws/subscribe.rs (the /stacks/{id}/live handler) as the structural model.
- Keep frames measured-values-only (same FleetAgentRecord); no verdicts.

## Status Updates

*To be added during implementation*
