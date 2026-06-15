# Monitoring Deployment Health

Brokkr agents continuously monitor the health of deployed Kubernetes resources and report status back to the broker. This guide covers configuring health monitoring, interpreting health status, and troubleshooting common issues.

## How Health Monitoring Works

When an agent applies deployment objects to a Kubernetes cluster, it tracks those resources and periodically checks their health. The agent examines pod status, container states, and Kubernetes conditions to determine overall health. This information is reported to the broker, where it can be queried through the API (or any UI you build on top of it).

Health monitoring runs as a background process on each agent. On each check interval, the agent lists pods across all namespaces, attributes each pod to its deployment object — by the `brokkr.io/deployment-object-id` label or annotation when present, otherwise by walking the pod's ownerReference chain up to the Brokkr-applied top-level object — analyzes pod status, and sends a consolidated health report to the broker. Standard controller-managed workloads are attributed automatically; see the [Deployment Health Reference](../reference/deployment-health.md) for the exact discovery rules.

## Health Status Values

The health monitoring system reports one of four statuses: `healthy`, `degraded`, `failing`, or `unknown`. The agent derives these from pod conditions such as `ImagePullBackOff`, `CrashLoopBackOff`, and `OOMKilled`. See the [Deployment Health Reference](../reference/deployment-health.md) for the full status definitions, detected conditions, and the health summary schema.

## Configuring Health Monitoring

### Enabling Health Monitoring

Health monitoring is enabled by default. Configure it through Helm values:

```yaml
# Helm values for agent
agent:
  deploymentHealth:
    enabled: true
    intervalSeconds: 60
```

Or set environment variables directly:

```bash
BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED=true
BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL=60
```

### Adjusting Check Interval

The check interval determines how frequently the agent evaluates deployment health. The default is 60 seconds, which balances responsiveness with API load.

For environments where rapid detection is critical:

```yaml
agent:
  deploymentHealth:
    intervalSeconds: 30  # Check every 30 seconds
```

For large clusters with many deployments, increase the interval to reduce API load:

```yaml
agent:
  deploymentHealth:
    intervalSeconds: 120  # Check every 2 minutes
```

### Disabling Health Monitoring

To disable health monitoring entirely:

```yaml
agent:
  deploymentHealth:
    enabled: false
```

Disabling it removes broker visibility into deployment status.

## Viewing Health Status

### Via API

Query health status for a specific deployment object:

```bash
curl "http://broker:3000/api/v1/deployment-objects/{id}/health" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

Response:

```json
{
  "deployment_object_id": "a1b2c3d4-...",
  "overall_status": "healthy",
  "health_records": [
    {
      "agent_id": "e5f6g7h8-...",
      "status": "healthy",
      "summary": "{\"pods_ready\": 3, \"pods_total\": 3, \"conditions\": []}",
      "checked_at": "2025-01-02T10:00:00Z"
    }
  ]
}
```

### Understanding the Summary

The `summary` field reports how many pods were found and ready, which problematic conditions were detected, and optional per-resource detail. The field-by-field schema is documented in the [Deployment Health Reference](../reference/deployment-health.md#healthsummary).

## Common Scenarios

### ImagePullBackOff

When the agent reports `ImagePullBackOff`:

1. Verify the image name and tag are correct
2. Check that the image exists in the registry
3. Verify the cluster has network access to the registry
4. Check image pull secrets are configured correctly

```bash
# Check pod events for details
kubectl describe pod <pod-name> -n <namespace>

# Check image pull secrets
kubectl get secrets -n <namespace>
```

### CrashLoopBackOff

When containers repeatedly crash:

1. Check container logs for error messages:
   ```bash
   kubectl logs <pod-name> -n <namespace> --previous
   ```

2. Verify the application configuration is correct
3. Check resource limits aren't too restrictive
4. Ensure required environment variables and secrets are present

### OOMKilled

When containers are killed for memory:

1. Increase memory limits:
   ```yaml
   resources:
     limits:
       memory: "512Mi"  # Increase as needed
   ```

2. Investigate application memory usage
3. Consider memory profiling to identify leaks

### Unknown Status

When status shows as `unknown`:

1. Verify pods exist for the deployment object
2. Check the agent has RBAC permissions to list pods
3. Check agent logs for API errors:
   ```bash
   kubectl logs -l app.kubernetes.io/name=brokkr-agent -c agent
   ```

## Multi-Agent Deployments

When a deployment object is targeted to multiple agents, each agent reports its own health status. The broker stores health per agent, reflecting that the same deployment may have different health on different clusters.

The health endpoint always returns records from all reporting agents in the `health_records` array, along with an `overall_status` that reflects the aggregate state:

```bash
curl "http://broker:3000/api/v1/deployment-objects/{id}/health" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Webhook Integration

Health-status changes do not emit webhooks: health reports are stored by the broker and only surfaced through the health API. The closest webhook signal is `deployment.failed`, which fires when an agent posts a FAILURE apply event for a deployment object — that is, when applying the resources failed, not when monitored pods later become unhealthy:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Apply Failure Alerts",
    "url": "https://alerts.example.com/webhook",
    "event_types": ["deployment.failed"]
  }'
```

To alert on health degradation (e.g., `CrashLoopBackOff` after a successful apply), poll the health endpoints or feed cluster-side monitoring from the pods themselves.

## Troubleshooting

### Health Not Updating

If health status isn't updating:

1. Check the agent is running and connected:
   ```bash
   kubectl get pods -l app.kubernetes.io/name=brokkr-agent
   ```

2. Verify health monitoring is enabled:
   ```bash
   kubectl get configmap brokkr-agent-config -o yaml
   ```

3. Check agent logs for health check errors:
   ```bash
   kubectl logs -l app.kubernetes.io/name=brokkr-agent -c agent | grep -i health
   ```

### Incorrect Health Status

If reported health doesn't match actual pod status:

1. Verify pods have the correct deployment object ID label
2. Check the health check interval - status may be stale
3. Confirm the agent has permission to list pods across namespaces

### High API Load

If health monitoring causes excessive Kubernetes API load:

1. Increase the check interval
2. Consider reducing the number of deployment objects per agent
3. Monitor agent metrics for API call rates

## Related Documentation

- [Deployment Health Reference](../reference/deployment-health.md) - Statuses, conditions, and summary schema
- [Configuration Reference](../getting-started/configuration.md) - Agent configuration options
- [Architecture](../explanation/architecture.md) - How agents monitor health
- [Webhooks](./webhooks.md) - Alert on apply failures
