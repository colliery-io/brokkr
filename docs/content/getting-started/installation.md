---
title: "Installation Guide"
weight: 1
---

# Installing Brokkr

This guide will help you install Brokkr using Helm, the recommended installation method.

## Prerequisites

Before installing Brokkr, ensure you have:

- **Kubernetes cluster** (v1.20 or later)
- **kubectl** CLI configured to access your cluster
- **Helm** 3.8 or later installed ([installation guide](https://helm.sh/docs/intro/install/))

### Verifying Prerequisites

```bash
# Check Kubernetes version
kubectl version --short

# Check Helm version
helm version --short

# Verify cluster access
kubectl cluster-info
```

## Quick Start

Get Brokkr up and running in under 10 minutes with a broker and agent in your Kubernetes cluster.

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

### 3. Create an Agent and Get PAK

Create an agent registration and retrieve its Pre-Authentication Key (PAK):

```bash
# Create a new agent
curl -X POST http://localhost:3000/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-agent",
    "cluster_name": "development"
  }'
```

The response will include the agent's PAK:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-agent",
  "cluster_name": "development",
  "status": "ACTIVE",
  "pak": "brokkr_BRxxxxxxxx_yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
}
```

Save the `pak` value - you'll need it to install the agent.

### 4. Install the Agent

Install the agent using the PAK from step 3:

```bash
# Install agent (replace <PAK> with the actual PAK from step 3)
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

### 5. Verify Installation

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

Brokkr includes pre-configured values files for different environments:

```bash
# Development (bundled PostgreSQL, minimal resources)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-broker/values/development.yaml

# Staging (external PostgreSQL, moderate resources)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-broker/values/staging.yaml

# Production (external PostgreSQL, production-grade resources)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-broker/values/production.yaml
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
# Create agent via broker API (see Quick Start step 3)
# Then install with the returned PAK:

helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK_FROM_BROKER>" \
  --wait
```

#### Using Provided Values Files

Brokkr includes pre-configured values files for agents:

```bash
# Development (minimal resources, cluster-wide RBAC)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-agent/values/development.yaml

# Staging (moderate resources)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-agent/values/staging.yaml

# Production (production-grade resources)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-agent/values/production.yaml
```

View all available agent values files:
- [Development](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/development.yaml)
- [Staging](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/staging.yaml)
- [Production](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/production.yaml)

## Chart Versions

Brokkr Helm charts are published to GitHub Container Registry (GHCR).

### Installing Specific Versions

```bash
# Install a specific release version
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 1.0.0 \
  --set postgresql.enabled=true

# List available versions
# Visit: https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker
```

### Development Builds

Development builds are available for testing:

```bash
# Development builds use semver pre-release format with timestamps
# Example: 0.0.0-develop.20251021150606

# Find the latest development build at:
# https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker

# Install development build (replace timestamp with actual version)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.0.0-develop.20251021150606 \
  --set postgresql.enabled=true
```

## Upgrading Brokkr

Upgrade your Brokkr installation to a newer version:

```bash
# Upgrade broker
helm upgrade brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 1.1.0 \
  --reuse-values

# Upgrade agent
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --version 1.1.0 \
  --reuse-values
```

## Uninstalling Brokkr

Remove Brokkr from your cluster:

```bash
# Uninstall agent
helm uninstall brokkr-agent

# Uninstall broker (this will also remove bundled PostgreSQL if enabled)
helm uninstall brokkr-broker

# Note: PersistentVolumes may remain - delete manually if needed
kubectl get pv
kubectl delete pv <pv-name>
```

## Verifying the Installation

### Health Checks

```bash
# Broker health endpoint
kubectl exec deploy/brokkr-broker -- wget -qO- http://localhost:3000/healthz

# Agent health endpoint
kubectl exec deploy/brokkr-agent -- wget -qO- http://localhost:8080/healthz
```

Both should return "OK".

### Connectivity Tests

```bash
# Check agent registration in broker
kubectl logs deploy/brokkr-broker | grep "agent registered"

# Check agent connection to broker
kubectl logs deploy/brokkr-agent | grep "connected to broker"

# List registered agents via API
kubectl port-forward svc/brokkr-broker 3000:3000 &
curl http://localhost:3000/api/v1/agents
```

### Test Deployment

Create a test namespace to verify end-to-end functionality:

```bash
# Port forward to broker
kubectl port-forward svc/brokkr-broker 3000:3000 &

# Create a stack
STACK_ID=$(curl -s -X POST http://localhost:3000/api/v1/stacks \
  -H "Content-Type: application/json" \
  -d '{"name": "test-stack", "description": "Test stack"}' \
  | jq -r '.id')

# Deploy a test namespace
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-test",
    "is_deletion_marker": false
  }'

# Verify namespace was created
kubectl get namespace brokkr-test

# Clean up
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects \
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
| `postgresql.enabled` | Enable bundled PostgreSQL | `false` |
| `postgresql.auth.password` | PostgreSQL password | `brokkr` |
| `database.host` | External database host | `""` |
| `database.port` | External database port | `5432` |
| `database.name` | Database name | `brokkr` |
| `database.user` | Database username | `brokkr` |
| `database.password` | Database password | `""` |
| `replicaCount` | Number of broker replicas | `1` |
| `image.tag` | Image tag to use | Chart appVersion |
| `resources.limits.cpu` | CPU limit | `1000m` |
| `resources.limits.memory` | Memory limit | `1Gi` |
| `tls.enabled` | Enable TLS | `false` |

### Agent Values

Key configuration options for the agent chart:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `broker.url` | Broker URL | **Required** |
| `broker.pak` | Agent PAK | **Required** |
| `agent.pollingInterval` | Polling interval | `30s` |
| `rbac.mode` | RBAC mode (cluster/namespace) | `cluster` |
| `rbac.create` | Create RBAC resources | `true` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `image.tag` | Image tag to use | Chart appVersion |

For complete configuration options, see the chart values files:
- [Broker Chart Values](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-broker/values.yaml)
- [Agent Chart Values](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values.yaml)

## Next Steps

- Follow our [Quick Start Guide](quick-start) to deploy your first application
- Learn about [Basic Concepts](../explanation/core-concepts) in Brokkr
- Explore [Configuration Examples](../../how-to/configuration)

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

- Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues)
- Read the [Troubleshooting Guide](../../how-to/troubleshoot)

---

## Building from Source

For contributors or advanced users who want to build Brokkr from source:

### Prerequisites
- Rust toolchain (1.8+)
- PostgreSQL database (v12+)
- Kubernetes cluster
- Docker (for building images)

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/colliery-io/brokkr.git
cd brokkr

# Build using Cargo
cargo build --release

# The binaries will be available in target/release/
# - brokkr-broker: The central management service
# - brokkr-agent: The Kubernetes cluster agent
```

### Running Locally

```bash
# Set up database
export BROKKR__DATABASE__URL="postgres://brokkr:brokkr@localhost:5432/brokkr"

# Run broker
./target/release/brokkr-broker serve

# Run agent (in another terminal)
export BROKKR__AGENT__PAK="<your-pak>"
export BROKKR__AGENT__BROKER_URL="http://localhost:3000"
./target/release/brokkr-agent start
```

### Development Environment

For active development:

```bash
# Install Angreal (development task runner)
pip install angreal

# Start the development environment
angreal local up

# Rebuild specific services
angreal local rebuild broker
angreal local rebuild agent
```

For more details on contributing, see our [Development Guide](../../contributing/development).
