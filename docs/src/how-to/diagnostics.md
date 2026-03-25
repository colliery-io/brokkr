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

## Step 5: Read the Results

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

**Diagnostic moves to `failed`:**
- The agent encountered an error collecting data
- Check the agent logs for Kubernetes API errors
- Verify the agent has RBAC permissions to read pods, events, and logs

## Cleanup

Diagnostics are automatically cleaned up by the broker's background task based on `broker.diagnostic_cleanup_interval_seconds` (default: 15 minutes) and `broker.diagnostic_max_age_hours` (default: 1 hour).

## Related Documentation

- [Diagnostics Reference](../reference/diagnostics.md) — complete API and data model reference
- [Monitoring Deployment Health](./deployment-health.md) — continuous health monitoring
- [Health Endpoints](../reference/health-endpoints.md) — health check configuration
