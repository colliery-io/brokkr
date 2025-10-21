# Brokkr Agent Helm Chart

This Helm chart deploys the Brokkr agent to a Kubernetes cluster. The agent connects to a Brokkr broker and reports cluster state.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- A running Brokkr broker instance
- Broker Pre-Authenticated Key (PAK) for agent authentication

## Installation

### Basic Installation

Deploy with default settings (cluster-wide RBAC):

```bash
helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.pak=your-pak-token \
  --set broker.clusterName=production-cluster
```

### Installation with Custom Agent Name

```bash
helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.pak=your-pak-token \
  --set broker.clusterName=production-cluster \
  --set broker.agentName=prod-k8s-agent
```

## Configuration

### Broker Connection

The agent requires connection details to communicate with the broker:

```yaml
broker:
  url: http://brokkr-broker:3000  # Broker service URL
  agentName: ""                    # Optional agent identifier (auto-generated if empty)
  clusterName: ""                  # Cluster identifier for broker
  pak: ""                          # Pre-Authenticated Key for agent authentication
```

**Security Note**: The PAK is a sensitive credential. In production, use Kubernetes secrets:

```bash
kubectl create secret generic agent-credentials \
  --from-literal=pak=your-pak-token

helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.clusterName=production \
  --set broker.pak="" \
  --set-string 'extraEnv[0].name=BROKKR__BROKER__PAK' \
  --set-string 'extraEnv[0].valueFrom.secretKeyRef.name=agent-credentials' \
  --set-string 'extraEnv[0].valueFrom.secretKeyRef.key=pak'
```

### Agent Polling Configuration

Control how frequently the agent polls the broker:

```yaml
agent:
  pollingInterval: 30  # Seconds between broker polls
```

### RBAC Configuration

The agent requires Kubernetes API access to observe cluster state. Two modes are supported:

#### Cluster-Wide Access (Default)

Grants the agent access to all namespaces and cluster-scoped resources:

```yaml
rbac:
  create: true
  clusterWide: true
```

**Use when**:
- You want complete cluster visibility
- The agent should monitor all namespaces
- You have cluster-admin permissions to install

**Creates**: `ClusterRole` and `ClusterRoleBinding`

#### Namespace-Scoped Access

Restricts the agent to only the namespace where it's deployed:

```yaml
rbac:
  clusterWide: false
```

**Use when**:
- Operating in a multi-tenant cluster
- You want to limit the agent's scope
- You only have namespace-admin permissions

**Creates**: `Role` and `RoleBinding`

**Limitations**: Cannot access cluster-scoped resources (nodes, persistent volumes, cluster roles)

#### Custom Additional Permissions

Extend the agent's permissions for custom resources:

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

#### Disabling RBAC Creation

If you manage RBAC separately:

```yaml
rbac:
  create: false

serviceAccount:
  create: false
  name: my-existing-service-account
```

For detailed information about RBAC permissions and security implications, see [RBAC.md](./RBAC.md).

### Service Account Configuration

```yaml
serviceAccount:
  create: true
  name: ""  # Auto-generated if empty
```

To use an existing service account:

```yaml
serviceAccount:
  create: false
  name: my-service-account
```

### Resource Configuration

Configure resource requests and limits:

```yaml
resources:
  requests:
    memory: 128Mi
    cpu: 50m
  limits:
    memory: 256Mi
    cpu: 200m
```

### Security Context

The agent runs as a non-root user by default:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 10001
  fsGroup: 10001
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `image.repository` | string | `"ghcr.io/colliery-io/brokkr-agent"` | Container image repository |
| `image.tag` | string | `"latest"` | Container image tag |
| `image.pullPolicy` | string | `"IfNotPresent"` | Image pull policy |
| `replicaCount` | int | `1` | Number of agent replicas |
| `broker.url` | string | `"http://brokkr-broker:3000"` | Broker service URL |
| `broker.agentName` | string | `""` | Agent identifier (auto-generated if empty) |
| `broker.clusterName` | string | `""` | Cluster identifier |
| `broker.pak` | string | `""` | Pre-Authenticated Key for authentication |
| `agent.pollingInterval` | int | `30` | Polling interval in seconds |
| `rbac.create` | bool | `true` | Create RBAC resources |
| `rbac.clusterWide` | bool | `true` | Use ClusterRole (true) or Role (false) |
| `rbac.additionalRules` | array | `[]` | Additional RBAC rules |
| `serviceAccount.create` | bool | `true` | Create service account |
| `serviceAccount.name` | string | `""` | Service account name |
| `resources.requests.memory` | string | `"128Mi"` | Memory request |
| `resources.requests.cpu` | string | `"50m"` | CPU request |
| `resources.limits.memory` | string | `"256Mi"` | Memory limit |
| `resources.limits.cpu` | string | `"200m"` | CPU limit |

## Examples

### Development Setup

```bash
helm install dev-agent charts/brokkr-agent \
  --set broker.url=http://dev-broker:3000 \
  --set broker.pak=dev-pak-token \
  --set broker.clusterName=dev-cluster
```

### Production Setup with External Broker

```bash
helm install prod-agent charts/brokkr-agent \
  --set broker.url=https://broker.example.com \
  --set broker.pak=prod-pak-token \
  --set broker.clusterName=prod-us-east-1 \
  --set broker.agentName=prod-primary-agent \
  --set agent.pollingInterval=60 \
  --set resources.requests.memory=256Mi \
  --set resources.requests.cpu=100m
```

### Multi-Tenant Namespace-Scoped Deployment

```bash
helm install tenant-agent charts/brokkr-agent \
  --namespace tenant-namespace \
  --set broker.url=http://shared-broker:3000 \
  --set broker.pak=tenant-pak-token \
  --set broker.clusterName=shared-cluster \
  --set broker.agentName=tenant-a-agent \
  --set rbac.clusterWide=false
```

### Deployment with Custom Resources

```bash
helm install agent-with-crds charts/brokkr-agent \
  --set broker.url=http://broker:3000 \
  --set broker.pak=pak-token \
  --set broker.clusterName=cluster \
  --set-json 'rbac.additionalRules=[{"apiGroups":["custom.io"],"resources":["customresources"],"verbs":["get","list","watch"]}]'
```

## RBAC Permissions

The agent requires read-only access to Kubernetes resources for cluster observation. Default permissions include:

**Core API**: pods, namespaces, nodes, services, endpoints, configmaps, secrets, persistentvolumes, persistentvolumeclaims, events

**Apps API**: deployments, statefulsets, daemonsets, replicasets

**Batch API**: jobs, cronjobs

**Networking API**: ingresses, networkpolicies

**RBAC API**: roles, rolebindings, clusterroles, clusterrolebindings

**Verbs**: `get`, `list`, `watch` (read-only)

For detailed information about why each permission is needed and security implications, see [RBAC.md](./RBAC.md).

## Troubleshooting

### Agent Cannot Connect to Broker

**Symptom**: Agent logs show connection errors to broker

**Solutions**:
1. Verify broker URL is correct: `kubectl get configmap <release-name>-brokkr-agent -o yaml`
2. Check broker is accessible: `kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- curl http://broker:3000/healthz`
3. Verify network policies allow traffic from agent to broker

### Agent Cannot Access Kubernetes Resources

**Symptom**: Agent logs show "Forbidden" or "unauthorized" errors

**Solutions**:
1. Verify RBAC resources were created: `kubectl get clusterrole,clusterrolebinding -l app.kubernetes.io/name=brokkr-agent`
2. Test permissions: `kubectl auth can-i list pods --as=system:serviceaccount:<namespace>:<service-account>`
3. See [RBAC.md](./RBAC.md) for detailed troubleshooting

### Invalid PAK Token

**Symptom**: Agent logs show authentication errors

**Solutions**:
1. Verify PAK is correct in ConfigMap or Secret
2. Generate a new PAK from the broker
3. Ensure PAK has not expired

### Agent Not Polling

**Symptom**: Agent starts but doesn't poll broker

**Solutions**:
1. Check agent logs: `kubectl logs -l app.kubernetes.io/name=brokkr-agent --tail=100 -f`
2. Verify polling interval is configured: `kubectl get configmap <release-name>-brokkr-agent -o yaml`
3. Check if agent is stuck in a crash loop: `kubectl get pods -l app.kubernetes.io/name=brokkr-agent`

### Viewing Logs

```bash
kubectl logs -l app.kubernetes.io/name=brokkr-agent --tail=100 -f
```

## Security Considerations

1. **PAK Protection**: Store the PAK in Kubernetes secrets, not in values files
2. **RBAC Scope**: Use namespace-scoped mode (`rbac.clusterWide: false`) in multi-tenant environments
3. **Secret Access**: The agent has read access to all secrets in scope - see [RBAC.md](./RBAC.md) for mitigation strategies
4. **Resource Limits**: Configure appropriate resource limits to prevent resource exhaustion
5. **Network Policies**: Restrict agent network access to only the broker

## Uninstallation

```bash
helm uninstall my-agent
```

This removes all resources created by the chart.

## Architecture

```
┌─────────────────┐
│  Brokkr Broker  │
│                 │
└────────▲────────┘
         │
         │ HTTP/HTTPS
         │ PAK Auth
         │
┌────────┴────────┐
│  Brokkr Agent   │
│                 │
│  Control Loop:  │
│  1. Poll Broker │
│  2. Read K8s    │
│  3. Report Back │
└────────┬────────┘
         │
         │ RBAC
         │
┌────────▼────────┐
│ Kubernetes API  │
│                 │
│  Resources:     │
│  - Pods         │
│  - Deployments  │
│  - Services     │
│  - etc.         │
└─────────────────┘
```

## Development Phases

**Phase 1** (Complete): Basic agent deployment and broker connection
**Phase 2** (Current): Comprehensive RBAC for cluster observation
**Phase 3** (Future): Reconciliation operations with write permissions

See [RBAC.md](./RBAC.md) for information about future write permissions.
