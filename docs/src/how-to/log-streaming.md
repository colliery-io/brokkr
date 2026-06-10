# Streaming Pod Logs and Live Tail

This guide shows how to stream container logs from a managed cluster to the broker, read the retained history over REST, and follow a live tail over WebSocket. Use it for immediate operational debugging — Brokkr retains telemetry for at most **6 hours** and is not a log warehouse; for long-term log centralization, ship logs to a dedicated platform such as Datadog.

## Prerequisites

- A stack with at least one deployment object applied by an agent
- The stack's UUID
- A PAK authorized for the stack: admin, or the generator that owns it (agent PAKs cannot open the live tail)

## Step 1: Opt the Pod Into Log Streaming

Log streaming is off by default and opted in per pod. A pod is eligible only when **both** annotations are present on the pod itself, so set them in the workload's pod template:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    metadata:
      annotations:
        k8s.brokkr.io/stack: "<your-stack-uuid>"
        brokkr.io/stream-logs: "true"
    spec:
      containers:
        - name: myapp
          image: myapp:1.2.3
```

Submit this as a deployment object to your stack as usual. Once the pods restart with the annotations, the agent tails every container in each annotated pod and forwards lines to the broker.

Lines are rate-limited at 100 lines/second per container. Over-rate lines are dropped and a `log_gap` marker is emitted so you can see that a gap occurred rather than silently missing data.

## Step 2: Read Retained History over REST

The broker keeps the most recent window (hard ceiling: 6 hours) in PostgreSQL. Query it per stack:

```bash
# Pod log lines (default limit 500, max 5000)
curl "http://localhost:3000/api/v1/stacks/$STACK_ID/logs?limit=1000" \
  -H "Authorization: Bearer $PAK"

# Kubernetes events for the stack's objects
curl "http://localhost:3000/api/v1/stacks/$STACK_ID/events?since=2026-06-09T12:00:00Z" \
  -H "Authorization: Bearer $PAK"
```

Both responses include a `retention` block (`retention_ceiling_seconds`, `effective_retention_seconds`, `oldest_available_ts`) so clients can show the available window. A `since` value older than the retention ceiling is silently clamped to the ceiling.

## Step 3: Follow a Live Tail over WebSocket

Connect to the live subscription endpoint to receive telemetry frames as they arrive:

```bash
websocat -H="Authorization: Bearer $PAK" \
  "ws://localhost:3000/api/v1/stacks/$STACK_ID/live"
```

Frames are JSON `WsMessage` objects — `pod_log_line`, `k8s_event`, and `log_gap` (see the [WebSocket Protocol reference](../reference/ws-protocol.md) for shapes). If your client falls behind, the broker drops frames and sends a `log_gap` with `reason: "buffer_full"` instead of blocking ingestion.

From a browser (which cannot set headers on `new WebSocket()`), pass the PAK as a subprotocol:

```javascript
const ws = new WebSocket(
  `wss://broker.example.com/api/v1/stacks/${stackId}/live`,
  [`brokkr.pak.${pak}`, "brokkr.v1"],
);
ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  if (msg.type === "pod_log_line") console.log(msg.body.line);
};
```

With the TypeScript SDK, `client.liveSubscriptionUrl(stackId)` computes the WebSocket URL from your configured base URL, and `client.listTelemetryLogs(stackId)` / `client.listTelemetryEvents(stackId)` wrap the REST history endpoints (see [TypeScript SDK](./sdks/typescript.md)).

## Stopping the Stream

Remove the `brokkr.io/stream-logs` annotation (or set it to anything other than `"true"`) in the pod template and roll the workload. The agent stops tailing pods that no longer carry the opt-in.

## Troubleshooting

- **No lines appear**: confirm both annotations are on the *pod* (`kubectl get pod <pod> -o jsonpath='{.metadata.annotations}'`), not only on the Deployment. Annotations on the top-level object do not propagate to pods.
- **Gaps in the tail**: `log_gap` frames with `reason: "rate_limit"` mean a container exceeded 100 lines/second; the limit is a product stance, not a tunable — ship high-volume logs to a dedicated platform.
- **History shorter than 6 hours**: the window is a ceiling, not a guarantee; `oldest_available_ts` in the response tells you what is actually retained.

## Related Documentation

- [WebSocket Protocol](../reference/ws-protocol.md) — frame shapes and endpoints
- [Agent Annotations and Labels](../reference/agent-annotations.md) — the full key catalog
- [Internal Broker↔Agent WS Channel](../explanation/internal-ws-channel.md) — why the 6-hour ceiling exists
