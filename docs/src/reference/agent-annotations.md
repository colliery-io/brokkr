# Agent Annotations and Labels

The agent stamps tracking metadata onto every Kubernetes object it applies, and uses a small set of labels and annotations to discover workloads for health checks, diagnostics, and log streaming. This page catalogs those keys. Constants are defined in `crates/brokkr-agent/src/k8s/objects.rs` and `crates/brokkr-agent/src/pod_logs.rs`.

## Annotations Stamped on Every Applied Object

During reconciliation, `create_k8s_objects()` sets these five annotations on each object parsed from a deployment object's YAML before server-side apply:

| Annotation | Value | Used for |
|------------|-------|----------|
| `k8s.brokkr.io/stack` | Stack UUID | Identifying which stack owns the object; pruning queries; kube-event ownership resolution |
| `k8s.brokkr.io/deployment-checksum` | SHA-256 checksum of the deployment object's YAML | Pruning: managed objects whose checksum no longer matches the current target state are deleted |
| `brokkr.io/deployment-object-id` | Deployment object UUID | Linking the applied object back to its deployment object record |
| `k8s.brokkr.io/last-config-applied` | Debug serialization of the object as applied | Inspection/debugging |
| `brokkr.io/owner-id` | Agent UUID | Ownership verification before the agent deletes an object |

The agent merges these keys into the manifest's own `metadata.annotations` (`crates/brokkr-agent/src/k8s/objects.rs`), so user-declared annotations are preserved — unless they collide with one of the five Brokkr keys, which always win. Annotations inside nested templates (for example a Deployment's `spec.template.metadata.annotations`) are untouched — that is where workload-level opt-ins like log streaming belong.

These annotations are set on the top-level objects the agent applies (Deployments, Services, and so on). They do **not** propagate to pods created by controllers: a Deployment's pods carry only whatever labels and annotations its pod template declares.

## Keys the Agent Looks For

| Key | Kind | Where it must be | Effect |
|-----|------|------------------|--------|
| `brokkr.io/deployment-object-id=<uuid>` | **Label** | On pods | Diagnostics (`cli/commands.rs`) discover pods by this label selector, and deployment health honors it as a manual override. Health otherwise attributes pods automatically via their ownerReference chain (`deployment_health.rs`); diagnostics still require the label — without it they return empty results |
| `k8s.brokkr.io/stack=<stack-uuid>` | Annotation | On pods (for log streaming); on any object (for kube-event forwarding) | Marks the pod/object as belonging to a stack |
| `brokkr.io/stream-logs: "true"` | Annotation | On pods | Opts the pod into log streaming. Both this and the stack annotation must be present on the pod itself (`pod_logs.rs`) |

## Example: a Pod Template Opted Into Health and Log Streaming

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
        # diagnostics discovery / manual health override (UUID of the deployment object)
        brokkr.io/deployment-object-id: "7f5c0a31-93b2-4f64-a9d1-2c3e4b5a6d70"
      annotations:
        # log streaming opt-in: both annotations required on the pod
        k8s.brokkr.io/stack: "550e8400-e29b-41d4-a716-446655440000"
        brokkr.io/stream-logs: "true"
    spec:
      containers:
        - name: myapp
          image: myapp:1.2.3
```

The stack UUID is known before you author the manifest (the stack is created first). The deployment object UUID is generated when the deployment object is created, so labeling pods with it requires a second deployment object revision after the first one's ID is known — which is why health attribution walks ownerReferences automatically and the label is needed only for diagnostics.

## Pruning Semantics

During each reconciliation the agent lists the objects annotated with the stack's `k8s.brokkr.io/stack` value and deletes those whose `k8s.brokkr.io/deployment-checksum` no longer matches current target state (`crates/brokkr-agent/src/k8s/api.rs:reconcile_target_state`). Two exemptions:

- Objects with `metadata.ownerReferences` are never deleted by the agent — their owning controller is responsible for cleanup.
- Before deleting, the agent verifies `brokkr.io/owner-id` matches its own agent UUID (`objects.rs:verify_object_ownership`).

## Related Documentation

- [Understanding Reconciliation](../explanation/reconciliation.md) — the apply/prune loop
- [Streaming Pod Logs and Live Tail](../how-to/log-streaming.md) — using the log-streaming opt-in
- [Monitoring Deployment Health](../how-to/deployment-health.md) — health discovery and statuses
