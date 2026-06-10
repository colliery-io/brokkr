# Deployment Health

Reference for the agent's deployment health monitoring: how pods are discovered, the status values reported, the conditions detected, and the health summary schema.

## Pod Discovery

On each check interval, the agent lists pods **across all namespaces** once and attributes each pod to a deployment object by, in order:

1. the `brokkr.io/deployment-object-id=<uuid>` **label** on the pod (manual opt-in),
2. the same key as an **annotation** on the pod itself (bare `Pod` manifests applied by Brokkr are stamped with it),
3. walking the pod's controller `ownerReferences` chain upward (Pod → ReplicaSet → Deployment, Job → Pod, StatefulSet/DaemonSet → Pod, up to four hops) until it reaches a Brokkr-applied object carrying the annotation.

Standard controller-managed workloads submitted through Brokkr are therefore attributed automatically — no manual labeling is required. A deployment object reports `unknown` only when no pods can be attributed to it (for example, before pods are scheduled, or for objects that create no pods).

Discovery is implemented in `crates/brokkr-agent/src/deployment_health.rs` (`discover_pods`).

If zero pods carry the label, the deployment object reports `unknown`.

## Status Values

| Status | Meaning |
|--------|---------|
| `healthy` | All discovered pods are ready; no problematic conditions detected |
| `degraded` | One or more pods have a detected problematic condition |
| `failing` | A pod entered the `Failed` phase (`PodFailed`) |
| `unknown` | No pods found for the label, a pod is in the `Unknown` phase, or the health check errored |

## Detected Conditions

### Degraded conditions (container waiting states)

Reported when a container (or init container) is waiting with one of these reasons:

- `ImagePullBackOff`
- `ErrImagePull`
- `CrashLoopBackOff`
- `CreateContainerConfigError`
- `InvalidImageName`
- `RunContainerError`
- `ContainerCannotRun`

Init container conditions are prefixed: `InitContainer:<reason>`.

### Terminated issues (container terminated states)

Reported when a container terminated with one of these reasons:

- `OOMKilled` (also detected from the container's last terminated state, catching recent crashes)
- `Error`
- `ContainerCannotRun`

### Pod-level conditions

- `PodFailed` — pod entered the `Failed` phase; sets the overall status to `failing`

## HealthSummary

Each health report carries a structured summary:

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

| Field | Type | Description |
|-------|------|-------------|
| `pods_ready` | integer | Number of discovered pods in Ready state |
| `pods_total` | integer | Total number of pods discovered by the label query |
| `conditions` | array of strings | De-duplicated list of detected problematic conditions |
| `resources` | array | Per-resource details for pods with degraded waiting conditions |

Each `resources` entry has:

| Field | Type | Description |
|-------|------|-------------|
| `kind` | string | Resource kind (`Pod`) |
| `name` | string | Resource name |
| `namespace` | string | Resource namespace |
| `ready` | boolean | Whether the resource is ready |
| `message` | string or null | Human-readable status message from Kubernetes |

## API Endpoints

```
GET /api/v1/deployment-objects/{id}/health   # health records + overall_status for one deployment object
GET /api/v1/stacks/{id}/health               # aggregated health for a stack
PATCH /api/v1/agents/{id}/health-status      # agent-side reporting endpoint
```

## Related Documentation

- [Monitoring Deployment Health](../how-to/deployment-health.md) — configuration and troubleshooting
- [Agent Annotations and Labels](./agent-annotations.md) — keys the agent stamps and looks for
