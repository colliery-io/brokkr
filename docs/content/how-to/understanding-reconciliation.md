---
title: "Understanding Reconciliation"
weight: 7
---

# Understanding Reconciliation

Brokkr agents continuously reconcile the desired state defined in deployment objects with the actual state in Kubernetes clusters. This guide explains how the reconciliation loop works, how sequence IDs ensure correct ordering, how the system handles pre-existing resources, and what to do when things go wrong.

## The Reconciliation Model

Brokkr uses a pull-based reconciliation model where agents periodically fetch their target state from the broker and ensure their cluster matches. This differs from push-based systems where a central controller actively pushes changes—instead, each agent owns the reconciliation responsibility for its cluster.

The core loop runs at a configurable polling interval (default: 30 seconds). On each cycle, the agent fetches deployment objects from the broker, validates the resources, applies them using Kubernetes server-side apply, and reports the result back to the broker as an event. This cycle continues indefinitely, ensuring clusters stay aligned with the desired state even if drift occurs.

## Polling and Target State

When an agent polls for its target state, it receives deployment objects from all stacks it's targeting. The broker returns objects ordered by sequence ID, ensuring the agent processes them in the order they were created.

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
   - `brokkr.io/stack-id`: Links the resource to its stack
   - `brokkr.io/checksum`: Identifies which deployment object version created it

5. **Pruning**: After applying desired resources, the agent queries the cluster for all resources with the stack annotation and deletes any that don't match the current checksum. This removes resources that were part of previous deployments but aren't in the current desired state.

## Handling Pre-Existing Resources

When Brokkr encounters resources that already exist in the cluster, several scenarios arise:

**Resources without Brokkr annotations**: Server-side apply succeeds, and Brokkr adds its annotations. The resource becomes managed by Brokkr going forward. Any fields not specified in the deployment object remain unchanged.

**Resources with matching annotations**: The apply is idempotent—the resource is updated to match the desired state. If the content hasn't changed, this is effectively a no-op.

**Resources with different checksum**: The resource was created by a previous deployment object. It gets updated with the new content and checksum. If the resource is no longer in the desired state, it gets pruned during the cleanup phase.

**Resources with owner references**: During pruning, Brokkr skips resources that have owner references. These resources are managed by Kubernetes controllers and will be garbage collected when their owner is deleted.

## Rollback on Failure

If reconciliation fails partway through applying resources, the agent attempts to roll back changes to maintain cluster consistency:

**Namespace rollback**: If the agent created new namespaces as part of this reconciliation and a later step fails, those namespaces are deleted. This prevents orphaned namespaces from accumulating after failed deployments.

**No resource rollback**: Individual resources that were successfully applied are not rolled back. This means partial applies can occur. The next reconciliation cycle will attempt the full apply again.

**Error reporting**: Failed reconciliation generates a failure event that's sent to the broker. The event includes the error message, enabling visibility into what went wrong through the broker's API or webhooks.

## Deletion Markers

When a stack is deleted, Brokkr creates a special deployment object with `is_deletion_marker: true`. When the agent receives this:

1. The agent identifies all cluster resources belonging to that stack
2. Each resource is deleted from the cluster
3. A success event is reported to the broker

Deletion markers ensure resources are cleaned up even if the agent was offline when the stack was deleted. The agent will process the deletion marker on its next poll and remove the resources.

## Configuration Options

### Polling Interval

Control how frequently the agent checks for changes:

```yaml
agent:
  polling_interval: 30  # seconds
```

Shorter intervals mean faster propagation of changes but higher API load on both the broker and Kubernetes API server. For production deployments, 30-60 seconds is typically appropriate.

### Retry Behavior

The agent implements exponential backoff for transient Kubernetes API errors:

- Initial retry interval: 1 second
- Maximum retry interval: 60 seconds
- Maximum elapsed time: 5 minutes
- Backoff multiplier: 2.0

Retryable errors include:
- HTTP 429 (Too Many Requests)
- HTTP 500 (Internal Server Error)
- HTTP 503 (Service Unavailable)
- HTTP 504 (Gateway Timeout)

Non-retryable errors fail immediately and are reported to the broker.

## Troubleshooting Reconciliation Issues

### Resources Not Being Applied

If resources aren't appearing in your cluster:

1. **Check agent status**: Verify the agent is running and in ACTIVE status. Inactive agents skip deployment object requests.

2. **Check targeting**: Confirm the stack is targeted to the agent via `GET /api/v1/agents/{id}/targets`.

3. **Check agent logs**: Look for validation errors or API failures in the agent container logs.

4. **Check events**: Query the broker for events related to the deployment object to see if failures were reported.

### Resources Not Being Deleted

If resources persist after stack deletion:

1. **Verify deletion marker**: Check that a deletion marker deployment object was created for the stack.

2. **Check annotations**: Verify the resources have the `brokkr.io/stack-id` annotation. Resources without this annotation aren't managed by Brokkr.

3. **Check owner references**: Resources with owner references are skipped during pruning. They'll be cleaned up when their owner is deleted.

### Validation Failures

If deployments fail validation:

1. **Check YAML syntax**: Ensure the YAML in your deployment object is valid.

2. **Check API versions**: Verify the apiVersion and kind are correct for your Kubernetes version.

3. **Check namespaces**: If referencing a namespace, ensure it's either included in the deployment object or already exists in the cluster.

4. **Check CRDs**: If using custom resources, ensure the CRD is either included in the deployment object or already installed.

### Drift Detection

Brokkr doesn't continuously monitor for drift—it only reconciles during polling cycles. If resources are modified outside of Brokkr:

- The next deployment object apply will restore the Brokkr-managed state
- Fields not managed by Brokkr (not in the deployment object) will be preserved
- To force a reconcile, submit the same deployment object content again (it gets a new sequence ID)

## Related Documentation

- [Core Concepts](/explanation/core-concepts) - Understanding the pull-based model
- [Deployment Health](/how-to/deployment-health) - Monitoring applied resources
- [Quick Start Guide](/getting-started/quick-start) - First deployment walkthrough
- [Managing Stacks](/how-to/managing-stacks) - Stack lifecycle and deletion
