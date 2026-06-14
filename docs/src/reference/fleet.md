# Fleet Reference

This page catalogs Brokkr's fleet observability surface (shipped in v0.8.0): the per-agent **fleet record**, the REST endpoints that return it, the WebSocket live-push stream that broadcasts it, the broker triggers that emit updates, and the Prometheus metrics that count subscribers. For the design rationale — what a fleet record measures, why it carries no health verdicts, and how the hybrid event/sweep trigger is shaped — see [Fleet Legibility](../explanation/fleet-legibility.md).

A fleet record exposes **measured signals only** (no computed health verdicts), assembled entirely from data the broker already holds. The rollup is computed with bounded, grouped queries — one grouped query per signal, not one query per agent (`crates/brokkr-broker/src/api/v1/fleet.rs:FleetAggregates::load`).

The REST `FleetAgentRecord` (`crates/brokkr-broker/src/api/v1/fleet.rs`) is the authoritative `utoipa::ToSchema` type exposed by the OpenAPI surface. The live-push frames carry its `brokkr-wire` twin (`crates/brokkr-wire/src/lib.rs:FleetAgentRecord`), a plain serde struct kept field-for-field identical via the single conversion point `FleetAgentRecord::to_wire()`.

> The Brokkr web UI (`examples/ui-slim`) is a demonstration of what a consumer can build on this surface. It is not a supported product or the consumption path. The REST endpoints and the live stream below are the consumption surface.

## FleetAgentRecord

The record returned by `GET /fleet` (as a `Vec`), nested in the `record` field of `GET /agents/{id}/fleet-status`, and carried as the body of each `fleet_update` live-push frame.

All time-relative fields (`heartbeat_age_seconds`, `seconds_since_last_event`) are computed on read as `now - timestamp`, clamped to be non-negative.

| Field | Type | Meaning | Source |
|-------|------|---------|--------|
| `agent_id` | UUID | The agent's unique identifier | `agents.id` |
| `name` | string | The agent's name | `agents.name` |
| `status` | string | The agent's lifecycle status (e.g. `"ACTIVE"`) | `agents.status` |
| `ws_connected` | boolean | Whether the agent currently holds a broker↔agent WebSocket connection | `true` iff the agent has an entry in the in-memory `ConnectionRegistry` snapshot |
| `connected_since` | ISO-8601 datetime or null | When the current WebSocket connection was established; `null` when not connected | `ConnectionRegistry` snapshot (`connected_since`) |
| `last_heartbeat` | ISO-8601 datetime or null | The agent's last recorded heartbeat timestamp | `agents.last_heartbeat` |
| `heartbeat_age_seconds` | integer (i64) or null | Seconds since the last heartbeat (`now - last_heartbeat`, clamped ≥ 0); `null` when no heartbeat recorded | computed from `agents.last_heartbeat` |
| `pending_object_count` | integer (i64) | Number of pending (not-yet-acknowledged) deployment objects targeted at this agent; `0` when none | grouped query `deployment_objects().pending_counts_by_agent()` |
| `pending_work_orders` | integer (i64) | Number of `PENDING` work orders this agent is eligible to claim; `0` when none | grouped query `work_orders().pending_counts_by_agent()` |
| `claimed_work_orders` | integer (i64) | Number of work orders currently `CLAIMED` by this agent; `0` when none | grouped query `work_orders().claimed_counts_by_agent()` |
| `last_event_at` | ISO-8601 datetime or null | Timestamp of this agent's most recent (non-deleted) event, if any | grouped query `agent_events().last_event_at_by_agent()` |
| `seconds_since_last_event` | integer (i64) or null | Seconds since the last event (`now - last_event_at`, clamped ≥ 0); `null` when no events | computed from `last_event_at` |
| `health_failing` | integer (i64) | Count of this agent's deployment-health records with status `failing`; `0` when none | grouped query `deployment_health().status_counts_by_agent()` |
| `health_degraded` | integer (i64) | Count of this agent's deployment-health records with status `degraded`; `0` when none | grouped query `deployment_health().status_counts_by_agent()` |
| `k8s_reachable` | boolean or null | Latest agent-reported reachability of its own Kubernetes API; `null` when the agent has never reported | `agents.k8s_reachable` (trusted as-is — the one signal the broker cannot compute) |
| `k8s_api_latency_ms` | integer (i64) or null | Latest agent-reported latency (milliseconds) of the Kubernetes API reachability probe; `null` when unreported or not measured | `agents.k8s_api_latency_ms` |

`health_failing` / `health_degraded` are raw record counts, not verdicts. The record carries no overall health status field; consumers derive any rollup verdict themselves. See [Deployment Health](./deployment-health.md) for the per-deployment-object health model these counts aggregate.

## REST Endpoints

Both fleet REST endpoints are **admin-only**. Authentication is the standard v1 PAK middleware; the handler then requires `AuthPayload.admin` to be true (`require_admin`). A non-admin PAK (generator or agent PAK) receives `403 Forbidden` with error code `admin_required`.

| Method | Path | Auth | Success body |
|--------|------|------|--------------|
| GET | `/api/v1/fleet` | Admin PAK only | `Vec<FleetAgentRecord>` |
| GET | `/api/v1/agents/{id}/fleet-status` | Admin PAK only | `AgentFleetStatusResponse` |

There is **no `/fleet/{id}` route**. The per-agent detail view is mounted under `/api/v1/agents/{id}/fleet-status`.

### GET /api/v1/fleet

Handler: `list_fleet`. Returns one `FleetAgentRecord` per agent (every agent returned by `dal.agents().list()`), assembled from the shared `FleetAggregates`.

| Status | Body | Condition |
|--------|------|-----------|
| 200 | `Vec<FleetAgentRecord>` | Success |
| 403 | `ErrorResponse` (`admin_required`) | Caller PAK is not admin |
| 500 | `ErrorResponse` | Aggregate computation or agent fetch failed |

### GET /api/v1/agents/{id}/fleet-status

Handler: `get_agent_fleet_status`. Path parameter `id` (`Uuid`) is the agent ID. Returns the agent's fleet record plus its most recent events.

`AgentFleetStatusResponse` (`crates/brokkr-broker/src/api/v1/fleet.rs`):

| Field | Type | Meaning |
|-------|------|---------|
| `record` | `FleetAgentRecord` | The per-agent fleet record (same shape as a `GET /fleet` entry) |
| `recent_events` | array of `AgentEvent` | The agent's most recent events, newest first, capped at 20 (`RECENT_EVENTS_LIMIT`) |

| Status | Body | Condition |
|--------|------|-----------|
| 200 | `AgentFleetStatusResponse` | Success |
| 403 | `ErrorResponse` (`admin_required`) | Caller PAK is not admin |
| 404 | `ErrorResponse` (`agent_not_found`) | No agent with the given `id` |
| 500 | `ErrorResponse` | Aggregate computation or event fetch failed |

## Live Push: GET /api/v1/fleet/live

A read-only, server → client WebSocket stream (`crates/brokkr-broker/src/ws/fleet_subscribe.rs`). The broker pushes a frame whenever an agent's fleet record changes (see [Update Triggers](#update-triggers)). Each frame carries one agent's full record, keyed by `agent_id`; frames are not deltas.

### Authentication

Admin-gated, the same gate as `GET /fleet`: after the standard v1 PAK middleware runs, the upgrade handler requires `AuthPayload.admin`. A non-admin payload yields `403 Forbidden`; a missing payload yields `401 Unauthorized`.

Non-browser clients send the PAK as a header on the upgrade request:

```
Authorization: Bearer <PAK>
```

Browsers cannot set headers on `new WebSocket()` and instead offer the PAK as a subprotocol:

```
Sec-WebSocket-Protocol: brokkr.pak.<PAK>, brokkr.v1
```

The `brokkr.pak.<PAK>` subprotocol is consulted only when no `Authorization` header is present (`ws_subprotocol_auth`); the broker lifts the PAK into an `Authorization: Bearer` header before auth runs. The handshake response echoes back **only** the non-secret `brokkr.v1` marker (`WS_MARKER_SUBPROTOCOL`), never the PAK-bearing subprotocol.

### Frame Shape

Each frame is a JSON-encoded `WsMessage` using the externally-tagged envelope shared by every Brokkr WS surface (`crates/brokkr-wire/src/lib.rs:WsMessage`, `#[serde(tag = "type", content = "body", rename_all = "snake_case")]`):

```json
{ "type": "fleet_update", "body": { /* FleetAgentRecord */ } }
```

Only the `fleet_update` variant is emitted on this stream. The `body` is the wire `FleetAgentRecord` — field-for-field identical to a `GET /fleet` element (measured signals only, no health verdicts):

```json
{
  "type": "fleet_update",
  "body": {
    "agent_id": "<uuid>",
    "name": "demo-agent",
    "status": "ACTIVE",
    "ws_connected": true,
    "connected_since": "<ISO-8601 | null>",
    "last_heartbeat": "<ISO-8601 | null>",
    "heartbeat_age_seconds": 0,
    "pending_object_count": 0,
    "pending_work_orders": 0,
    "claimed_work_orders": 0,
    "last_event_at": "<ISO-8601 | null>",
    "seconds_since_last_event": 0,
    "health_failing": 0,
    "health_degraded": 0,
    "k8s_reachable": true,
    "k8s_api_latency_ms": 12
  }
}
```

Each frame carries **one agent's full record**. The consumer replaces its row for that `agent_id` wholesale; frames are not deltas.

### Slow-Subscriber Policy

The stream has **no gap marker**. When a subscriber lags past the broadcast channel capacity, the broker receives `RecvError::Lagged(n)`, logs it at debug level (`"fleet-live subscriber lagged; continuing"`), drops the missed frames, and continues (`run_fleet_subscriber`). No gap marker is emitted. For the rationale, see [Fleet Legibility](../explanation/fleet-legibility.md).

The subscription is read-only. Any inbound frame other than a close is ignored. A client-side `Close` (or a stream error) ends the subscription; a closed broadcast channel ends it as well.

## Update Triggers

The broker emits a `fleet_update` from two sources: discrete **event-driven** producers and a periodic **computed-signal sweep**. Both call the single producer entry point `broadcast_agent_fleet_update`, which recomputes the affected agent's record (`build_agent_fleet_record`) and broadcasts it. It never returns an error and never panics; a push failure does not affect the operation that triggered it.

### Event-driven producers

| Trigger | Call site | Record reflects |
|---------|-----------|-----------------|
| Broker↔agent WebSocket connect | `crates/brokkr-broker/src/ws/handler.rs` (`run_connection`, on register) | `ws_connected = true`, `connected_since` set |
| Broker↔agent WebSocket disconnect | `crates/brokkr-broker/src/ws/handler.rs` (`run_connection`, after unregister) | `ws_connected = false` |
| Agent heartbeat (REST `POST /api/v1/agents/{id}/heartbeat`) | `crates/brokkr-broker/src/api/v1/agents.rs` (`record_heartbeat`) | refreshed `last_heartbeat`, `heartbeat_age_seconds`, and any reported `k8s_reachable` / `k8s_api_latency_ms` |

These cover the fields that change on a discrete event (`ws_connected`, `connected_since`, `last_heartbeat`, `heartbeat_age_seconds`, `k8s_reachable`, `k8s_api_latency_ms`).

### Periodic computed-signal sweep

The remaining fields are computed aggregates not tied to a single event, so a periodic sweep covers them (`start_fleet_sweep_task`, `crates/brokkr-broker/src/utils/background_tasks.rs`). The sweep runs on a **20-second cadence** (`crates/brokkr-broker/src/api/mod.rs`). On each tick it rebuilds all fleet records, then broadcasts a `fleet_update` only for agents whose **computed signals** changed since the previous tick (`select_changed_fleet_records`). The first tick seeds the baseline and broadcasts nothing.

The computed signals compared between ticks (`fleet_computed_signals`) are exactly:

| Signal | Field |
|--------|-------|
| Pending deployment objects | `pending_object_count` |
| Pending work orders | `pending_work_orders` |
| Claimed work orders | `claimed_work_orders` |
| Failing health records | `health_failing` |
| Degraded health records | `health_degraded` |

A change in any of these five values for an agent (or an agent newly appearing) triggers a `fleet_update` for that agent on the next tick. Fields **not** in this set — `ws_connected`, `connected_since`, `last_heartbeat`, `heartbeat_age_seconds`, `last_event_at`, `seconds_since_last_event`, `k8s_reachable`, `k8s_api_latency_ms`, `status`, `name` — do **not** by themselves cause the sweep to push; the WS connect/disconnect and heartbeat producers cover the event-bound subset, and the live-relevant time-relative fields (`heartbeat_age_seconds`, `seconds_since_last_event`) are recomputed on read in whatever frame is next emitted for that agent.

## Metrics

The fleet surface contributes the following Prometheus metrics on the broker `/metrics` endpoint (`crates/brokkr-broker/src/metrics.rs`). See [Monitoring and Observability](./monitoring.md) for the full broker metrics catalog and PromQL examples.

| Metric | Type | Labels | Meaning |
|--------|------|--------|---------|
| `brokkr_fleet_live_subscribers` | Gauge | none | Current count of `/api/v1/fleet/live` subscribers; incremented on subscribe, decremented on disconnect (`run_fleet_subscriber`) |
| `brokkr_active_agents` | Gauge | none | Number of agents with `status == "ACTIVE"`; refreshed from the database every ~30s by the agent-metrics refresh task |
| `brokkr_agent_heartbeat_age_seconds` | Gauge | `agent_id`, `agent_name` | Seconds since each agent's last heartbeat; refreshed from the database every ~30s (and set to `0` on each recorded heartbeat) |

`brokkr_active_agents` and `brokkr_agent_heartbeat_age_seconds` are refreshed independently of `GET /fleet` traffic by `start_agent_metrics_refresh_task` (`crates/brokkr-broker/src/utils/background_tasks.rs`).

## Related Documentation

- [Fleet Legibility](../explanation/fleet-legibility.md) — the why: what the record measures, why no verdicts, the hybrid trigger design
- [WebSocket Protocol](./ws-protocol.md) — the full WS message catalog and channel behavior, including the `fleet_update` envelope
- [API Reference](./api/README.md) — generated REST API reference
- [Monitoring and Observability](./monitoring.md) — broker and agent Prometheus metrics catalog
- [Deployment Health](./deployment-health.md) — the per-deployment-object health model behind `health_failing` / `health_degraded`
