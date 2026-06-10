# Reconciliation

Brokkr agents continuously reconcile the desired state defined in deployment objects with the actual state in Kubernetes clusters. This page explains how the reconciliation loop works, how sequence IDs ensure correct ordering, how the system handles pre-existing resources, and what happens when an apply fails. For diagnosing specific problems, see [Troubleshooting Reconciliation](../how-to/troubleshoot-reconciliation.md).

## The Reconciliation Model

Brokkr uses a pull-based reconciliation model where agents periodically fetch their target state from the broker and ensure their cluster matches. This differs from push-based systems where a central controller actively pushes changes—instead, each agent owns the reconciliation responsibility for its cluster.

The core loop runs at a configurable polling interval (default: 10 seconds; see the [Configuration Guide](../getting-started/configuration.md)). On each cycle, the agent fetches deployment objects from the broker, validates the resources, applies them using Kubernetes server-side apply, and reports the result back to the broker as an event. This cycle continues indefinitely, ensuring clusters stay aligned with the desired state even if drift occurs. Transient Kubernetes API errors (429, 500, 503, 504) are retried with exponential backoff; non-retryable errors fail immediately and are reported to the broker.

## Polling and Target State

When an agent polls for its target state, it receives deployment objects from all stacks it's associated with. The broker returns objects ordered by sequence ID, ensuring the agent processes them in the order they were created.

```
Agent -> Broker: GET /api/v1/agents/{id}/target-state
Broker -> Agent: [DeploymentObject, DeploymentObject, ...]
```

Each deployment object contains the complete YAML content to be applied and a checksum calculated from that content. The checksum serves as a version identifier—resources in the cluster are annotated with the checksum so the agent can identify which version they belong to.

## Sequence IDs and Ordering

Every deployment object receives a monotonically increasing sequence ID when created. This provides a global ordering that's essential for correct reconciliation:

- The agent processes objects in sequence order
- Later objects supersede earlier ones for the same stack
- The most recent (highest sequence ID) deployment object represents current desired state

When multiple deployment objects exist for a stack, the agent only needs to apply the most recent one. Earlier objects are historical records useful for audit purposes but don't affect the current cluster state.

## The Apply Process

For each deployment object, the reconciliation proceeds through several stages:

1. **Priority Resource Application**: Namespaces and CustomResourceDefinitions are applied first. These resources must exist before namespaced resources can be validated or created.

2. **Validation**: Remaining resources are validated using a Kubernetes dry-run apply. This catches schema errors, missing fields, and other issues before attempting the real apply.

3. **Server-Side Apply**: Resources are applied using Kubernetes server-side apply with force enabled. This approach allows Brokkr to take ownership of fields it manages while preserving fields managed by other controllers.

4. **Annotation Injection**: Each applied resource receives annotations identifying its stack and the deployment object checksum:
   - `k8s.brokkr.io/stack`: Links the resource to its stack
   - `k8s.brokkr.io/deployment-checksum`: Identifies which deployment object version created it

   Both keys are annotations, not labels. See [Agent Annotations and Labels](../reference/agent-annotations.md) for the full set.

5. **Pruning**: After applying desired resources, the agent queries the cluster for all resources with the stack annotation and deletes any that don't match the current checksum. This removes resources that were part of previous deployments but aren't in the current desired state.

## Handling Pre-Existing Resources

When Brokkr encounters resources that already exist in the cluster, several scenarios arise:

**Resources without Brokkr annotations**: Server-side apply succeeds, and Brokkr adds its annotations. The resource becomes managed by Brokkr going forward. Any fields not specified in the deployment object remain unchanged.

**Resources with matching annotations**: The apply is idempotent—the resource is updated to match the desired state. If the content hasn't changed, this is effectively a no-op.

**Resources with different checksum**: The resource was created by a previous deployment object. It gets updated with the new content and checksum. If the resource is no longer in the desired state, it gets pruned during the cleanup phase.

**Resources with owner references**: During pruning, Brokkr skips resources that have owner references. These resources are managed by Kubernetes controllers and will be garbage collected when their owner is deleted.

## Rollback on Failure

If reconciliation fails partway through—during priority resource application, validation, or the main apply—the agent attempts a limited rollback to avoid leaving half-created scaffolding behind:

**Namespace rollback**: The agent deletes namespaces it *created* during the failed attempt, on a best-effort basis (deletion failures are logged and ignored). Before applying a declared Namespace, the agent checks whether it already exists; pre-existing namespaces are excluded from rollback, and when the existence check itself fails, the agent errs on the side of not deleting.

**No resource rollback**: Individual resources that were successfully applied are not rolled back. This means partial applies can occur. The next reconciliation cycle will attempt the full apply again.

**Error reporting**: Failed reconciliation generates a failure event that's sent to the broker. The event includes the error message, enabling visibility into what went wrong through the broker's API or webhooks.

## Deletion Markers

When a stack is deleted, Brokkr creates a special deployment object with `is_deletion_marker: true`. When the agent receives this:

1. The agent identifies all cluster resources belonging to that stack
2. Each resource is deleted from the cluster
3. A success event is reported to the broker

Deletion markers ensure resources are cleaned up even if the agent was offline when the stack was deleted. The agent will process the deletion marker on its next poll and remove the resources.

## Drift

Brokkr doesn't continuously monitor for drift—it only reconciles during polling cycles. If resources are modified outside of Brokkr, the next deployment object apply restores the Brokkr-managed state, while fields not managed by Brokkr (not in the deployment object) are preserved. Submitting the same deployment object content again (which gets a new sequence ID) forces a reconcile.

## Related Documentation

- [Troubleshooting Reconciliation](../how-to/troubleshoot-reconciliation.md) - Diagnosing apply and deletion problems
- [Core Concepts](./core-concepts.md) - Understanding the pull-based model
- [Configuration Guide](../getting-started/configuration.md) - Polling interval and agent settings
- [Managing Stacks](../how-to/managing-stacks.md) - Stack lifecycle and deletion
