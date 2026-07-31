# Multi-Tenancy Reference

In Brokkr, **a tenant is a generator**. A generator is both an identity principal (it holds its own PAK) and a tenancy boundary: every stack it creates carries its `generator_id`, every agent that serves it must be registered with it, and every template it owns is private to it unless it is a system template. Multiple teams therefore share one broker, one database, and one admin, while remaining separated from each other at the application layer.

A second, unrelated feature is sometimes mistaken for the tenant model: the `database.schema` setting, which places a broker's tables in a named PostgreSQL schema. That is a **deployment and configuration** feature for running fully separate broker instances against one PostgreSQL server — useful for environment separation or hard data separation, but it is not how a team is onboarded onto a shared control plane. It is documented under [Running Separate Broker Instances](#running-separate-broker-instances-schema-separation).

| | Tenancy model | Schema separation |
|---|---|---|
| Unit | A generator | A broker instance |
| Boundary | Generator ownership + agent registration | Separate PostgreSQL schema |
| Brokers | One, shared | One per schema |
| Admin PAK | One, shared across tenants | One per instance |
| Onboarding a team | Create a generator, hand over its PAK | Deploy another broker |
| Typical use | Multiple teams or applications on one control plane | Environments (dev/staging/prod), hard data separation |

## The Tenant Model

### Tenant Identity

| Aspect | Behavior |
|--------|----------|
| Tenant record | A generator (`GET /api/v1/generators`), created by admin via `POST /api/v1/generators` |
| Tenant credential | The generator's PAK, returned once at creation and again on rotation |
| Tenant name | The generator's `name` (unique among non-deleted generators) |
| Tenant ID | The generator's `id` — the value used as `generator_id` on stacks and as `?pak_id=` in scoped listings |
| Lifecycle | Soft-delete cascades to the tenant's stacks and their deployment objects |

Creating, listing, and deleting generators is admin-only. A generator PAK can read and update its own generator record, rotate its own PAK, and **list the agents registered with it** (`GET /agents`, scoped; `GET /generators/{id}/registered-agents` for the raw registration records) — but it cannot create other generators or manage agents. See [Generators Reference](generators.md) for the full endpoint and permission matrix.

### What a Generator PAK Is Scoped To

| Resource | Scope under a generator PAK |
|----------|-----------------------------|
| Stacks | Only stacks whose `generator_id` is the caller's own. `GET /stacks` returns only those; stack creation must set `generator_id` to the caller's own ID |
| Deployment objects | Inherit `generator_id` from the parent stack |
| Templates | The generator's own templates, plus system templates (`generator_id = null`). Reading or instantiating another generator's template returns `403` with code `template_not_accessible` |
| Agents | Not manageable. Agent creation, deletion, and registration are admin (or agent-self) operations |
| Fleet | Not readable. `GET /fleet` and `GET /agent-events` are admin-only |

### Registration Is the Consent Boundary

An agent must be **registered** with a generator before that generator's stacks reach it. Registration is the agent's opt-in: the generator declares which labels it publishes to; the agent's registrations declare which generators it accepts work from. Matching selects *within* the generators an agent has consented to — it never creates responsibility across generators.

This applies to every association path:

| Association path | Consent enforcement |
|------------------|--------------------|
| Explicit target (`POST /agents/{id}/targets`) | Gated at creation. Targeting an agent that is not registered with the stack's owning generator returns `403` with code `agent_not_registered`. Because creation is gated, existing targets are served without a further check |
| Shared label (agent label == stack label) | Filtered at read time. A label match only associates the agent with the stack when the agent is registered with the stack's owning generator |
| Shared annotation (agent key/value == stack key/value) | Filtered at read time, on the same rule as labels |

Consequences:

- An agent with **no registrations** receives nothing from label or annotation matching.
- Two tenants may use identical label or annotation values without colliding. A label match against another tenant's stack does not deliver that stack.
- The served set returned by `GET /agents/{id}/target-state` — and the broker-computed pending-object counts that back the fleet rollup — obey the same rule.
- There is **no admin override**. Admin cannot force-target an unregistered agent; the escape hatch is to register the agent first, which is one audited call.

For the rationale, see [Security Model](../explanation/security-model.md#generator-registration-and-application-scopes); for operational steps, see [Agent Registration](../how-to/agent-registration.md).

### System Generator

| Property | Value |
|----------|-------|
| Name | `__system__` (`is_system = true`) |
| Provisioned | At broker startup |
| Auto-registration | Every agent created through `POST /agents` is registered with it |
| Purpose | Carries fleet/system stacks that reach all agents without per-tenant registration |
| Listing | Excluded from `GET /generators` and from `GET /paks` |
| Tenant-scoped reads | Refused with `system_generator_not_a_tenant` (403) for non-admin callers |

The system generator is **not** the admin generator. The admin generator is a separate entity tied to the admin role/PAK; agents are not auto-registered with it.

It is also **not a tenant**, which is why it is excluded from every tenant-facing surface: the generator and PAK listings, the console's scope selector, and the tenant-scoped agent reads. Because every agent is auto-registered with it, treating it as a tenant would quietly turn any scoped query into a fleet-wide one. The broker provisions it without a PAK, so nothing can authenticate as it; the 403 is defense in depth rather than a reachable path today.

### Agent Self-Registration at Startup

An agent registers itself with generators at startup. Sources, in precedence order (highest first):

1. `--generator-ids` CLI flag
2. `BROKKR__AGENT__GENERATOR_IDS` config (config key `agent.generator_ids`) — see [Environment Variables](environment-variables.md)
3. `BROKKR_GENERATOR_IDS` legacy bare env var (**deprecated**, still honored, logs a warning)

Values are comma-separated UUIDs; malformed entries are skipped with a warning; an empty value means system/fleet scope only. For the Helm chart, set `broker.generatorIds`, which renders to `BROKKR__AGENT__GENERATOR_IDS` in the agent ConfigMap.

Registration can also be managed out of band: pass optional `generator_ids` to `POST /agents`, use the `register`/`deregister`/`registrations` `brokkr` CLI commands (see [CLI Reference](cli.md)), or call the registration endpoints documented in the [API Reference](api/README.md).

## Listing Tenants

### `GET /api/v1/paks`

Lists tenants as named PAK owners, reduced to identity only. The path is `/api/v1/paks`; it is grouped under the `auth` tag in the OpenAPI document, but it is **not** under `/api/v1/auth/`.

| Aspect | Value |
|--------|-------|
| Path | `GET /api/v1/paks` |
| Auth | Admin. The operator console's ephemeral read-only PAK is a read-only admin and qualifies |
| Returns | `200` with a JSON array of `{ "id": <uuid>, "name": <string> }` |
| Contents | All non-deleted, non-system generators. The system generator is never included |
| Errors | `403` `admin_required` for non-admin callers; `500` on internal failure |

```json
[
  { "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890", "name": "team-acme" },
  { "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901", "name": "team-globex" }
]
```

This endpoint exists so a scope selector can be populated without fetching the full generator model. For generator metadata (description, activity timestamps, active flag), use `GET /api/v1/generators`.

## Scoped Views: the `pak_id` Query Parameter

Three admin listing endpoints accept `?pak_id=<generator uuid>` to narrow the result to one tenant:

| Endpoint | Effect of `?pak_id=` |
|----------|---------------------|
| `GET /api/v1/fleet` | Keeps only fleet records for agents registered with that generator |
| `GET /api/v1/stacks` | Returns only stacks owned by that generator. Meaningful only on the admin path — a generator PAK already sees only its own stacks, and the parameter does not widen that |
| `GET /api/v1/agent-events` | Returns only events from agents registered with that generator |

| Input | Result |
|-------|--------|
| Omitted | Unscoped listing (all tenants) |
| A known generator ID | The listing narrowed to that tenant |
| An unknown but well-formed UUID | `200` with an empty list |
| A malformed UUID | `400` |

> **`pak_id` is a view filter, not an authorization boundary.** It narrows what a caller *sees*; it does not change what a caller is *permitted* to see. The caller must already be admin to reach these endpoints at all, and an admin who omits the parameter sees everything. Do not build a multi-team access model on `pak_id`.

Real isolation comes from two places:

1. **The generator PAK's own scoping** — a generator PAK is confined to its own stacks, deployment objects, and templates by the handlers themselves.
2. **Agent registration** — an agent only receives stacks from generators it has registered with.

## Credential Introspection

`POST /api/v1/auth/pak` reports which identity the supplied PAK resolves to:

| Field | Type | Meaning |
|-------|------|---------|
| `admin` | boolean | The PAK carries the admin role |
| `agent` | string \| null | The agent's UUID, when the PAK is an agent PAK |
| `generator` | string \| null | The generator (tenant) UUID, when the PAK is a generator PAK |
| `readonly` | boolean | The credential may not mutate state |

`readonly` is `true` only for the operator console's ephemeral read-only PAK, which the broker mints per process, holds in memory, and never persists. A read-only credential passes every `GET`/`HEAD` request plus two allowlisted `POST` routes (`/auth/pak` and deployment-object diagnostics); anything else is rejected with `403`. Every other PAK — admin, agent, generator — reports `readonly: false`.

A generator can use this endpoint to discover its own tenant ID:

```bash
curl -s -X POST http://broker:3000/api/v1/auth/pak \
  -H "Authorization: Bearer $GENERATOR_PAK" | jq -r '.generator'
```

## Running Separate Broker Instances (Schema Separation)

Setting `database.schema` places one broker's tables in a named PostgreSQL schema, so several independent broker instances can share a single PostgreSQL server without seeing each other's data. This is a deployment/configuration feature, independent of the tenant model above. Use it for environment separation (dev/staging/prod on one database server) or when data must be separated at the storage layer rather than the application layer. Each instance still has its own tenants (generators) inside it.

### Behavior

When `database.schema` is set, the broker creates the schema on startup (`CREATE SCHEMA IF NOT EXISTS`), sets `search_path` on every connection checkout, and runs migrations within the schema. Each instance has its own complete set of tables invisible to the others.

### Configuration

No schema configuration is needed for a single instance; all data lives in the `public` schema.

```toml
[database]
url = "postgres://brokkr:password@db:5432/brokkr"
```

To place an instance in its own schema:

```toml
[database]
url = "postgres://brokkr:password@db:5432/brokkr"
schema = "brokkr_staging"
```

Or via environment variable:

```bash
BROKKR__DATABASE__SCHEMA=brokkr_staging
```

### Schema Name Constraints

Schema names are validated to prevent SQL injection:

- **Must be non-empty** and **must start with a letter** (`a-z`, `A-Z`)
- **Allowed characters:** alphanumeric (`a-z`, `A-Z`, `0-9`) and underscores (`_`)
- **Maximum length:** limited by PostgreSQL (63 characters)
- **No special characters**, spaces, or SQL keywords

Valid examples: `brokkr_staging`, `org_12345`, `production_v2`

Invalid examples: `brokkr-staging` (hyphen), `1tenant` (leading digit), `_internal` (leading underscore), `drop table;` (SQL injection), `my schema` (space)

### Deployment Topology

You run one broker process per schema, all pointing to the same PostgreSQL server:

```
┌──────────────────────────────────────────┐
│             PostgreSQL Server             │
│  ┌────────────────┐  ┌────────────────┐  │
│  │ brokkr_staging │  │  brokkr_prod   │  │
│  │  agents        │  │  agents        │  │
│  │  stacks        │  │  stacks        │  │
│  │  generators    │  │  generators    │  │
│  └────────────────┘  └────────────────┘  │
└──────────────────────────────────────────┘
          ▲                     ▲
          │                     │
┌─────────┴────────┐  ┌─────────┴────────┐
│ Broker (staging) │  │  Broker (prod)   │
│ schema=          │  │  schema=         │
│ brokkr_staging   │  │  brokkr_prod     │
└──────────────────┘  └──────────────────┘
```

Each broker instance:
- Has its own admin PAK
- Manages its own agents and generators
- Runs its own migrations on startup
- Operates independently

### Connection Pool Behavior

When a schema is configured:

- The connection pool calls `setup_schema(schema)` at initialization
- Every connection acquired from the pool automatically executes `SET search_path TO <schema>` before use
- This happens at the r2d2 pool level, so application code doesn't need schema awareness

The connection pool size is 50 by default. The total connection count across all broker instances on one database server is bounded by PostgreSQL's `max_connections`; each additional instance adds up to one pool's worth of connections.

### Data Separation Guarantees

| Aspect | Separation level |
|--------|-----------------|
| Tables | Full — each schema has its own tables |
| Sequences | Full — sequence counters are per-schema |
| Migrations | Full — each schema migrates independently |
| Admin PAK | Full — each instance has its own admin |
| Agents | Full — agents belong to one instance |
| Generators | Full — generators belong to one instance |

**Not separated:**
- PostgreSQL server resources (CPU, memory, disk, connections)
- Network access to the database server
- Database-level settings (e.g., `max_connections`)

Stronger separation requires separate PostgreSQL databases or servers.

### Migration Behavior

| Startup | What Happens |
|---------|-------------|
| First | All migrations + admin role creation + admin PAK generation |
| Subsequent | Pending migrations only |

Each schema has its own `app_initialization` table. Different instances can be at different migration versions if their brokers are updated at different times.

### Limitations

- **No cross-instance queries**: a broker can only see data in its configured schema
- **No API across instances**: an instance is created by configuring and deploying another broker; there is no endpoint that enumerates or manages other instances. (Tenants *within* an instance do have an API — see [Listing Tenants](#listing-tenants))
- **Shared database resources**: high load on one instance can affect others on the same database server
- **Schema name is static**: changing an instance's schema name requires data migration

### Kubernetes Deployment

For the Helm-based per-instance walkthrough, see [Running Separate Broker Instances](../how-to/multi-tenant-setup.md#running-separate-broker-instances-schema-separation) in the multi-tenant setup guide.

## Related Documentation

- [Multi-Tenant Setup](../how-to/multi-tenant-setup.md) — onboarding teams onto a shared broker, and the separate-instance walkthrough
- [Generators Reference](generators.md) — full generator API, permissions, and resource scoping
- [Agent Registration](../how-to/agent-registration.md) — registering agents with generators
- [Configuration Guide](../getting-started/configuration.md) — database configuration options
- [Installation Guide](../getting-started/installation.md) — deployment options including external PostgreSQL
- [Security Model](../explanation/security-model.md) — authentication and authorization
