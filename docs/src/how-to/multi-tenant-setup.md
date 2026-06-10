# How-To: Setting Up Multi-Tenant Isolation

This guide walks through configuring Brokkr for multi-tenant operation using PostgreSQL schema isolation. Each tenant gets a fully isolated dataset while sharing a single database server.

## Goal

Set up two tenants (`acme` and `globex`) on a shared PostgreSQL instance, each with their own broker instance and complete data isolation.

## Prerequisites

- A PostgreSQL server accessible to both broker instances
- The `brokkr` user with permission to create schemas
- Helm (for Kubernetes deployment) or direct access to run broker binaries

## Step 1: Prepare the Database

Create the shared database if it doesn't exist:

```sql
CREATE DATABASE brokkr;
CREATE USER brokkr WITH PASSWORD 'your-secure-password';
GRANT ALL PRIVILEGES ON DATABASE brokkr TO brokkr;

-- Grant schema creation permission
GRANT CREATE ON DATABASE brokkr TO brokkr;
```

You don't need to create the schemas manually — Brokkr creates them on first startup.

## Step 2: Deploy Tenant Broker Instances

### Option A: Helm (Kubernetes)

Deploy each tenant as a separate Helm release:

```bash
# Tenant: Acme
helm install brokkr-acme oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-acme --create-namespace \
  --set postgresql.enabled=false \
  --set postgresql.external.host=postgres.example.com \
  --set postgresql.external.port=5432 \
  --set postgresql.external.database=brokkr \
  --set postgresql.external.username=brokkr \
  --set postgresql.external.password=your-secure-password \
  --set postgresql.external.schema=tenant_acme

# Tenant: Globex
helm install brokkr-globex oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-globex --create-namespace \
  --set postgresql.enabled=false \
  --set postgresql.external.host=postgres.example.com \
  --set postgresql.external.port=5432 \
  --set postgresql.external.database=brokkr \
  --set postgresql.external.username=brokkr \
  --set postgresql.external.password=your-secure-password \
  --set postgresql.external.schema=tenant_globex
```

### Option B: Environment Variables (Direct)

Run each broker with different schema settings. The broker's bind address is fixed at `0.0.0.0:3000` — there is no configuration option to change the port — so each tenant's broker must run in its own container or on its own host (or behind its own port mapping):

```bash
# Host/container 1: Acme broker
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=tenant_acme \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve

# Host/container 2: Globex broker
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=tenant_globex \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve
```

Per-tenant settings can be provided as `BROKKR__*` environment variables as shown above, or via a per-tenant TOML file referenced by `BROKKR_CONFIG_FILE` (environment variables still override file values).

## Step 3: First Startup

On first startup, each broker instance:

1. Creates the schema (`CREATE SCHEMA IF NOT EXISTS tenant_acme`)
2. Runs all database migrations within the schema
3. Creates the admin role and an admin PAK

By default, `broker.pak_hash` is set to a publicly-known development hash, which would give **both tenants the same well-known dev PAK**. For any real multi-tenant setup, override it per tenant: either supply your own per-tenant hash, or set it empty to force the broker to generate a fresh PAK:

```bash
# Force per-tenant PAK generation
BROKKR__BROKER__PAK_HASH="" \
BROKKR__DATABASE__SCHEMA=tenant_acme \
... brokkr-broker serve
```

When the broker generates a PAK, it is written to `/tmp/brokkr-keys/key.txt` inside the broker's filesystem — it is not logged. **Capture it for each tenant**:

```bash
# Kubernetes
kubectl exec -n brokkr-acme <acme-broker-pod> -- cat /tmp/brokkr-keys/key.txt

# Direct/container
cat /tmp/brokkr-keys/key.txt
```

## Step 4: Register Agents Per Tenant

Each tenant's agents connect to their tenant's broker instance:

```bash
# Create agent for Acme tenant
curl -s -X POST http://acme-broker:3000/api/v1/agents \
  -H "Authorization: <acme-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "acme-prod", "cluster_name": "us-east-1"}'

# Create agent for Globex tenant
curl -s -X POST http://globex-broker:3000/api/v1/agents \
  -H "Authorization: <globex-admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "globex-prod", "cluster_name": "eu-west-1"}'
```

## Step 5: Deploy Tenant Agents

Point each agent at the correct tenant broker:

```bash
# Acme agent
helm install brokkr-agent-acme oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --namespace brokkr-acme \
  --set broker.url=http://brokkr-acme-brokkr-broker:3000 \
  --set broker.pak="<acme-agent-pak>" \
  --set broker.agentName=acme-prod \
  --set broker.clusterName=us-east-1

# Globex agent
helm install brokkr-agent-globex oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --namespace brokkr-globex \
  --set broker.url=http://brokkr-globex-brokkr-broker:3000 \
  --set broker.pak="<globex-agent-pak>" \
  --set broker.agentName=globex-prod \
  --set broker.clusterName=eu-west-1
```

## Step 6: Verify Isolation

Confirm that each tenant only sees its own data:

```bash
# Acme sees only Acme agents
curl -s http://acme-broker:3000/api/v1/agents \
  -H "Authorization: <acme-admin-pak>" | jq '.[].name'
# Output: "acme-prod"

# Globex sees only Globex agents
curl -s http://globex-broker:3000/api/v1/agents \
  -H "Authorization: <globex-admin-pak>" | jq '.[].name'
# Output: "globex-prod"
```

Acme's admin PAK does **not** work against Globex's broker, and vice versa.

## Connection Pool Sizing

Each broker instance uses a connection pool (default: 50 connections). With multiple tenants on one database, the total connections across all broker instances must stay under PostgreSQL's `max_connections` (default: 100). Increase it or reduce per-tenant pool sizes for many tenants. See [Multi-Tenancy Reference](../reference/multi-tenancy.md) for detailed capacity planning.

## Schema Naming

Use a consistent pattern like `tenant_{name}` (e.g., `tenant_acme`). Schema names allow only alphanumeric characters and underscores, max 63 characters. See [Multi-Tenancy Reference](../reference/multi-tenancy.md#schema-name-constraints) for full constraints.

## Related Documentation

- [Multi-Tenancy Reference](../reference/multi-tenancy.md) — data isolation details and limitations
- [Configuration Guide](../getting-started/configuration.md) — database configuration
- [Installation Guide](../getting-started/installation.md) — deployment options
