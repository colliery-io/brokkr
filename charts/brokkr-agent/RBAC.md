# Brokkr Agent RBAC Permissions

This document explains the Role-Based Access Control (RBAC) permissions required by the Brokkr agent and the security implications of each permission.

## Overview

The Brokkr agent is a Kubernetes control loop that:
1. Polls the broker for desired state instructions
2. Reads the Kubernetes API to gather current cluster state
3. Reports state back to the broker
4. (Future) Executes reconciliation actions based on broker directives

The agent requires read-only access to various Kubernetes resources to perform cluster observation and reporting. All permissions follow the principle of least privilege.

## Permission Justification

### Core API Group (`""`)

#### Pods and Pod Subresources
**Resources**: `pods`, `pods/log`, `pods/status`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- `pods`: Inventory of all running workloads in the cluster
- `pods/log`: Access to container logs for troubleshooting and observability
- `pods/status`: Current health and resource usage of pods

**Data collected**:
- Pod names, namespaces, labels, and annotations
- Container images and resource requests/limits
- Pod phase and conditions (Running, Pending, Failed, etc.)
- Container logs (when troubleshooting issues)
- Resource usage metrics

**Security implications**:
- Log access may expose sensitive application data
- Secrets referenced by pods are visible in pod spec (but values are not accessible unless secrets permission is also granted)

#### Namespaces
**Resources**: `namespaces`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Understand cluster organization and multi-tenancy boundaries
- Group resources by namespace for reporting
- Detect namespace creation/deletion events

**Data collected**:
- Namespace names, labels, and annotations
- Namespace status and phase

**Security implications**: Minimal - namespace metadata is generally non-sensitive

#### Nodes
**Resources**: `nodes`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Understand cluster topology and capacity
- Monitor node health and resource availability
- Track node conditions (Ready, MemoryPressure, DiskPressure, etc.)

**Data collected**:
- Node names, labels, and annotations
- Node capacity and allocatable resources
- Node conditions and status
- Kubelet version and OS information

**Security implications**: Node information is generally non-sensitive but reveals cluster infrastructure details

#### Services and Endpoints
**Resources**: `services`, `endpoints`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Understand service topology and networking
- Map services to backing pods
- Detect service configuration changes

**Data collected**:
- Service names, types, and selectors
- Service ClusterIPs and external IPs
- Endpoint addresses and readiness

**Security implications**: Service topology may reveal application architecture

#### ConfigMaps
**Resources**: `configmaps`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Detect configuration changes that may affect applications
- Understand application configuration patterns

**Data collected**:
- ConfigMap names and keys (not values in basic observation)
- ConfigMap metadata

**Security implications**: ConfigMaps may contain sensitive configuration data; agent reads both metadata and data

#### Secrets
**Resources**: `secrets`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Inventory secrets for security posture assessment
- Detect secret creation/rotation events
- Identify unused or orphaned secrets

**Data collected**:
- Secret names and types
- Secret metadata (labels, annotations)
- **Full secret data** (base64-decoded values)

**Security implications**: **HIGH RISK** - Agent has access to all secret values including passwords, API keys, certificates, etc. This permission should be carefully considered based on your security requirements.

**Mitigation options**:
- Use namespace-scoped mode (`rbac.clusterWide: false`) to limit secret access to specific namespaces
- Remove secrets from the RBAC rules if secret access is not required for your use case
- Implement additional audit logging for secret access
- Use tools like sealed-secrets or external secret managers to reduce cluster secret exposure

#### Persistent Volumes and Claims
**Resources**: `persistentvolumes`, `persistentvolumeclaims`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Track storage usage and capacity
- Monitor PVC binding and status
- Detect storage-related issues

**Data collected**:
- PV/PVC names, capacities, and storage classes
- PVC binding status
- Volume modes and access modes

**Security implications**: Minimal - volume metadata is generally non-sensitive

### Apps API Group (`apps`)

**Resources**: `deployments`, `deployments/status`, `statefulsets`, `statefulsets/status`, `daemonsets`, `daemonsets/status`, `replicasets`, `replicasets/status`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Inventory application workloads
- Monitor rollout status and health
- Detect scaling events and configuration changes

**Data collected**:
- Workload specifications and desired state
- Current replica counts and rollout status
- Update strategies and history
- Pod template specifications

**Security implications**: Workload specs may reveal application architecture and scaling patterns

### Batch API Group (`batch`)

**Resources**: `jobs`, `jobs/status`, `cronjobs`, `cronjobs/status`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Monitor batch job execution
- Track cronjob schedules and history
- Detect failed or stuck jobs

**Data collected**:
- Job specifications and status
- Cronjob schedules and last execution times
- Job success/failure counts

**Security implications**: Job specs may reveal batch processing logic and schedules

### Networking API Group (`networking.k8s.io`)

**Resources**: `ingresses`, `ingresses/status`, `networkpolicies`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Understand external access patterns
- Monitor ingress health and routing
- Assess network security policies

**Data collected**:
- Ingress rules and TLS configuration
- Network policy allow/deny rules
- Service routing and load balancer status

**Security implications**: Ingress configuration may reveal external endpoints and routing logic; network policies reveal security boundaries

### RBAC API Group (`rbac.authorization.k8s.io`)

**Resources**: `roles`, `rolebindings`, `clusterroles`, `clusterrolebindings`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Security posture assessment
- Detect overly permissive roles
- Audit RBAC configuration changes
- Identify service accounts and their permissions

**Data collected**:
- Role definitions and permissions
- Role bindings and subject assignments
- Service account to role mappings

**Security implications**: RBAC configuration reveals security boundaries but is necessary for security assessment; this is read-only access and cannot modify permissions

### Events

**Resources**: `events`
**Verbs**: `get`, `list`, `watch`

**Why needed**:
- Change tracking and audit trail
- Debugging cluster issues
- Correlating resource changes with events

**Data collected**:
- Event messages, reasons, and types
- Event sources and affected objects
- Event timestamps

**Security implications**: Minimal - events are primarily diagnostic information

## Configuration Options

### Cluster-Wide vs Namespace-Scoped

The agent supports two RBAC modes:

#### Cluster-Wide (Default)
```yaml
rbac:
  clusterWide: true
```

**Creates**: `ClusterRole` and `ClusterRoleBinding`
**Scope**: Access to all namespaces and cluster-scoped resources
**Use case**: Central cluster observability and management

**Advantages**:
- Complete cluster visibility
- Can monitor all workloads across namespaces
- Simpler configuration

**Disadvantages**:
- Requires cluster-admin permissions to install
- May not be acceptable in strict multi-tenant environments

#### Namespace-Scoped
```yaml
rbac:
  clusterWide: false
```

**Creates**: `Role` and `RoleBinding`
**Scope**: Access only to resources in the release namespace
**Use case**: Restricted environments or namespace-specific monitoring

**Advantages**:
- Minimal permissions required
- Suitable for multi-tenant clusters
- Can be deployed by namespace admins

**Disadvantages**:
- Cannot access cluster-scoped resources (nodes, persistent volumes, cluster roles)
- Cannot monitor workloads in other namespaces
- Limited cluster visibility

**Important**: Cluster-scoped resources (nodes, namespaces, persistentvolumes, clusterroles, clusterrolebindings) are automatically excluded from the Role when using namespace-scoped mode. These resources cannot be accessed with a namespace-scoped Role.

**Current Limitation**: The agent currently requires cluster-wide access to function properly. Namespace-scoped mode and disabled RBAC will cause the agent to fail during startup. This will be addressed in a future release to allow the agent to operate with limited permissions.

### Additional Custom Rules

You can extend the agent's permissions with custom rules:

```yaml
rbac:
  additionalRules:
    - apiGroups: ["custom.io"]
      resources: ["customresources"]
      verbs: ["get", "list", "watch"]
    - apiGroups: [""]
      resources: ["resourcequotas"]
      verbs: ["get", "list"]
```

**Use cases**:
- Support for custom resource definitions (CRDs)
- Additional Kubernetes resources not included by default
- Vendor-specific API groups

## Security Best Practices

1. **Principle of Least Privilege**: Only grant permissions actually needed for your use case. If you don't need log access or secret access, remove those resources from the RBAC rules.

2. **Namespace-Scoped for Multi-Tenancy**: Use `rbac.clusterWide: false` in multi-tenant clusters where full cluster access is not appropriate.

3. **Regular Auditing**: Periodically review the agent's RBAC permissions and ensure they align with current requirements.

4. **Secret Access Considerations**: The agent has read access to all secrets. Consider:
   - Using external secret management (AWS Secrets Manager, HashiCorp Vault, etc.)
   - Using sealed-secrets to reduce secret exposure in the cluster
   - Removing secret access from RBAC if not needed for your use case

5. **Monitoring and Logging**: Enable Kubernetes audit logging to track what the agent accesses.

6. **Service Account Security**: Never share the agent's service account with other workloads.

## Future Write Permissions (Phase 3+)

The current RBAC configuration is **read-only**. Future versions of Brokkr will support reconciliation operations that require write permissions:

**Potential future verbs**: `create`, `update`, `patch`, `delete`
**Target resources**: Resources that the broker will manage

When write permissions are added:
- They will be disabled by default and require explicit opt-in
- More granular resource name restrictions will be implemented
- Audit logging will be enhanced for all write operations
- Separate roles may be created for read vs. write operations

## Testing RBAC Configuration

### Verify RBAC Resources are Created

```bash
# Check ClusterRole/Role
kubectl get clusterrole <release-name>-brokkr-agent
# or
kubectl get role <release-name>-brokkr-agent -n <namespace>

# Check ClusterRoleBinding/RoleBinding
kubectl get clusterrolebinding <release-name>-brokkr-agent
# or
kubectl get rolebinding <release-name>-brokkr-agent -n <namespace>
```

### Test Agent Can Access Resources

```bash
# Get the service account name
SA_NAME=$(kubectl get sa -n <namespace> -l app.kubernetes.io/name=brokkr-agent -o jsonpath='{.items[0].metadata.name}')

# Test pod access
kubectl auth can-i list pods --as=system:serviceaccount:<namespace>:$SA_NAME

# Test secret access
kubectl auth can-i get secrets --as=system:serviceaccount:<namespace>:$SA_NAME
```

### Test Permission Failures

To test that the agent fails gracefully without permissions:

1. Deploy with `rbac.create: false` and a service account without permissions
2. Check agent logs for permission errors
3. Verify agent reports the permission issue to the broker

## Troubleshooting

### Agent Cannot Access Resources

**Symptom**: Agent logs show "Forbidden" or "unauthorized" errors

**Solutions**:
1. Verify RBAC resources were created: `kubectl get clusterrole,clusterrolebinding -l app.kubernetes.io/name=brokkr-agent`
2. Check service account is correctly bound: `kubectl get clusterrolebinding <release-name>-brokkr-agent -o yaml`
3. Verify cluster RBAC is enabled: `kubectl cluster-info dump | grep authorization-mode`

### Namespace-Scoped Mode Cannot Access Cluster Resources

**Symptom**: Agent cannot see nodes, cluster roles, or persistent volumes

**Explanation**: This is expected behavior in namespace-scoped mode. These resources are cluster-scoped and cannot be accessed with a namespace-scoped Role.

**Solution**: Use `rbac.clusterWide: true` if cluster-wide visibility is required.

### Custom Resources Not Accessible

**Symptom**: Agent cannot access custom resources (CRDs)

**Solution**: Add the CRD's API group and resources to `rbac.additionalRules`:
```yaml
rbac:
  additionalRules:
    - apiGroups: ["your.custom.api"]
      resources: ["yourresources"]
      verbs: ["get", "list", "watch"]
```

## References

- [Kubernetes RBAC Documentation](https://kubernetes.io/docs/reference/access-authn-authz/rbac/)
- [Service Accounts](https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/)
- [Authorization Overview](https://kubernetes.io/docs/reference/access-authn-authz/authorization/)
