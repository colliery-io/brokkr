# Monitoring Your Agent Fleet

This guide shows how to watch a fleet of Brokkr agents and spot trouble: pull a one-shot snapshot of every agent, drill into a single agent, and follow live updates over WebSocket. The fleet endpoints expose **measured signals only** — heartbeat age, WebSocket connectivity, backpressure counts, health counts, and agent-reported Kubernetes reachability. Brokkr surfaces those numbers; *you* decide what counts as "trouble." For the reasoning behind that split, see [Fleet Legibility](../explanation/fleet-legibility.md).

This is **application-level** fleet observability — what your agents are doing right now. It is distinct from [Setting Up Monitoring](./monitoring-setup.md), which wires Prometheus/infra metrics for the broker and agent processes themselves. Use that guide for scrape configs and dashboards; use this one to query and stream fleet state.

> The bundled web UI (`examples/ui-slim`) is a demo, not a product. To monitor a real fleet, call the API directly or build your own consumer against the endpoints below.

## Prerequisites

- **An admin PAK.** Every fleet endpoint is admin-only; a non-admin PAK gets `403 Forbidden`. To mint or rotate one, see [Managing PAKs](./pak-management.md).
- A reachable broker (examples below use `http://localhost:3000`).

Export it once so the snippets work as written:

```bash
export ADMIN_PAK="brokkr_..."
export BROKER="http://localhost:3000"
```

## Step 1: Pull the Whole Fleet

`GET /api/v1/fleet` returns one `FleetAgentRecord` per agent — the rollup snapshot:

```bash
curl "$BROKER/api/v1/fleet" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Each element looks like this:

```json
{
  "agent_id": "e5f6g7h8-1234-5678-9abc-def012345678",
  "name": "prod-us-east",
  "status": "ACTIVE",
  "ws_connected": true,
  "connected_since": "2026-06-14T09:00:00Z",
  "last_heartbeat": "2026-06-14T09:14:55Z",
  "heartbeat_age_seconds": 5,
  "pending_object_count": 0,
  "pending_work_orders": 0,
  "claimed_work_orders": 1,
  "last_event_at": "2026-06-14T09:14:50Z",
  "seconds_since_last_event": 10,
  "health_failing": 0,
  "health_degraded": 0,
  "k8s_reachable": true,
  "k8s_api_latency_ms": 12
}
```

Scan each record for the signals that tell you an agent is in trouble:

- **Heartbeat staleness** — `heartbeat_age_seconds` (rising means the agent has gone quiet).
- **WebSocket connectivity** — `ws_connected`.
- **Backpressure** — `pending_object_count`, `pending_work_orders`, `claimed_work_orders`.
- **Health counts** — `health_failing`, `health_degraded`.
- **Kubernetes reachability** — `k8s_reachable` (and `k8s_api_latency_ms`).

For exact types, nullability, and how each is derived, see the [Fleet Reference](../reference/fleet.md).

## Step 2: Drill Into One Agent

When a row in the rollup looks off, get the detail view for that agent. Copy the `agent_id` of the row that looks wrong from Step 1 into `AGENT_ID` (for example, pick the agent with the largest `heartbeat_age_seconds`):

```bash
AGENT_ID=$(curl -s "$BROKER/api/v1/fleet" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r 'max_by(.heartbeat_age_seconds // 0) | .agent_id')
```

`GET /api/v1/agents/{id}/fleet-status` returns the same record plus its most recent events (newest first, up to 20):

```bash
curl "$BROKER/api/v1/agents/$AGENT_ID/fleet-status" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

```json
{
  "record": {
    "agent_id": "e5f6g7h8-...",
    "name": "prod-us-east",
    "heartbeat_age_seconds": 5,
    "...": "...same FleetAgentRecord shape as Step 1..."
  },
  "recent_events": [
    { "agent_id": "e5f6g7h8-...", "event_type": "...", "...": "..." }
  ]
}
```

`record` is identical in shape to a rollup entry; `recent_events` is the trailing event tail you use to see *what* the agent has been doing — useful context when a signal looks wrong. A missing agent returns `404`.

## Step 3: Stream Live Updates

To watch the fleet continuously instead of polling, connect to the live WebSocket at `GET /api/v1/fleet/live`. It is admin-only, exactly like `GET /fleet`. The broker pushes a `fleet_update` frame whenever an agent's signals change — on WebSocket connect/disconnect, on heartbeat, and on a periodic sweep that re-broadcasts agents whose computed signals (backpressure, health counts) changed since the last tick.

Each frame is a `WsMessage` of type `fleet_update`, whose `body` is a single `FleetAgentRecord` (the same fields as Step 1). The pattern is **keyed replacement**: hold a map of records keyed by `agent_id` and overwrite each on its frame. There is no per-agent gap concept (see [Fleet Legibility](../explanation/fleet-legibility.md)).

> **No SDK wraps this endpoint.** Consumers build the URL themselves: take your broker base URL, switch the scheme (`http` → `ws`, `https` → `wss`), and append `/api/v1/fleet/live`.

From a CLI client that can set headers:

```bash
websocat -H="Authorization: Bearer $ADMIN_PAK" \
  "ws://localhost:3000/api/v1/fleet/live"
```

From a browser, which cannot set an `Authorization` header on `new WebSocket()`, pass the PAK as a subprotocol of the form `brokkr.pak.<PAK>` (the broker lifts it into the auth header; it echoes back only the non-secret `brokkr.v1` marker):

```javascript
// Build the WS URL from your broker base URL (http -> ws, https -> wss).
const wsUrl = brokerBaseUrl.replace(/^http/, "ws") + "/api/v1/fleet/live";

const fleet = new Map(); // agent_id -> latest FleetAgentRecord

const ws = new WebSocket(wsUrl, [`brokkr.pak.${adminPak}`, "brokkr.v1"]);

ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  if (msg.type === "fleet_update") {
    const record = msg.body;
    // Keyed replacement: overwrite this agent's row with the latest record.
    fleet.set(record.agent_id, record);
    render(fleet); // your re-render
  }
};
```

The connection is read-only — anything the client sends is ignored. See the [WebSocket Protocol reference](../reference/ws-protocol.md) for the full frame envelope and other message types.

## What to Alert On

Brokkr reports signals and ships no thresholds, so you own the severity decision (for why, see [Fleet Legibility](../explanation/fleet-legibility.md)). The table below maps each signal to the operational condition it indicates — pick your own thresholds:

| Signal in `FleetAgentRecord` | What a bad value indicates |
| --- | --- |
| `heartbeat_age_seconds` rising / large | Agent has gone quiet — process down, crashed, or partitioned from the broker. |
| `ws_connected: false` (esp. with stale heartbeat) | Agent's live channel is gone; live pushes for it will stop. |
| `pending_object_count` high and not draining | Agent isn't applying assigned deployment objects — stuck reconcile or unhealthy agent. |
| `pending_work_orders` high and not draining | Work is queued but not being claimed — agent not pulling work. |
| `claimed_work_orders` stuck non-zero | Work claimed but never completing — a wedged job. |
| `health_failing` > 0 | One or more of the agent's deployments is in `failing`. |
| `health_degraded` > 0 | One or more deployments is `degraded` (partial). |
| `k8s_reachable: false` | The agent reports it cannot reach its own cluster API. |
| `k8s_api_latency_ms` elevated | Agent's cluster API is slow — degrading apply/health throughput. |

Decide thresholds and severity in your own consumer (or in your existing alerting pipeline).

## Related Documentation

- [Fleet Reference](../reference/fleet.md) — full `FleetAgentRecord` / `AgentFleetStatusResponse` schema
- [WebSocket Protocol](../reference/ws-protocol.md) — frame envelope and message types
- [Fleet Legibility](../explanation/fleet-legibility.md) — why Brokkr reports measured signals, not health verdicts
- [Managing PAKs](./pak-management.md) — minting and rotating the admin PAK these endpoints require
- [Monitoring Deployment Health](./deployment-health.md) — per-deployment health detail behind the `health_failing` / `health_degraded` counts
- [Setting Up Monitoring](./monitoring-setup.md) — Prometheus/infra metrics for the broker and agent processes (distinct from fleet observability)
