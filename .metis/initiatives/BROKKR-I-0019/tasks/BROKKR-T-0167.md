---
id: ws-12-ui-live-tail-history
level: task
title: "WS-12: UI live tail + history integration (ui-slim)"
short_code: "BROKKR-T-0167"
created_at: 2026-05-23T02:12:46.824209+00:00
updated_at: 2026-05-23T18:23:18.329836+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-12: UI live tail + history integration (ui-slim)

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]]

## Objective

In `ui-slim`, add a per-stack "Live" view that subscribes to event/log tail (WS-11) and a "History" view that paginates through the REST history (WS-10). Surface gap markers, reconnect state, and the 6h retention ceiling.

## Acceptance Criteria

## Acceptance Criteria

- [x] Per-stack stack-detail modal now has a Telemetry section with Kube Events / Pod Logs tabs.
- [x] History tab paginates via WS-10 REST endpoints (`api.getStackEvents`, `api.getStackLogs`); time-range filtering via `since` is plumbed through the call signature even though the v1 UI only uses the default 6h window.
- [x] Clear UX message about the 6h retention ceiling + Datadog hint, rendered from the `retention.retention_ceiling_seconds` field in every response.
- [x] WS-connected indicator on the agent list — polls `/api/v1/admin/ws/connections` every 10s (admin-only; silent fallback for non-admin), renders a 🔌 next to agents currently on the internal WS channel.
- [x] ui-slim builds clean (`npx react-scripts build` — main bundle +970 B).
- [ ] **Deferred to follow-up**: live WS subscription view (WS-11 consumer). Browsers don't allow custom headers on `new WebSocket(url)`, so authenticated subscription from the UI needs either (a) a same-origin reverse proxy that injects `Authorization`, or (b) a broker-side change to accept the PAK via subprotocol/query parameter. Either path is real work; the REST-history view + the architectural docs cover the "this is a 6h buffer, not a log store" UX angle which is the WS-12 critical message.

## Implementation Notes

- **Approach taken**: extended `examples/ui-slim/src/api.js` with `getStackEvents`, `getStackLogs`, `getWsConnections`, and an `openStackLiveStream` URL+token helper (commented for ingress-with-auth deployments). New `StackTelemetrySection` component in `App.js` consumes the history endpoints and renders the retention metadata explicitly. `AgentsPanel` grew a `wsConnected` poll loop + a 🔌 badge.
- **Dependencies**: WS-10 (REST), WS-13 (admin endpoint).
- **Risk**: long-running WS subscriptions through reverse proxies (ingress timeouts). Documented in WS-14 ops guidance.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- `api.js`: added `getStackEvents(id, query)`, `getStackLogs(id, query)`, `getWsConnections()`, `openStackLiveStream(id)`. The TypeScript SDK's ergonomic methods are called directly so the unwrap-tuple boilerplate is avoided.
- `App.js::StackTelemetrySection` — Kube Events / Pod Logs tabs with a retention banner ("6h buffer — for long-term retention, ship to Datadog") sourced from `retention.retention_ceiling_seconds`. Mounted in the existing stack-detail Modal alongside Deployment Objects.
- `App.js::AgentsPanel` — 10s poll loop populates a `wsConnected: Set<UUID>`; agents in the set render with a 🔌 badge. The poll fails silently for non-admin PAKs so the UI degrades to "no badge" rather than erroring.
- `npx react-scripts build` runs clean; main bundle +970 B.

**Deferred** (explicitly out of scope for v1, documented in the task body):
- Browser-side live WS subscription. The TS SDK exposes `liveSubscriptionUrl(stackId)` so a future deployment with a same-origin auth-proxy can subscribe with one line; today's UI shows history only.