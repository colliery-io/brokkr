# Troubleshooting Reconciliation

This guide covers diagnosing common reconciliation problems: resources not being applied, resources not being deleted, and validation failures. For how reconciliation works conceptually, see [Reconciliation](../explanation/reconciliation.md).

## Resources Not Being Applied

If resources aren't appearing in your cluster:

1. **Check agent status**: Verify the agent is running and in ACTIVE status. Inactive agents skip deployment object requests.

2. **Check targeting**: Confirm the stack is associated with the agent—either explicitly via `GET /api/v1/agents/{id}/targets`, or dynamically through a shared label or annotation (see [Managing Stacks](./managing-stacks.md#label-based-targeting)).

3. **Check agent logs**: Look for validation errors or API failures in the agent container logs.

4. **Check events**: Query the broker for events related to the deployment object to see if failures were reported.

## Resources Not Being Deleted

If resources persist after stack deletion:

1. **Verify deletion marker**: Check that a deletion marker deployment object was created for the stack (stack deletion creates one automatically).

2. **Check annotations**: Verify the resources carry the `k8s.brokkr.io/stack` **annotation**. Resources without this annotation aren't managed by Brokkr. Because it's an annotation, not a label, `kubectl get -l k8s.brokkr.io/stack=...` will find nothing—inspect annotations instead:

   ```bash
   # List resources of a kind with their stack annotation, then filter
   kubectl get deployments -A -o jsonpath="{range .items[*]}{.metadata.namespace}{'\t'}{.metadata.name}{'\t'}{.metadata.annotations.k8s\.brokkr\.io/stack}{'\n'}{end}" \
     | grep "$STACK_ID"
   ```

   See [Agent Annotations and Labels](../reference/agent-annotations.md) for all keys the agent stamps and reads.

3. **Check owner references**: Resources with owner references are skipped during pruning. They'll be cleaned up when their owner is deleted.

## Validation Failures

If deployments fail validation:

1. **Check YAML syntax**: Ensure the YAML in your deployment object is valid.

2. **Check API versions**: Verify the apiVersion and kind are correct for your Kubernetes version.

3. **Check namespaces**: If referencing a namespace, ensure it's either included in the deployment object or already exists in the cluster.

4. **Check CRDs**: If using custom resources, ensure the CRD is either included in the deployment object or already installed.

## Drift Detection

Brokkr doesn't continuously monitor for drift—it only reconciles during polling cycles. If resources are modified outside of Brokkr:

- The next deployment object apply will restore the Brokkr-managed state
- Fields not managed by Brokkr (not in the deployment object) will be preserved
- To force a reconcile, submit the same deployment object content again (it gets a new sequence ID)

## Related Documentation

- [Reconciliation](../explanation/reconciliation.md) - How the reconciliation loop works
- [Agent Annotations Reference](../reference/agent-annotations.md) - Tracking keys on applied resources
- [Deployment Health](./deployment-health.md) - Monitoring applied resources
- [Managing Stacks](./managing-stacks.md) - Stack lifecycle and deletion
