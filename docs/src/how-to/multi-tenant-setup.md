# How-To: Setting Up Multi-Tenant Isolation

This guide walks through configuring Brokkr for multi-tenant operation using PostgreSQL schema isolation. Each tenant gets a fully isolated dataset while sharing a single database server.

This approach uses deployment-level isolation: one broker per tenant, all sharing a single database server. For application-level isolation — a single broker serving multiple applications, where each agent opts in to the generators it serves — see generator registration in the [Security Model](../explanation/security-model.md#generator-registration-and-application-scopes). The two mechanisms are complementary, not interchangeable.

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

First mint an admin PAK and hash for each tenant. The chart does not do this for you, and a release that sets neither `broker.pakHash` nor `broker.pakHashExistingSecret` falls back to the publicly-known development admin PAK compiled into the broker binary — anyone would be able to authenticate as that tenant's admin:

```bash
# One run per tenant — save the printed PAK (the credential) and hash separately
brokkr-broker generate-pak   # -> Acme admin PAK + hash
brokkr-broker generate-pak   # -> Globex admin PAK + hash
```

(No local binary? The container image's entrypoint is the broker: `docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak`.)

Then deploy each tenant as a separate Helm release, passing that tenant's hash:

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
  --set postgresql.external.schema=tenant_acme \
  --set broker.pakHash=<acme-admin-pak-hash>

# Tenant: Globex
helm install brokkr-globex oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-globex --create-namespace \
  --set postgresql.enabled=false \
  --set postgresql.external.host=postgres.example.com \
  --set postgresql.external.port=5432 \
  --set postgresql.external.database=brokkr \
  --set postgresql.external.username=brokkr \
  --set postgresql.external.password=your-secure-password \
  --set postgresql.external.schema=tenant_globex \
  --set broker.pakHash=<globex-admin-pak-hash>
```

`broker.pakHash` renders the hash into the broker ConfigMap in plaintext — fine for dev/test. For production, put each tenant's hash in a pre-created Secret and reference it instead:

```bash
kubectl create secret generic acme-admin-pak-hash \
  --namespace brokkr-acme \
  --from-literal=BROKKR__BROKER__PAK_HASH=<acme-admin-pak-hash>

helm install brokkr-acme oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-acme --create-namespace \
  ... \
  --set broker.pakHashExistingSecret=acme-admin-pak-hash
```

> **Warning:** setting `broker.pakHash` to an empty string is the same as omitting it — the chart only renders `BROKKR__BROKER__PAK_HASH` when the value is non-empty, so an empty value silently leaves the publicly-known development admin PAK active. It does **not** make the broker generate a fresh PAK.

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
3. Creates the admin role, storing the configured admin PAK hash (or generating a fresh PAK only if no hash is configured at all)

By default, `broker.pak_hash` is set to a publicly-known development hash, which would give **both tenants the same well-known dev PAK**. For any real multi-tenant setup, override it per tenant. There are two approaches:

**Recommended (production):** Mint a PAK and its hash offline with `brokkr-broker generate-pak`, then set `BROKKR__BROKER__PAK_HASH` to the generated hash before first startup (for Helm, that is the `broker.pakHash` / `broker.pakHashExistingSecret` flow from Step 2). This is the day-zero bootstrap flow — it touches no database and writes no keyfile, so you control the PAK from the start:

```bash
# Generate an admin PAK + SHA-256 hash offline (one run per tenant)
brokkr-broker generate-pak
# Then set the hash for that tenant's broker
BROKKR__BROKER__PAK_HASH="<generated-hash>" \
BROKKR__DATABASE__SCHEMA=tenant_acme \
... brokkr-broker serve
```

**Alternative (direct/env-var runs only):** Explicitly set `BROKKR__BROKER__PAK_HASH` to an empty string to force the broker to generate a fresh PAK on first startup:

```bash
# Force per-tenant PAK generation
BROKKR__BROKER__PAK_HASH="" \
BROKKR__DATABASE__SCHEMA=tenant_acme \
... brokkr-broker serve
```

This only works when the variable is genuinely set to empty in the broker's environment. Via Helm, `--set broker.pakHash=""` does **not** do this — the chart drops the variable and the public default hash applies (see the warning in Step 2). To force generation under the chart you would have to inject `BROKKR__BROKER__PAK_HASH: ""` through `extraEnv`; prefer the recommended flow instead.

When the broker generates a PAK this way, it writes the raw PAK to `/tmp/brokkr-keys/key.txt` inside the broker's filesystem — it is not logged. The file is written on the true first startup only (later restarts never recreate it) and is deleted on graceful shutdown, so **capture it promptly for each tenant**:

```bash
# Kubernetes (only when generation was forced via extraEnv)
kubectl exec -n brokkr-acme <acme-broker-pod> -- cat /tmp/brokkr-keys/key.txt

# Direct/container
cat /tmp/brokkr-keys/key.txt
```

**Recovering a lost admin PAK:** when a hash is configured (the recommended flow), no key file is ever written — there is nothing to `cat` out of the pod. If you lose a tenant's admin PAK, mint a replacement pair with `brokkr-broker generate-pak`, update that tenant's configured hash (`BROKKR__BROKER__PAK_HASH`, or the chart's `broker.pakHash` / Secret), and run `brokkr-broker rotate admin` with that tenant's configuration to store the new hash. A plain restart does not re-run the admin bootstrap.

## Step 4: Create Agents Per Tenant

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

Schema-per-tenant isolation does not require generators, so no generator configuration is shown above. If you also use application-specific generators within a tenant, register the agent with them via `broker.generatorIds` (a YAML list or comma-separated string, rendered to `BROKKR__AGENT__GENERATOR_IDS`):

```bash
  --set broker.generatorIds="{<generator-uuid>,<generator-uuid>}"
```

See [Agent Registration](agent-registration.md) for the operational steps.

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

Use a consistent pattern like `tenant_{name}` (e.g., `tenant_acme`). Brokkr requires schema names to start with a letter and contain only letters, numbers, and underscores. (PostgreSQL caps identifiers at 63 characters; Brokkr itself does not validate length.) See [Multi-Tenancy Reference](../reference/multi-tenancy.md#schema-name-constraints) for full constraints.

## Related Documentation

- [Multi-Tenancy Reference](../reference/multi-tenancy.md) — data isolation details and limitations
- [Configuration Guide](../getting-started/configuration.md) — database configuration
- [Installation Guide](../getting-started/installation.md) — deployment options
