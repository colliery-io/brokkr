# Installing Brokkr

This guide will help you install Brokkr using Helm, the recommended installation method.

## Prerequisites

Before installing Brokkr, ensure you have:

- **Kubernetes cluster** (v1.29 or later — the agent chart declares `kubeVersion: ">=1.29.0-0"`)
- **kubectl** CLI configured to access your cluster
- **Helm** 3.8 or later installed ([installation guide](https://helm.sh/docs/intro/install/))

### Verifying Prerequisites

```bash
# Check Kubernetes version
kubectl version

# Check Helm version
helm version --short

# Verify cluster access
kubectl cluster-info
```

## Quick Start

Get a broker and agent running in your cluster in under 10 minutes.

### 1. Install the Broker

Install the broker with bundled PostgreSQL for development:

```bash
# Install broker with bundled PostgreSQL
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set postgresql.enabled=true \
  --wait

# Verify broker is running
kubectl get pods -l app.kubernetes.io/name=brokkr-broker
```

Expected output:
```
NAME                             READY   STATUS    RESTARTS   AGE
brokkr-broker-xxxxxxxxxx-xxxxx   1/1     Running   0          2m
```

### 2. Get Broker URL

```bash
# Port forward to access the broker locally
kubectl port-forward svc/brokkr-broker 3000:3000 &

# The broker is now accessible at http://localhost:3000
```

### 3. Get the Admin PAK

Every `/api/v1` request must carry a PAK (Prefixed API Key) in the `Authorization` header. On first startup the broker creates the admin role:

- **If you installed with the commands above** (no `broker.pakHash` chart value), the broker's embedded default configuration supplies a publicly known hash, and the admin PAK is `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8` — fine for a throwaway dev cluster, **never for production**.
- **If you explicitly set `broker.pakHash` to an empty value**, the broker generates a fresh admin PAK at first startup and writes it to `/tmp/brokkr-keys/key.txt` inside the broker container (the file does not exist otherwise):

```bash
kubectl exec deploy/brokkr-broker -- cat /tmp/brokkr-keys/key.txt
```
- **If you set `broker.pakHash` to the hash of your own PAK**, use that PAK.

Export it for the following steps:

```bash
export ADMIN_PAK="<your-admin-pak>"
```

### 4. Create an Agent and Get Its PAK

Create an agent registration and retrieve its PAK:

```bash
# Create a new agent
curl -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-agent",
    "cluster_name": "development"
  }'
```

The response wraps the agent record and the one-time PAK in `initial_pak`:
```json
{
  "agent": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-agent",
    "cluster_name": "development",
    "status": "INACTIVE"
  },
  "initial_pak": "brokkr_BRxxxxxxxx_yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
}
```

Save the `initial_pak` value (`jq -r '.initial_pak'`) — it is shown only once; you'll need it to install the agent.

### 5. Install the Agent

Install the agent using the `initial_pak` from step 4:

```bash
# Install agent (replace <PAK> with the initial_pak from step 4)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  --wait

# Verify agent is running
kubectl get pods -l app.kubernetes.io/name=brokkr-agent
```

Expected output:
```
NAME                             READY   STATUS    RESTARTS   AGE
brokkr-agent-xxxxxxxxxxx-xxxxx   1/1     Running   0          1m
```

### 6. Verify Installation

Check that both components are healthy:

```bash
# Check broker health
kubectl exec deploy/brokkr-broker -- wget -qO- http://localhost:3000/healthz

# Check agent health
kubectl exec deploy/brokkr-agent -- wget -qO- http://localhost:8080/healthz

# View agent registration in broker logs
kubectl logs deploy/brokkr-broker | grep -i agent
```

You should see "OK" from both health checks and agent registration messages in the broker logs.

## Detailed Installation

### Broker Installation

The broker is the central management service that coordinates deployments across your Kubernetes clusters.

#### Development Setup (Bundled PostgreSQL)

For development and testing, use the bundled PostgreSQL:

```bash
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set postgresql.enabled=true \
  --set postgresql.auth.password=brokkr \
  --wait
```

#### Using Provided Values Files

Brokkr includes pre-configured values files for different environments — development (bundled PostgreSQL, minimal resources), staging (external PostgreSQL, moderate resources), and production (external PostgreSQL, production-grade resources). Install with the one for your environment:

```bash
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-broker/values/<environment>.yaml
```

You can also download these files and customize them:

```bash
# Download development values
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-broker/values/development.yaml

# Edit as needed
vi development.yaml

# Install with custom values
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  -f development.yaml
```

View all available values files:
- [Development](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-broker/values/development.yaml)
- [Staging](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-broker/values/staging.yaml)
- [Production](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-broker/values/production.yaml)

### Agent Installation

The agent runs in each Kubernetes cluster you want to manage and communicates with the broker.

#### Basic Agent Installation

```bash
# Create agent via broker API (see Quick Start step 4)
# Then install with the returned initial_pak:

helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK_FROM_BROKER>" \
  --wait
```

#### Using Provided Values Files

Brokkr includes pre-configured values files for agents — development (minimal resources, cluster-wide RBAC), staging (moderate resources), and production (production-grade resources). Install with the one for your environment:

```bash
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-agent/values/<environment>.yaml
```

View all available agent values files:
- [Development](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/development.yaml)
- [Staging](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/staging.yaml)
- [Production](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/production.yaml)

## Chart Versions, Upgrades, and Uninstallation

For pinning chart versions, installing development builds, upgrading, and uninstalling, see [Installing, Upgrading, and Uninstalling with Helm](../how-to/install-operations.md).

## Verifying the Installation

For the broker/agent health checks and connectivity verification, see [Quick Start step 6](#6-verify-installation).

### Test Deployment

Create a test namespace to verify end-to-end functionality:

```bash
# Port forward to broker
kubectl port-forward svc/brokkr-broker 3000:3000 &

# Stacks require an owning generator. Look up the admin-generator the broker
# created at first startup (it is linked to the admin PAK):
GEN_ID=$(curl -s http://localhost:3000/api/v1/generators \
  -H "Authorization: Bearer $ADMIN_PAK" \
  | jq -r '.[] | select(.name=="admin-generator") | .id')

# Create a stack
STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"test-stack\", \"description\": \"Test stack\", \"generator_id\": \"$GEN_ID\"}" \
  | jq -r '.id')

# Target your agent to the stack so it receives the deployment
AGENT_ID=$(curl -s http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" | jq -r '.[0].id')

curl -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/targets \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\", \"stack_id\": \"$STACK_ID\"}"

# Deploy a test namespace
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-test",
    "is_deletion_marker": false
  }'

# Verify namespace was created
kubectl get namespace brokkr-test

# Clean up
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-test",
    "is_deletion_marker": true
  }'

kubectl get namespace brokkr-test  # Should show Terminating/NotFound
```

## Configuration Reference

### Broker Values

Key configuration options for the broker chart:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `postgresql.enabled` | Enable bundled PostgreSQL | `true` |
| `postgresql.auth.password` | PostgreSQL password (bundled) | `brokkr` |
| `postgresql.external.host` | External database host | `""` |
| `postgresql.external.port` | External database port | `5432` |
| `postgresql.external.database` | Database name | `brokkr` |
| `postgresql.external.username` | Database username | `brokkr` |
| `postgresql.external.password` | Database password | `brokkr` |
| `postgresql.external.schema` | PostgreSQL schema (multi-tenant) | `""` |
| `replicaCount` | Number of broker replicas | `1` |
| `image.tag` | Image tag to use | `latest` |
| `broker.logLevel` | Log level | `info` |
| `broker.webhookEncryptionKey` | Hex-encoded 32-byte key for webhook secrets at rest; a random key is generated on every boot if unset | unset |
| `configReload.enabled` | Watch the ConfigMap and reload hot-reloadable settings automatically | `true` |
| `configReload.debounceSeconds` | Debounce window for successive reloads | `5` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `tls.enabled` | Enable TLS | `false` |

### Agent Values

Key configuration options for the agent chart:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `broker.url` | Broker URL | `http://brokkr-broker:3000` |
| `broker.pak` | Agent PAK (Prefixed API Key) | **Required** |
| `broker.agentName` | Human-readable agent name | `""` |
| `broker.clusterName` | Name of the managed cluster | `""` |
| `agent.pollingInterval` | Seconds between broker polls (the agent binary's own default is `10`) | `30` |
| `agent.deploymentHealth.enabled` | Enable deployment health checks | `true` |
| `agent.deploymentHealth.intervalSeconds` | Health check interval | `60` |
| `rbac.create` | Create RBAC resources | `true` |
| `rbac.clusterWide` | Cluster-wide RBAC (vs namespaced) | `true` |
| `rbac.secretAccess.enabled` | Enable secret access | `false` |
| `resources.limits.cpu` | CPU limit | `200m` |
| `resources.limits.memory` | Memory limit | `256Mi` |
| `image.tag` | Image tag to use | `latest` |

Note on `rbac.clusterWide`: Namespace-scoped mode (`rbac.clusterWide: false`) deploys within its namespace, and the chart automatically sets `BROKKR__AGENT__WATCH_NAMESPACE` so telemetry streaming and health discovery operate in-namespace too. Remaining constraints: reconciliation pruning skips resource types it cannot list, and stacks containing cluster-scoped resources (Namespaces, CRDs) fail to apply. See the [agent chart's RBAC documentation](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/RBAC.md).

For complete configuration options, see the chart values files:
- [Broker Chart Values](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-broker/values.yaml)
- [Agent Chart Values](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values.yaml)

## Production Checklist

Three defaults are safe for development but dangerous in production:

1. **Replace the default admin PAK.** The default configuration embeds a publicly known `broker.pak_hash`. Set `broker.pak_hash` (env var `BROKKR__BROKER__PAK_HASH`) to the hash of a PAK you generated, or leave it empty so the broker generates one and writes it to `/tmp/brokkr-keys/key.txt` on first startup.
2. **Set a persistent webhook encryption key.** If `broker.webhook_encryption_key` (`BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`, 64 hex chars / 32 bytes) is unset, the broker generates a random key on every startup — webhook URLs and auth headers encrypted under the previous key become unreadable after a restart.
3. **Lower the log level.** The binary default is `debug`; the Helm chart sets `broker.logLevel: info`. If you run the binary outside the chart, set `BROKKR__LOG__LEVEL=info` (or `warn`).

See the [Configuration Guide](./configuration.md) for details on each setting.

## Next Steps

- Follow the [Deploy Your First Application](../tutorials/first-deployment.md) tutorial to deploy your first application
- Learn about [Basic Concepts](../explanation/core-concepts.md) in Brokkr
- Explore [Configuration Guide](./configuration.md)

## Troubleshooting

### Common Issues

**Broker pod not starting:**
```bash
# Check pod status
kubectl describe pod -l app.kubernetes.io/name=brokkr-broker

# Check logs
kubectl logs -l app.kubernetes.io/name=brokkr-broker
```

**Agent not connecting to broker:**
```bash
# Verify broker URL is accessible from agent
kubectl exec deploy/brokkr-agent -- wget -qO- http://brokkr-broker:3000/healthz

# Check agent logs for connection errors
kubectl logs -l app.kubernetes.io/name=brokkr-agent
```

**Database connection errors:**
```bash
# Check PostgreSQL is running
kubectl get pods -l app.kubernetes.io/name=postgresql

# Check database credentials
kubectl get secret brokkr-broker-postgresql -o yaml
```

**PAK authentication failures:**
- Verify the PAK is correct and not expired
- Check that the agent name matches the registration
- Ensure the broker URL is accessible

### Getting Help

- Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues) for known issues and solutions

## Building from Source

For contributors or anyone building Brokkr from source and running it locally, see the [Local Development Environment](./development.md) guide.
