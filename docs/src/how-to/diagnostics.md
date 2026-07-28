# How-To: Running On-Demand Diagnostics

This guide shows how to collect pod statuses, Kubernetes events, and container logs from a remote cluster when a deployment is misbehaving. Brokkr's diagnostic system lets you request this data through the broker API without direct `kubectl` access to the target cluster.

## When to Use Diagnostics

Use on-demand diagnostics when:

- A deployment shows `degraded` or `failing` health status
- You need to see pod conditions, restart counts, or OOMKill events
- You want container logs from a remote cluster you can't directly access
- You're troubleshooting why a deployment object failed to apply

## Prerequisites

- Admin PAK for the broker
- The `deployment_object_id` of the resource you want to diagnose
- The `agent_id` of the agent running in the target cluster
- The agent must be connected and sending heartbeats
- The workload's pods must carry the `brokkr.io/deployment-object-id=<id>` label — diagnostics discover pods by it, in the namespaces declared by the deployment object's manifests; see [Known Limitations](../reference/diagnostics.md#known-limitations)

## Step 1: Identify the Deployment Object

If you know the stack, list its deployment objects:

```bash
curl -s "http://localhost:3000/api/v1/stacks/${STACK_ID}/deployment-objects" \
  -H "Authorization: <admin-pak>" | jq '.[] | {id, sequence_id, created_at}'
```

Check the health status to confirm something is wrong:

```bash
curl -s "http://localhost:3000/api/v1/deployment-objects/${DO_ID}/health" \
  -H "Authorization: <admin-pak>" | jq .
```

## Step 2: Find the Target Agent

List agents that target this stack:

```bash
curl -s "http://localhost:3000/api/v1/agents" \
  -H "Authorization: <admin-pak>" | jq '.[] | {id, name, cluster_name, last_heartbeat}'
```

Verify the agent has a recent heartbeat (within the last few minutes).

## Step 3: Request Diagnostics

Create a diagnostic request:

```bash
curl -s -X POST "http://localhost:3000/api/v1/deployment-objects/${DO_ID}/diagnostics" \
  -H "Authorization: <admin-pak>" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"${AGENT_ID}\",
    \"requested_by\": \"oncall-engineer\",
    \"retention_minutes\": 120
  }" | jq .
```

Save the diagnostic request ID from the response:

```bash
DIAG_ID="..."
```

The `retention_minutes` field controls how long the request stays active before expiring. Default is 60 minutes, maximum is 1440 (24 hours).

## Step 4: Wait for Results

The agent picks up the diagnostic request on its next poll cycle. Poll the diagnostic status:

```bash
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" | jq '.request.status'
```

Status progression: `pending` → `claimed` → `completed`

A request that is never claimed eventually becomes `expired`. A request that an agent claimed but never finished becomes `failed` once it passes its expiry — that is the signal an agent took the work and stopped answering, usually because it crashed, was evicted, or was rescheduled.

Note that a collection *error* is not a `failed` request: if the agent hits an error while collecting, it still reports a `completed` request and puts the error in the result payload. Check the payload rather than waiting for a failure status.

## Step 5: Check Whether Collection Actually Succeeded

`completed` means the agent submitted a result, not that the result contains data. Before reading the results, check the `events` array for an error entry:

```bash
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" \
  | jq -r '.result.events' \
  | jq -e 'if (type == "array" and length == 1 and .[0].error) then .[0].error else empty end'
```

If that prints an error string, collection failed on the agent — jump to [Troubleshooting](#troubleshooting). An error result also has `pod_statuses` set to `[]` and `log_tails` set to `null`, for example:

```json
[{"error":"Failed to list pods in namespace default: ApiError: pods is forbidden"}]
```

If it prints nothing, collection succeeded and you can read the results below. Note that a successful collection can still be empty (`pod_statuses: []`) when no pods carry the expected label — that case has no `error` entry.

## Step 6: Read the Results

Once the status is `completed`, the full results are available:

```bash
# Pod statuses
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" | jq -r '.result.pod_statuses' | jq .

# Kubernetes events
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" | jq -r '.result.events' | jq .

# Container logs
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" | jq -r '.result.log_tails' | jq .
```

### Reading Pod Statuses

Look for:

- **Phase**: `Pending` or `Failed` indicates problems
- **Conditions**: Check `Ready=False` with the reason
- **Containers**: Look for `restart_count > 0`, `state=waiting` with reasons like `CrashLoopBackOff`, or `state=terminated` with reason `OOMKilled`

### Reading Events

Filter for warnings:

```bash
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" \
  | jq -r '.result.events' \
  | jq '.[] | select(.event_type == "Warning")'
```

Common warning events: `FailedScheduling`, `Unhealthy`, `BackOff`, `FailedMount`.

### Reading Logs

Log tails are keyed by `pod-name/container-name`:

```bash
curl -s "http://localhost:3000/api/v1/diagnostics/${DIAG_ID}" \
  -H "Authorization: <admin-pak>" \
  | jq -r '.result.log_tails' \
  | jq 'to_entries[] | "\(.key):\n\(.value)\n---"' -r
```

Each container's last 100 log lines are included.

## Troubleshooting

**Diagnostic stays in `pending` state:**
- Check the agent's heartbeat — it may be disconnected
- Verify the agent is targeting the stack that contains the deployment object
- Check the agent logs for errors

**Diagnostic moves to `expired`:**
- The retention period elapsed before the agent could claim it
- Increase `retention_minutes` and try again
- Check if the agent is running and polling

**Diagnostic is `completed` but the result contains an `error` entry:**
- The agent claimed the request and then failed to collect data; it reports this through the result payload rather than a `failed` status
- Read the error string itself — it carries the Kubernetes API error verbatim
- Verify the agent has RBAC permissions to read pods, events, and logs in the target namespaces
- Check the agent logs for the same failure with surrounding context
- Fix the cause and issue a new diagnostic request; results are never retried in place

**Diagnostic is `failed`:**
- An agent claimed the request and never submitted a result before it expired. The agent process most likely died, was evicted, lost its credential, or was rescheduled mid-collection
- Check that agent's health and recent restarts; repeated `failed` diagnostics against one agent are evidence about that agent's stability, not about the diagnostic itself
- Note the distinction from `expired`, which means nobody ever claimed the request — that points at an agent being offline or not polling, rather than crashing
- A collection error does **not** produce `failed`; it produces a `completed` request with an error in the payload (Step 5)

## Cleanup

Diagnostics are automatically cleaned up by the broker's background task based on `broker.diagnostic_cleanup_interval_seconds` (default: 15 minutes) and `broker.diagnostic_max_age_hours` (default: 1 hour).

## Related Documentation

- [Diagnostics Reference](../reference/diagnostics.md) — complete API and data model reference
- [Monitoring Deployment Health](./deployment-health.md) — continuous health monitoring
- [Health Endpoints](../reference/health-endpoints.md) — health check configuration
