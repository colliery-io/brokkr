# WebSocket Protocol

This page catalogs Brokkr's three WebSocket surfaces and the wire messages they carry. For the design rationale, lane prioritization, and operating guidance, see [Internal Broker↔Agent WS Channel](../explanation/internal-ws-channel.md).

The wire types are defined in the `brokkr-wire` crate (`crates/brokkr-wire/src/lib.rs`). They are **not** part of the OpenAPI surface and are not generated into the SDKs; external integrators use the REST API. `WIRE_VERSION` equals the crate version, which is released in lockstep with the broker and SDKs.

## Endpoints

| Endpoint | Direction | Auth | Purpose |
|----------|-----------|------|---------|
| `GET /internal/ws/agent` | bidirectional | Agent PAK only (admin and generator PAKs are rejected) | Internal broker↔agent channel: control-plane pushes down, telemetry up |
| `GET /api/v1/stacks/{id}/live` | server → client | Admin PAK or generator PAK owning the stack (agent PAKs are not accepted in v1) | Read-only live tail of a stack's telemetry frames |
| `GET /api/v1/fleet/live` | server → client | Admin PAK only (same gate as `GET /fleet`) | Read-only consumer-facing live stream of per-agent fleet records |

These endpoints are standard HTTP→WebSocket upgrades served by the broker (handlers: `crates/brokkr-broker/src/ws/handler.rs`, `crates/brokkr-broker/src/ws/subscribe.rs`, and `crates/brokkr-broker/src/ws/fleet_subscribe.rs`).

### Authentication

Non-browser clients send the PAK as a normal header on the upgrade request:

```
Authorization: Bearer <PAK>
```

Browsers cannot set headers on `new WebSocket()`. For `/api/v1/stacks/{id}/live` and `/api/v1/fleet/live`, a browser client instead offers the PAK as a subprotocol:

```
Sec-WebSocket-Protocol: brokkr.pak.<PAK>, brokkr.v1
```

The broker extracts the PAK from the `brokkr.pak.` subprotocol (consulted only when no `Authorization` header is present) and echoes back only the non-secret `brokkr.v1` marker.

## Message Envelope

Every frame is a JSON-encoded `WsMessage` with an external tag (`crates/brokkr-wire/src/lib.rs:WsMessage`):

```json
{ "type": "<variant>", "body": { ... } }
```

Variant names are `snake_case`.

## Message Catalog

### Broker → agent (control plane)

| `type` | Body | Meaning |
|--------|------|---------|
| `work_order` | `WorkOrder` (same shape as REST) | A work order this agent may claim was created |
| `target_changed` | `AgentTarget` | A stack target was added for this agent |
| `stack_changed` | `Stack` | A targeted stack's metadata or deployment objects changed |

Control pushes are hints: they are fire-and-forget, sent after the database commit, and the agent's REST polling remains the source of truth (see [ADR-0008](https://github.com/colliery-io/brokkr/blob/main/.metis/adrs/BROKKR-A-0008.md)).

### Agent → broker (uplink)

| `type` | Body | Meaning |
|--------|------|---------|
| `heartbeat` | `{ agent_id, sent_at }` | Liveness signal sent on the agent's poll tick while the connection is up |
| `agent_event` | `AgentEvent` (same shape as REST) | Deployment SUCCESS/FAILURE event |
| `agent_health` | `DeploymentHealth` (same shape as REST) | Health status for one deployment object |

### Agent → broker (streaming telemetry)

| `type` | Body | Meaning |
|--------|------|---------|
| `k8s_event` | `K8sEvent` | A Kubernetes Event for a stack-managed object |
| `pod_log_line` | `PodLogLine` | One container log line from an opted-in pod |
| `log_gap` | `LogGap` | Marker that lines were dropped, so consumers render a visible gap |

Body shapes for the telemetry-only types:

```json
// k8s_event
{
  "agent_id": "<uuid>", "stack_id": "<uuid>",
  "observed_at": "<ISO-8601>",
  "reason": "FailedScheduling", "message": "...",
  "event_type": "Warning", "source": "scheduler",
  "involved_object": {
    "api_version": "v1", "kind": "Pod",
    "namespace": "default", "name": "myapp-abc12", "uid": "<uid>"
  }
}

// pod_log_line
{
  "agent_id": "<uuid>", "stack_id": "<uuid>",
  "namespace": "default", "pod": "myapp-abc12", "container": "myapp",
  "ts": "<ISO-8601>", "line": "..."
}

// log_gap
{
  "agent_id": "<uuid>", "stack_id": "<uuid>",
  "since_ts": "<ISO-8601>", "dropped_count": 42,
  "reason": "rate_limit"
}
```

`log_gap.reason` is one of `rate_limit`, `buffer_full`, `disconnected` (`crates/brokkr-wire/src/lib.rs:GapReason`).

### Broker → consumer (fleet live-push)

Carried only on `GET /api/v1/fleet/live` (BROKKR-I-0028). The broker pushes one frame whenever it observes a discrete fleet event for an agent — a broker↔agent WebSocket connect/disconnect, or a heartbeat receipt. It also pushes a frame from a periodic ~20-second sweep when an agent's computed signals (deployment-object backpressure counts or deployment-health counts) have changed since the last tick. See [Fleet Reference](./fleet.md) for the complete trigger model. The consumer pulls `GET /fleet` once for the baseline, then replaces a row in place keyed by `agent_id`.

| `type` | Body | Meaning |
|--------|------|---------|
| `fleet_update` | `FleetAgentRecord` | One agent's full fleet record (measured values only — same shape as a `GET /fleet` entry); replace by `agent_id` |

The `fleet_update` body is the wire twin of the REST `FleetAgentRecord` (`crates/brokkr-wire/src/lib.rs:FleetAgentRecord`), field-for-field identical to the `GET /fleet` element — measured signals only, no health verdicts:

```json
// fleet_update
{
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
```

Unlike the per-stack live tail, the fleet stream has **no gap marker**: a slow subscriber that lags simply drops the missed frames and continues, because the next `fleet_update` for that `agent_id` supersedes any it missed (the consumer holds the latest record per agent).

## Channel Behavior

| Property | Value | Source |
|----------|-------|--------|
| Per-connection control lane capacity | 64 messages, drained before telemetry | `crates/brokkr-broker/src/ws/handler.rs` |
| Per-connection telemetry lane capacity | 1024 messages | `crates/brokkr-broker/src/ws/handler.rs` |
| Live-tail broadcast capacity (per stack) | 1024 frames; lagged subscribers receive a synthetic `log_gap` | `crates/brokkr-broker/src/ws/broadcaster.rs` |
| Fleet live-push broadcast capacity (fleet-wide) | 1024 frames; lagged subscribers drop and continue (no gap marker — replace-by-`agent_id`) | `crates/brokkr-broker/src/ws/broadcaster.rs` |
| Agent outbound/inbound queues | 256 messages each; a full outbound lane falls back to REST | `crates/brokkr-agent/src/broker_ws.rs` |
| Agent reconnect backoff | Exponential, 1s initial, 60s max | `crates/brokkr-agent/src/broker_ws.rs` |
| Auth-rejection limit | 5 consecutive 401/403s, then the agent stops dialing until restart | `crates/brokkr-agent/src/broker_ws.rs` |
| Telemetry retention | 6-hour hard ceiling, evicted by server-side `created_at` | `crates/brokkr-broker/src/ws/eviction.rs` |

## Observability

WebSocket activity is exposed via the `brokkr_ws_*` Prometheus metrics (see [Monitoring](./monitoring.md#websocket-channel-metrics)) and the per-connection snapshot at `GET /api/v1/admin/ws/connections`. Consumer-facing fleet live-push subscribers are counted by the `brokkr_fleet_live_subscribers` gauge.

## Related Documentation

- [Internal Broker↔Agent WS Channel](../explanation/internal-ws-channel.md) — design and operating notes
- [Streaming Pod Logs and Live Tail](../how-to/log-streaming.md) — using the live tail
- [Monitoring & Observability](./monitoring.md) — metrics catalog
