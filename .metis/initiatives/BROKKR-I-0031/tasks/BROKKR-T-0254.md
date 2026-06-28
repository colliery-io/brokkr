---
id: slice-1c-first-live-read-view
level: task
title: "Slice 1c: first live read view — Fleet, bound to GET /api/v1/fleet + /fleet/live WS"
short_code: "BROKKR-T-0254"
created_at: 2026-06-28T01:32:27.844905+00:00
updated_at: 2026-06-28T01:32:27.844905+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Slice 1c: first live read view — Fleet

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Bind the **Fleet** view to real broker data — closing the walking skeleton end-to-end:
broker → wasm console → live fleet on screen. Proves the data path (REST + live WS) and
the Aurora-themed components against real types, turning the rest of the views into
mechanical scope work.

### Type
- [x] Feature — first real read surface (data half of the walking skeleton)

## Acceptance Criteria

- [ ] Fleet view loads from **`GET /api/v1/fleet`**: per-cluster panels of agent rows
      (mono name, status pill, health pill, label chips, `⇄ ws` when on the internal WS,
      right-aligned heartbeat "ago" colored by recency) — matching the handoff Fleet spec.
- [ ] Subscribes to **`/api/v1/fleet/live`** (WS) and updates rows live (heartbeat aging,
      status/health changes); Live/Paused toggle gates the stream.
- [ ] Fleet KPI strip (total / active / degraded / failing) bound to the same data.
- [ ] States handled: **Loading**, **Empty**, **ErrorState** (per the handoff state views;
      classify by HTTP status, offer retry).
- [ ] A typed API client layer in `brokkr-web` (reusing `brokkr-client`/broker types where
      wasm permits) — not hand-rolled JSON.
- [ ] Renders correctly served by the broker (1b), themed only via tokens (1a).

## Implementation Notes

### Technical Approach
- Data shapes: `GET /api/v1/fleet` (fleet records) + `/fleet/live` WS frames (I-0027 /
  I-0028). Confirm the WS frame schema and reconnect/backoff behaviour.
- The agent-detail slide-over and the **run-diagnostic** write are **Slice 3**, not here —
  keep 1c read-only (rows can be non-clickable or open a stub).

### Dependencies
- Depends on [[BROKKR-T-0252]] (shell + `aurora-leptos` wired) and [[BROKKR-T-0253]] (served
  by the broker, so this runs against a real broker).

### Risk Considerations
- wasm WebSocket client + reconnect; map `/fleet/live` frames onto the row model.
- Keep status→color mapping centralized (shared with later views).

## Status Updates

*To be added during implementation*
