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

Run each broker with different schema settings:

```bash
# Terminal 1: Acme broker (port 3000)
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=tenant_acme \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve

# Terminal 2: Globex broker (port 3001 - change bind port in config)
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=tenant_globex \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve
```

### Option C: Configuration Files

Create a config file per tenant:

```toml
# /etc/brokkr/acme.toml
[database]
url = "postgres://brokkr:password@postgres.example.com:5432/brokkr"
schema = "tenant_acme"

[log]
level = "info"
format = "json"
```

```toml
# /etc/brokkr/globex.toml
[database]
url = "postgres://brokkr:password@postgres.example.com:5432/brokkr"
schema = "tenant_globex"

[log]
level = "info"
format = "json"
```

```bash
BROKKR_CONFIG_FILE=/etc/brokkr/acme.toml brokkr-broker serve
BROKKR_CONFIG_FILE=/etc/brokkr/globex.toml brokkr-broker serve
```

## Step 3: First Startup

On first startup, each broker instance:

1. Creates the schema (`CREATE SCHEMA IF NOT EXISTS tenant_acme`)
2. Runs all database migrations within the schema
3. Creates the admin role and generates an admin PAK
4. Logs the admin PAK to stdout

**Capture the admin PAK for each tenant** — it's only shown once:

```bash
# Look for this line in the logs
# INFO  Admin PAK: brokkr_BR...
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
