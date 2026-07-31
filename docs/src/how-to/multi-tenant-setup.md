# How-To: Set Up Multi-Tenant Operation

This guide walks through onboarding multiple teams onto **one** Brokkr broker. Each team becomes a generator — Brokkr's tenant — with its own PAK, its own stacks, and its own set of agents that have explicitly consented to serve it.

If instead you need fully separate broker instances (for example one per environment, or when data must be separated at the storage layer), skip to [Running Separate Broker Instances](#running-separate-broker-instances-schema-separation) at the end of this guide. That is a deployment feature, not the tenant model, and the two are independent — a separate instance still has generators inside it.

## Goal

Onboard two teams, `acme` and `globex`, onto a single running broker. Each team gets a generator PAK, deploys its own agent, and creates stacks that only its own agents receive.

## Prerequisites

- A running broker with its admin PAK available (`$ADMIN_PAK` below)
- `curl` and `jq`
- Helm and cluster access for each team's agent

## Step 1: Create a Generator Per Team

Create one generator per tenant with the admin PAK:

```bash
curl -s -X POST http://broker:3000/api/v1/generators \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"name": "team-acme", "description": "Acme platform team"}'
```

The response carries the tenant's ID and its PAK:

```json
{
  "generator": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "team-acme",
    "description": "Acme platform team",
    "is_active": true
  },
  "pak": "brokkr_BRgen12ab_GeneratorLongTokenExample01"
}
```

The `pak` is returned **once**. Hand it to the team over a secure channel and have them store it in their secret manager; the only recovery path is `POST /api/v1/generators/{id}/rotate-pak`. Keep the generator `id` too — the team needs it as `generator_id` on every stack they create, and you need it for scoped views later.

Repeat for `team-globex`. Note both IDs:

```bash
ACME_GEN_ID=a1b2c3d4-e5f6-7890-abcd-ef1234567890
GLOBEX_GEN_ID=b2c3d4e5-f6a7-8901-bcde-f12345678901
```

## Step 2: Create Each Team's Agents, Registered to Their Generator

Agent creation is admin-only. Pass `generator_ids` so the agent is registered with its team's generator at creation:

```bash
curl -s -X POST http://broker:3000/api/v1/agents \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme-prod",
    "cluster_name": "us-east-1",
    "generator_ids": ["'"$ACME_GEN_ID"'"]
  }'
```

The response contains the agent record and its one-time `initial_pak`:

```json
{
  "agent": { "id": "11112222-3333-4444-5555-666677778888", "name": "acme-prod" },
  "initial_pak": "brokkr_BRagt56ef_AgentLongTokenExample01"
}
```

Every agent is also auto-registered with the system generator, so fleet-wide stacks still reach it. A `generator_ids` entry that does not exist is rejected with `400 invalid_generator_id`.

Repeat for `globex-prod` with `$GLOBEX_GEN_ID`.

## Step 3: Deploy Each Team's Agent

Deploy the agent with its own PAK, and declare its generator registrations with `broker.generatorIds` so the agent re-asserts them at startup:

```bash
helm install brokkr-agent-acme oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --namespace acme --create-namespace \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<acme-agent-pak>" \
  --set broker.agentName=acme-prod \
  --set broker.clusterName=us-east-1 \
  --set broker.generatorIds="{$ACME_GEN_ID}"
```

`broker.generatorIds` accepts a YAML list or a comma-separated string and renders to `BROKKR__AGENT__GENERATOR_IDS`. Registration is the agent's own opt-in: it is what makes the cluster operator, not the broker admin, the party that decides which tenants' work the cluster will run. See [Agent Registration](agent-registration.md) for the full registration workflow.

Repeat for `globex`, pointing at the same broker URL with `$GLOBEX_GEN_ID`.

## Step 4: Let Each Team Create Stacks Under Its Generator

From here the team works with its own generator PAK and never needs the admin PAK. Stack creation must set `generator_id` to the team's own generator — the broker rejects any other value:

```bash
curl -s -X POST http://broker:3000/api/v1/stacks \
  -H "Authorization: Bearer $ACME_GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme-web",
    "description": "Acme web tier",
    "generator_id": "'"$ACME_GEN_ID"'"
  }'
```

A team member who needs their tenant ID can read it back from their PAK:

```bash
curl -s -X POST http://broker:3000/api/v1/auth/pak \
  -H "Authorization: Bearer $ACME_GENERATOR_PAK" | jq -r '.generator'
```

The team then labels the stack to select which of its agents receive it:

```bash
curl -s -X POST "http://broker:3000/api/v1/stacks/$STACK_ID/labels" \
  -H "Authorization: Bearer $ACME_GENERATOR_PAK" \
  -H "Content-Type: application/json" \
  -d '"env=prod"'
```

(The label endpoint takes a bare JSON string, not an object.)

Labels select **within** the generators an agent has registered with. `env=prod` on Acme's stack will never reach Globex's agent, even though Globex also labels its agents `env=prod` — the agent is not registered with Acme's generator, so the match is not considered. Teams do not have to coordinate on label vocabulary.

Templates behave the same way: a generator can instantiate its own templates and admin-owned system templates, but another tenant's template returns `403 template_not_accessible`.

## Step 5: Verify Tenant Separation

Each generator PAK sees only its own stacks:

```bash
curl -s http://broker:3000/api/v1/stacks \
  -H "Authorization: Bearer $ACME_GENERATOR_PAK" | jq '.[].name'
# Output: "acme-web"

curl -s http://broker:3000/api/v1/stacks \
  -H "Authorization: Bearer $GLOBEX_GENERATOR_PAK" | jq '.[].name'
# Output: "globex-api"
```

Confirm each agent's registrations are what you expect:

```bash
curl -s "http://broker:3000/api/v1/agents/$ACME_AGENT_ID/registrations" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq '.[].generator_id'
```

And confirm an agent's served set contains only its own tenant's work:

```bash
curl -s "http://broker:3000/api/v1/agents/$ACME_AGENT_ID/target-state" \
  -H "Authorization: Bearer $ADMIN_PAK" | jq 'length'
```

To prove the boundary holds against admin error, try to target Acme's stack at Globex's agent:

```bash
curl -s -o /dev/null -w '%{http_code}\n' \
  -X POST "http://broker:3000/api/v1/agents/$GLOBEX_AGENT_ID/targets" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "'"$GLOBEX_AGENT_ID"'", "stack_id": "'"$ACME_STACK_ID"'"}'
# 403 — agent_not_registered
```

There is no force flag. To target across the boundary you must first register the agent with that generator, which is one explicit, audited call.

## Step 6: Scope Operator Views to One Tenant

The broker serves an operator console with a tenant scope selector. It is backed by two pieces of API you can also drive yourself.

List the tenants (admin credential required; the console's read-only PAK qualifies):

```bash
curl -s http://broker:3000/api/v1/paks \
  -H "Authorization: Bearer $ADMIN_PAK"
```

```json
[
  { "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890", "name": "team-acme" },
  { "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901", "name": "team-globex" }
]
```

The path is `/api/v1/paks`, not `/api/v1/auth/paks`. The system generator is never listed.

Then narrow the admin listings to one tenant with `?pak_id=<generator id>`:

```bash
# Fleet records for agents registered with Acme
curl -s "http://broker:3000/api/v1/fleet?pak_id=$ACME_GEN_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"

# Stacks owned by Acme
curl -s "http://broker:3000/api/v1/stacks?pak_id=$ACME_GEN_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"

# Agent events from agents registered with Acme
curl -s "http://broker:3000/api/v1/agent-events?pak_id=$ACME_GEN_ID" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

An unknown but well-formed UUID returns an empty list; a malformed one returns `400`.

> **Do not treat `pak_id` as an access control.** It is a display filter for an admin who could see everything anyway — dropping the parameter shows all tenants again. Never hand a team an admin PAK plus a `pak_id` and call it tenant access. A team's access boundary is its own generator PAK; an agent's boundary is its registrations.

## What This Setup Does and Does Not Separate

| | Separated per tenant |
|---|---|
| Stacks, deployment objects, templates | Yes — by generator ownership |
| Which agents receive a tenant's stacks | Yes — by agent registration, on targets, labels, and annotations |
| Credentials | Yes — each team holds only its own generator PAK |
| Admin PAK | **No** — one admin governs all tenants on the broker |
| Database and broker process | **No** — one broker, one schema, shared resources |
| Fleet and audit views | **No** — admin-only surfaces span all tenants |

If a tenant needs its own admin, its own database tables, or resource separation from other tenants, run it as a separate broker instance instead.

## Running Separate Broker Instances (Schema Separation)

Setting a PostgreSQL schema per broker lets several fully independent broker instances share one PostgreSQL server. Use it for environment separation, or when data must be separated at the storage layer. Each instance has its own admin PAK, its own agents, and its own generators — the tenant model above still applies *inside* each one.

The example below uses two instances, `staging` and `prod`.

### Step A: Prepare the Database

Create the shared database if it doesn't exist:

```sql
CREATE DATABASE brokkr;
CREATE USER brokkr WITH PASSWORD 'your-secure-password';
GRANT ALL PRIVILEGES ON DATABASE brokkr TO brokkr;

-- Grant schema creation permission
GRANT CREATE ON DATABASE brokkr TO brokkr;
```

You don't need to create the schemas manually — Brokkr creates them on first startup.

### Step B: Mint an Admin PAK Per Instance

First mint an admin PAK and hash for each instance. The chart does not do this for you, and a release that sets neither `broker.pakHash` nor `broker.pakHashExistingSecret` falls back to the publicly-known development admin PAK compiled into the broker binary — anyone would be able to authenticate as that instance's admin:

```bash
# One run per instance — save the printed PAK (the credential) and hash separately
brokkr-broker generate-pak   # -> staging admin PAK + hash
brokkr-broker generate-pak   # -> prod admin PAK + hash
```

(No local binary? The container image's entrypoint is the broker: `docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak`.)

### Step C: Deploy the Instances

#### Option A: Helm (Kubernetes)

Deploy each instance as a separate Helm release, passing that instance's hash:

```bash
# Instance: staging
helm install brokkr-staging oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-staging --create-namespace \
  --set postgresql.enabled=false \
  --set postgresql.external.host=postgres.example.com \
  --set postgresql.external.port=5432 \
  --set postgresql.external.database=brokkr \
  --set postgresql.external.username=brokkr \
  --set postgresql.external.password=your-secure-password \
  --set postgresql.external.schema=brokkr_staging \
  --set broker.pakHash=<staging-admin-pak-hash>

# Instance: prod
helm install brokkr-prod oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-prod --create-namespace \
  --set postgresql.enabled=false \
  --set postgresql.external.host=postgres.example.com \
  --set postgresql.external.port=5432 \
  --set postgresql.external.database=brokkr \
  --set postgresql.external.username=brokkr \
  --set postgresql.external.password=your-secure-password \
  --set postgresql.external.schema=brokkr_prod \
  --set broker.pakHash=<prod-admin-pak-hash>
```

`broker.pakHash` renders the hash into the broker ConfigMap in plaintext — fine for dev/test. For production, put each instance's hash in a pre-created Secret and reference it instead:

```bash
kubectl create secret generic prod-admin-pak-hash \
  --namespace brokkr-prod \
  --from-literal=BROKKR__BROKER__PAK_HASH=<prod-admin-pak-hash>

helm install brokkr-prod oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --namespace brokkr-prod --create-namespace \
  ... \
  --set broker.pakHashExistingSecret=prod-admin-pak-hash
```

> **Warning:** setting `broker.pakHash` to an empty string is the same as omitting it — the chart only renders `BROKKR__BROKER__PAK_HASH` when the value is non-empty, so an empty value silently leaves the publicly-known development admin PAK active. It does **not** make the broker generate a fresh PAK.

#### Option B: Environment Variables (Direct)

Run each broker with different schema settings. The broker's bind address is fixed at `0.0.0.0:3000` — there is no configuration option to change the port — so each instance's broker must run in its own container or on its own host (or behind its own port mapping):

```bash
# Host/container 1: staging broker
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=brokkr_staging \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve

# Host/container 2: prod broker
BROKKR__DATABASE__URL=postgres://brokkr:password@postgres.example.com:5432/brokkr \
BROKKR__DATABASE__SCHEMA=brokkr_prod \
BROKKR__LOG__LEVEL=info \
  brokkr-broker serve
```

Per-instance settings can be provided as `BROKKR__*` environment variables as shown above, or via a per-instance TOML file referenced by `BROKKR_CONFIG_FILE` (environment variables still override file values).

### Step D: First Startup

On first startup, each broker instance:

1. Creates the schema (`CREATE SCHEMA IF NOT EXISTS brokkr_staging`)
2. Runs all database migrations within the schema
3. Creates the admin role, storing the configured admin PAK hash (or generating a fresh PAK only if no hash is configured at all)

By default, `broker.pak_hash` is set to a publicly-known development hash, which would give **every instance the same well-known dev PAK**. For any real deployment, override it per instance. There are two approaches:

**Recommended (production):** Mint a PAK and its hash offline with `brokkr-broker generate-pak`, then set `BROKKR__BROKER__PAK_HASH` to the generated hash before first startup (for Helm, that is the `broker.pakHash` / `broker.pakHashExistingSecret` flow from Step C). This is the day-zero bootstrap flow — it touches no database and writes no keyfile, so you control the PAK from the start:

```bash
# Generate an admin PAK + SHA-256 hash offline (one run per instance)
brokkr-broker generate-pak
# Then set the hash for that instance's broker
BROKKR__BROKER__PAK_HASH="<generated-hash>" \
BROKKR__DATABASE__SCHEMA=brokkr_staging \
... brokkr-broker serve
```

**Alternative (direct/env-var runs only):** Explicitly set `BROKKR__BROKER__PAK_HASH` to an empty string to force the broker to generate a fresh PAK on first startup:

```bash
# Force per-instance PAK generation
BROKKR__BROKER__PAK_HASH="" \
BROKKR__DATABASE__SCHEMA=brokkr_staging \
... brokkr-broker serve
```

This only works when the variable is genuinely set to empty in the broker's environment. Via Helm, `--set broker.pakHash=""` does **not** do this — the chart drops the variable and the public default hash applies (see the warning in Step C). To force generation under the chart you would have to inject `BROKKR__BROKER__PAK_HASH: ""` through `extraEnv`; prefer the recommended flow instead.

When the broker generates a PAK this way, it writes the raw PAK to `/tmp/brokkr-keys/key.txt` inside the broker's filesystem — it is not logged. The file is written on the true first startup only (later restarts never recreate it) and is deleted on graceful shutdown, so **capture it promptly for each instance**:

```bash
# Kubernetes (only when generation was forced via extraEnv)
kubectl exec -n brokkr-staging <staging-broker-pod> -- cat /tmp/brokkr-keys/key.txt

# Direct/container
cat /tmp/brokkr-keys/key.txt
```

**Recovering a lost admin PAK:** when a hash is configured (the recommended flow), no key file is ever written — there is nothing to `cat` out of the pod. If you lose an instance's admin PAK, mint a replacement pair with `brokkr-broker generate-pak`, update that instance's configured hash (`BROKKR__BROKER__PAK_HASH`, or the chart's `broker.pakHash` / Secret), and run `brokkr-broker rotate admin` with that instance's configuration to store the new hash. A plain restart does not re-run the admin bootstrap.

### Step E: Verify Instance Separation

Each instance sees only its own data, and its admin PAK works only against its own broker:

```bash
curl -s http://staging-broker:3000/api/v1/agents \
  -H "Authorization: Bearer <staging-admin-pak>" | jq '.[].name'

curl -s http://prod-broker:3000/api/v1/agents \
  -H "Authorization: Bearer <prod-admin-pak>" | jq '.[].name'
```

Then onboard teams inside each instance using Steps 1–6 above, against that instance's broker URL and admin PAK.

### Connection Pool Sizing

Each broker instance uses a connection pool (default: 50 connections). With multiple instances on one database, the total connections across all of them must stay under PostgreSQL's `max_connections` (default: 100). Increase it or reduce per-instance pool sizes for many instances. See [Multi-Tenancy Reference](../reference/multi-tenancy.md#connection-pool-behavior) for detailed capacity planning.

### Schema Naming

Use a consistent pattern like `brokkr_{name}` (e.g. `brokkr_staging`). Brokkr requires schema names to start with a letter and contain only letters, numbers, and underscores. (PostgreSQL caps identifiers at 63 characters; Brokkr itself does not validate length.) See [Multi-Tenancy Reference](../reference/multi-tenancy.md#schema-name-constraints) for full constraints.

## Related Documentation

- [Multi-Tenancy Reference](../reference/multi-tenancy.md) — the tenant model, `GET /api/v1/paks`, and `pak_id` semantics
- [Working with Generators](generators.md) — generator lifecycle, CI/CD integration, and PAK rotation
- [Agent Registration](agent-registration.md) — registering and deregistering agents
- [Configuration Guide](../getting-started/configuration.md) — database configuration
- [Installation Guide](../getting-started/installation.md) — deployment options
