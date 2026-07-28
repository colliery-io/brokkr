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

This walkthrough passes credentials as plaintext `--set` values, which is the fastest way to see Brokkr working on a throwaway cluster. Values set that way are rendered into ConfigMaps in cleartext, so for anything you intend to keep, use [Production Install: Credentials from Kubernetes Secrets](#production-install-credentials-from-kubernetes-secrets) instead — the steps are the same shape, with each credential read from a Secret you create first.

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

Every `/api/v1` request must carry a PAK (Prefixed API Key) in the `Authorization` header. On first startup the broker creates the admin role.

**For production (recommended)**, generate the admin PAK offline before deployment:

```bash
brokkr-broker generate-pak
```

You do not need a source checkout or a Rust toolchain for this. `brokkr-broker` is the same binary the broker image runs as its entrypoint, so if you are installing by Helm alone, run the command straight from the published image:

```bash
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
```

Substitute the release tag you intend to deploy if you are pinning the image rather than tracking `latest`. The command is entirely offline — no database, no cluster, no keyfile — so it is safe to run on a laptop before the broker exists. Everywhere this guide says `brokkr-broker generate-pak`, either form works.

This prints both the PAK value and its SHA-256 hash without touching a database or keyfile. Set the hash as `broker.pakHash` in your Helm values before first startup; the broker stores it on the admin role at initialization. Keep the PAK value secret and export it for the steps below.

For a throwaway dev cluster you can instead rely on how the broker provisions the admin role at first startup:

- **If you installed with the commands above** (no `broker.pakHash` chart value), the broker's embedded default configuration supplies a publicly known hash, and the admin PAK is `brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8` — fine for a throwaway dev cluster, **never for production**.
- **If you set `broker.pakHash` to the hash of your own PAK** (or reference a Secret containing it via `broker.pakHashExistingSecret`), use that PAK.

> **Warning:** Setting `broker.pakHash` to an empty value does **not** make the broker generate a fresh PAK. The chart only passes the hash to the broker when the value is non-empty, so an empty or omitted `broker.pakHash` silently leaves the publicly known development admin PAK above active. Anywhere beyond a throwaway dev cluster, always set `broker.pakHash` or `broker.pakHashExistingSecret` from the `brokkr-broker generate-pak` output.

The only configuration in which the broker mints an admin PAK for you is one where the hash is genuinely empty in its environment — which the chart cannot produce without an `extraEnv` override. If you do force that, the broker writes the PAK to `/tmp/brokkr-keys/key.txt` inside the broker's own filesystem, and that file is a one-shot artifact: it is written on the genuinely first startup only, never recreated by later restarts, and deleted when the broker shuts down gracefully. A pod restart, reschedule, or `helm upgrade` before you read it loses the admin PAK for good. Capture it in the same breath as the install, or avoid the race entirely by presetting the hash from `brokkr-broker generate-pak` as described above.

If the admin PAK is lost, it cannot be recovered from the stored hash — mint a replacement with `brokkr-broker generate-pak`, update `broker.pakHash` (or the Secret behind `broker.pakHashExistingSecret`), and run `brokkr-broker rotate admin` against the broker's database to store the new hash. Restarting the broker does not re-run the admin bootstrap. See [Managing PAKs](../how-to/pak-management.md#rotating-the-admin-pak).

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

You can optionally pre-register the agent with application generators at creation by adding `"generator_ids": ["<uuid>", ...]` to the request body. If omitted, the agent is registered only with the system/fleet generator and must be registered with an application generator separately before stacks owned by that generator can be targeted at it. See [Registering Agents with Generators](../how-to/agent-registration.md).

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

Note the `"status": "INACTIVE"` in the response: every new agent starts inactive and applies nothing until you activate it. You'll activate this agent in the [Test Deployment](#test-deployment) section before deploying anything through it.

### 5. Install the Agent

Install the agent using the `initial_pak` from step 4. The `broker.agentName` and `broker.clusterName` values must exactly match the name and cluster you registered in step 4 — at startup the agent looks up its own registration by that pair, and a mismatch leaves the pod crashlooping with "Agent not found":

```bash
# Install agent (replace <PAK> with the initial_pak from step 4)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  --set broker.agentName=my-agent \
  --set broker.clusterName=development \
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
kubectl exec deploy/brokkr-broker -- curl -fsS http://localhost:3000/healthz

# Check agent health
kubectl exec deploy/brokkr-agent -- curl -fsS http://localhost:8080/healthz

# View agent registration in broker logs
kubectl logs deploy/brokkr-broker | grep -i agent
```

You should see "OK" from both health checks and agent registration messages in the broker logs.

Both images are `debian:bookworm-slim` with `curl` installed and no `wget`, so `curl` is the tool to reach for in any in-pod check.

## Production Install: Credentials from Kubernetes Secrets

Four values the charts accept are credentials: the database connection URL, the admin PAK hash, the webhook encryption key, and the agent's PAK. Set as plaintext Helm values, each one is rendered into a ConfigMap — readable by anything with `get configmap` in the namespace, and preserved in your values files, in `helm get values`, and in git history if the values file is committed.

Both charts can instead read each credential from a Kubernetes Secret you create beforehand. When you set the matching `existingSecret` value, the chart **leaves the credential out of the ConfigMap** and injects it into the pod with a `secretKeyRef`. That omission is the observable difference, and it is worth checking after the install. This is the recommended path for any install you intend to keep, and the natural fit for GitOps: your values carry only the *name* of the Secret. It also pairs with tools like [external-secrets-operator](https://external-secrets.io/), which vend the credential from Vault, 1Password, or a cloud secret manager into the Secret at deploy time.

Create the Secrets before installing. A Secret or key that does not exist leaves the pod stuck in `CreateContainerConfigError` — Kubernetes cannot start a container whose environment it cannot resolve.

> **Install a recent enough chart.** Most of these `existingSecret` values are new, and Helm silently ignores values a chart does not know about: pin an older chart and the install appears to succeed while the credential goes into the ConfigMap in plaintext anyway. The `helm install` commands on this page carry no `--version`, so Helm resolves the newest published chart and you are fine. If you pin versions, check the minimum in [Chart Versions and the `existingSecret` Values](../how-to/install-operations.md#chart-versions-and-the-existingsecret-values) first, and verify the ConfigMap afterwards as shown below — that check is what catches the mistake.

### 1. Create the Broker Secrets

```bash
# Database connection URL — the full postgres:// URL, not its parts
kubectl create secret generic brokkr-broker-db \
  --from-literal=database-url='postgres://brokkr:<password>@postgres.example.com:5432/brokkr'

# Admin PAK hash, from `brokkr-broker generate-pak` (see Quick Start step 3)
kubectl create secret generic brokkr-broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH='<hash-from-generate-pak>'

# Webhook encryption key — 64 hex characters (32 bytes)
kubectl create secret generic brokkr-broker-webhook-key \
  --from-literal=BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

The key names above are the chart defaults, so nothing else is needed to point at them. If your secret manager writes different key names, set the matching `*ExistingSecretKey` value instead of renaming the key.

### 2. Install the Broker

```bash
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.existingSecret=brokkr-broker-db \
  --set broker.pakHashExistingSecret=brokkr-broker-admin-pak-hash \
  --set broker.webhookEncryptionKeyExistingSecret=brokkr-broker-webhook-key \
  --wait
```

Confirm no credential reached the ConfigMap:

```bash
kubectl get configmap brokkr-broker -o yaml
```

`BROKKR__DATABASE__URL`, `BROKKR__BROKER__PAK_HASH`, and `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` should all be absent from the output. Each is present in the pod's environment instead:

```bash
kubectl exec deploy/brokkr-broker -- printenv BROKKR__BROKER__PAK_HASH
```

Sourcing the webhook encryption key this way also heads off a hard failure later. With no key configured, the broker generates a fresh random key on every start and warns about it; once at least one webhook subscription exists, a broker that starts with the key unset **refuses to start** rather than come up unable to decrypt those subscriptions' stored URLs and auth headers. See [Configuring Webhooks](../how-to/webhooks.md).

### 3. Create the Agent Secret and Install

Register the agent through the broker API first (Quick Start step 4), then put the returned `initial_pak` in a Secret rather than on the `helm install` command line, where it would be visible in your shell history and in the agent's ConfigMap:

```bash
kubectl create secret generic brokkr-agent-credentials \
  --from-literal=BROKKR__AGENT__PAK='<initial_pak>'

helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.existingSecret=brokkr-agent-credentials \
  --set broker.agentName=my-agent \
  --set broker.clusterName=development \
  --wait
```

With `broker.existingSecret` set, the agent ConfigMap's `BROKKR__AGENT__PAK` renders as an empty string and the `secretKeyRef` in the Deployment supplies the real value, so the PAK itself never appears in the ConfigMap or in your values.

### Existing-Secret Values Reference

| Chart | Values | Credential | Default key in the Secret |
|-------|--------|------------|---------------------------|
| Broker | `postgresql.existingSecret` / `postgresql.existingSecretKey` | Full PostgreSQL connection URL (`BROKKR__DATABASE__URL`) | `database-url` |
| Broker | `broker.pakHashExistingSecret` / `broker.pakHashExistingSecretKey` | Admin PAK hash (`BROKKR__BROKER__PAK_HASH`) | `BROKKR__BROKER__PAK_HASH` |
| Broker | `broker.webhookEncryptionKeyExistingSecret` / `broker.webhookEncryptionKeyExistingSecretKey` | Webhook encryption key (`BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`) | `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` |
| Agent | `broker.existingSecret` / `broker.existingSecretKey` | Agent PAK (`BROKKR__AGENT__PAK`) | `BROKKR__AGENT__PAK` |

In every case the `existingSecret` value wins. When it is set, the plaintext value it replaces — the connection URL the chart would otherwise assemble from `postgresql.external.*` or the bundled database's credentials, `broker.pakHash`, `broker.webhookEncryptionKey`, or the agent's `broker.pak` — is ignored, the credential is left out of the ConfigMap, and the pod receives it through `secretKeyRef`. `postgresql.existingSecret` applies whether the database is bundled or external.

You can mix the two: source the admin PAK hash from a Secret while leaving a development database URL as a plain value, for example. Each pair is independent.

## Detailed Installation

### Broker Installation

The broker is the central management service that coordinates deployments across your Kubernetes clusters.

#### Development Setup (Bundled PostgreSQL)

For development and testing, use the bundled PostgreSQL. The password below is rendered into the broker's ConfigMap as part of `BROKKR__DATABASE__URL`, so keep this form to dev and test clusters and use [`postgresql.existingSecret`](#production-install-credentials-from-kubernetes-secrets) elsewhere:

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

Passing the PAK with `--set broker.pak` renders it into the agent's ConfigMap in plaintext — acceptable for dev and test, but for anything longer-lived source it from a Secret with `broker.existingSecret` as shown in [Production Install](#3-create-the-agent-secret-and-install).

```bash
# Create agent via broker API (see Quick Start step 4)
# Then install with the returned initial_pak.
# agentName and clusterName must match the created agent exactly,
# or the agent's startup self-lookup fails and the pod crashloops.

helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK_FROM_BROKER>" \
  --set broker.agentName=<AGENT_NAME> \
  --set broker.clusterName=<CLUSTER_NAME> \
  --wait
```

#### Using Provided Values Files

Brokkr includes pre-configured values files for agents — development (minimal resources, cluster-wide RBAC), staging (moderate resources), and production (production-grade resources). Install with the one for your environment:

```bash
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  --set broker.agentName=<AGENT_NAME> \
  --set broker.clusterName=<CLUSTER_NAME> \
  -f https://raw.githubusercontent.com/colliery-io/brokkr/main/charts/brokkr-agent/values/<environment>.yaml
```

View all available agent values files:
- [Development](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/development.yaml)
- [Staging](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/staging.yaml)
- [Production](https://github.com/colliery-io/brokkr/blob/main/charts/brokkr-agent/values/production.yaml)

## Chart Versions, Upgrades, and Uninstallation

Every `helm install` on this page is unpinned, so Helm pulls the newest published chart — the right default, and the one that supports everything documented here. For pinning to a specific version (and the minimum version the `existingSecret` values need), installing development builds, upgrading, and uninstalling, see [Installing, Upgrading, and Uninstalling with Helm](../how-to/install-operations.md).

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

# New agents start INACTIVE and skip all deployment work until activated.
# Activate yours so it will pull and reconcile deployment objects:
curl -X PUT http://localhost:3000/api/v1/agents/$AGENT_ID \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"status": "ACTIVE"}'

# Register the agent with the admin-generator first. Targeting a stack at an
# agent that is not registered with the stack's owning generator returns
# 403 agent_not_registered. (Skip this if you passed generator_ids at
# agent creation.) See ../how-to/agent-registration.md.
curl -X POST http://localhost:3000/api/v1/generators/$GEN_ID/register \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d "{\"agent_id\": \"$AGENT_ID\"}"

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

# Allow one poll cycle (the chart's default agent.pollingInterval is 30s),
# then verify the namespace was created
kubectl get namespace brokkr-test

# Clean up
curl -X POST http://localhost:3000/api/v1/stacks/$STACK_ID/deployment-objects \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "yaml_content": "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: brokkr-test",
    "is_deletion_marker": true
  }'

kubectl get namespace brokkr-test  # After the next poll cycle: Terminating/NotFound
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
| `broker.webhookEncryptionKey` | Hex-encoded 32-byte key for webhook secrets at rest; a random key is generated on every boot if unset. Rendered into the ConfigMap in plaintext — prefer `broker.webhookEncryptionKeyExistingSecret` | unset |
| `configReload.enabled` | Watch the ConfigMap and reload hot-reloadable settings automatically | `true` |
| `configReload.debounceSeconds` | Debounce window for successive reloads | `5` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `ingress.tls` | TLS termination for the ingress — the broker itself serves plain HTTP and cannot terminate TLS | unset |

The credential values that can be sourced from a pre-created Secret instead — the database URL, the admin PAK hash, and the webhook encryption key — are listed in the [Existing-Secret Values Reference](#existing-secret-values-reference).

### Agent Values

Key configuration options for the agent chart:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `broker.url` | Broker URL | `http://brokkr-broker:3000` |
| `broker.pak` | Agent PAK (Prefixed API Key). Rendered into the ConfigMap in plaintext (dev/test); ignored when `broker.existingSecret` is set | **Required** unless `broker.existingSecret` is set |
| `broker.existingSecret` | Name of a pre-created Secret to read the agent PAK from; injected via `secretKeyRef` and kept out of the ConfigMap. See the [Existing-Secret Values Reference](#existing-secret-values-reference) | `""` |
| `broker.existingSecretKey` | Key within that Secret holding the PAK | `BROKKR__AGENT__PAK` |
| `broker.generatorIds` | Generator UUIDs (YAML list or comma string) this agent self-registers with at startup. The agent is always auto-registered with the system/fleet generator; application generators must be listed here to serve their stacks. See [Registering Agents with Generators](../how-to/agent-registration.md) | `[]` |
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

These defaults and shortcuts are safe for development but dangerous in production:

1. **Replace the default admin PAK.** The default configuration embeds a publicly known `broker.pak_hash` — leaving it in place means anyone can use the development admin credential against your broker. Run `brokkr-broker generate-pak` — from the broker image if you have no binary, `docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak` (see [Get the Admin PAK](#3-get-the-admin-pak)) — and set the printed hash via `broker.pakHashExistingSecret` (or, on a dev cluster, as the plaintext `broker.pakHash`) before first startup. Leaving the chart value empty does **not** generate a fresh key; it silently keeps the publicly known default active.
2. **Set a persistent webhook encryption key.** If `broker.webhook_encryption_key` (`BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`, 64 hex chars / 32 bytes) is unset, the broker generates a random key on every startup — webhook URLs and auth headers encrypted under the previous key become unreadable after a restart, and once subscriptions exist the broker refuses to start with the key unset. Source it from a Secret with `broker.webhookEncryptionKeyExistingSecret`.
3. **Keep credentials out of ConfigMaps.** The database URL, admin PAK hash, webhook encryption key, and agent PAK all land in a ConfigMap in cleartext when set as plaintext Helm values. Source each from a pre-created Secret instead — see [Production Install: Credentials from Kubernetes Secrets](#production-install-credentials-from-kubernetes-secrets).
4. **Lower the log level.** The binary default is `debug`; the Helm chart sets `broker.logLevel: info`. If you run the binary outside the chart, set `BROKKR__LOG__LEVEL=info` (or `warn`).

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
kubectl exec deploy/brokkr-agent -- curl -fsS http://brokkr-broker:3000/healthz

# Check agent logs for connection errors
kubectl logs -l app.kubernetes.io/name=brokkr-agent
```

**Pod stuck in `CreateContainerConfigError`:**

A pod that never starts a container is usually missing a Secret it was told to read. Kubernetes cannot resolve the environment, so it never gets as far as running the image.

```bash
# The event names the missing Secret or key
kubectl describe pod -l app.kubernetes.io/name=brokkr-broker | tail -20

# Confirm the Secret exists and holds the expected key
kubectl get secret <secret-name> -o jsonpath='{.data}' | jq 'keys'
```

Check the key name against the defaults in the [Existing-Secret Values Reference](#existing-secret-values-reference), and set the matching `*ExistingSecretKey` value if your Secret uses a different one.

**Database connection errors:**
```bash
# Check PostgreSQL is running
kubectl get pods -l app.kubernetes.io/name=postgresql

# Check database credentials
kubectl get secret brokkr-broker-postgresql -o yaml
```

**PAK authentication failures:**
- Verify the PAK is correct. PAKs have no expiry — a key that used to work has been rotated or revoked (its agent or generator was deleted)
- Check that the agent name matches the registration
- Ensure the broker URL is accessible

### Getting Help

- Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues) for known issues and solutions

## Building from Source

For contributors or anyone building Brokkr from source and running it locally, see the [Local Development Environment](./development.md) guide.
