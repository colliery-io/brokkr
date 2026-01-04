---
title: "Monitoring Deployment Health"
weight: 6
---

# Monitoring Deployment Health

Brokkr agents continuously monitor the health of deployed Kubernetes resources and report status back to the broker. This provides centralized visibility into deployment health across all clusters without requiring direct cluster access. This guide covers configuring health monitoring, interpreting health status, and troubleshooting common issues.

## How Health Monitoring Works

When an agent applies deployment objects to a Kubernetes cluster, it tracks those resources and periodically checks their health. The agent examines pod status, container states, and Kubernetes conditions to determine overall health. This information is reported to the broker, where it can be viewed through the API or UI.

Health monitoring runs as a background process on each agent. On each check interval, the agent queries the Kubernetes API for pods associated with each deployment object, analyzes their status, and sends a consolidated health report to the broker.

## Health Status Values

The health monitoring system reports one of four status values:

| Status | Description |
|--------|-------------|
| `healthy` | All pods are ready and running without issues |
| `degraded` | Some pods have issues but the deployment is partially functional |
| `failing` | The deployment has failed or all pods are in error states |
| `unknown` | Health cannot be determined (no pods found or API errors) |

### Detected Conditions

The agent detects and reports these problematic conditions:

**Container Issues:**
- `ImagePullBackOff` - Unable to pull container image
- `ErrImagePull` - Error pulling container image
- `CrashLoopBackOff` - Container repeatedly crashing
- `CreateContainerConfigError` - Invalid container configuration
- `InvalidImageName` - Malformed image reference
- `RunContainerError` - Error starting container
- `ContainerCannotRun` - Container failed to run

**Resource Issues:**
- `OOMKilled` - Container killed due to memory limits
- `Error` - Container exited with error

**Pod Issues:**
- `PodFailed` - Pod entered failed phase

## Configuring Health Monitoring

### Enabling Health Monitoring

Health monitoring is enabled by default. Configure it through environment variables:

```yaml
# Helm values for agent
agent:
  config:
    deploymentHealthEnabled: true
    deploymentHealthInterval: 60
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
  config:
    deploymentHealthInterval: 30  # Check every 30 seconds
```

For large clusters with many deployments, increase the interval to reduce API load:

```yaml
agent:
  config:
    deploymentHealthInterval: 120  # Check every 2 minutes
```

### Disabling Health Monitoring

To disable health monitoring entirely:

```yaml
agent:
  config:
    deploymentHealthEnabled: false
```

Note that disabling health monitoring means the broker will not have visibility into deployment status.

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
  "agent_id": "e5f6g7h8-...",
  "status": "healthy",
  "summary": {
    "pods_ready": 3,
    "pods_total": 3,
    "conditions": []
  },
  "checked_at": "2025-01-02T10:00:00Z"
}
```

### Understanding the Summary

The health summary provides details about pod status:

```json
{
  "pods_ready": 2,
  "pods_total": 3,
  "conditions": ["ImagePullBackOff"],
  "resources": [
    {
      "kind": "Pod",
      "name": "my-app-abc123",
      "namespace": "production",
      "ready": false,
      "message": "Back-off pulling image \"myapp:invalid\""
    }
  ]
}
```

| Field | Description |
|-------|-------------|
| `pods_ready` | Number of pods in Ready state |
| `pods_total` | Total number of pods found |
| `conditions` | List of detected problematic conditions |
| `resources` | Per-resource details (optional) |

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
   kubectl logs -l app=brokkr-agent -c agent
   ```

## Multi-Agent Deployments

When a deployment object is targeted to multiple agents, each agent reports its own health status. The broker stores health per agent, reflecting that the same deployment may have different health on different clusters.

Query all health reports for a deployment:

```bash
curl "http://broker:3000/api/v1/deployment-objects/{id}/health?all_agents=true" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Webhook Integration

Configure webhooks to receive notifications when deployment health changes:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Health Alerts",
    "url": "https://alerts.example.com/webhook",
    "event_types": ["deployment.failed"]
  }'
```

The `deployment.failed` event fires when a deployment transitions to failing status.

## Troubleshooting

### Health Not Updating

If health status isn't updating:

1. Check the agent is running and connected:
   ```bash
   kubectl get pods -l app=brokkr-agent
   ```

2. Verify health monitoring is enabled:
   ```bash
   kubectl get configmap brokkr-agent-config -o yaml
   ```

3. Check agent logs for health check errors:
   ```bash
   kubectl logs -l app=brokkr-agent -c agent | grep -i health
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

- [Configuration Reference](/getting-started/configuration) - Agent configuration options
- [Architecture](/explanation/architecture) - How agents monitor health
- [Webhooks](/how-to/webhooks) - Alert on health changes
