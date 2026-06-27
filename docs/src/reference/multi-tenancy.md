# Multi-Tenancy Reference

Brokkr supports multi-tenancy through two complementary, independent mechanisms:

- **Schema-per-tenant isolation** — deployment-level isolation using PostgreSQL schemas. Each tenant gets a separate database schema and its own broker instance, sharing a single database server. Covered below.
- **Application-level isolation via generator registration** — logical isolation within a single broker. An agent must register with a generator (an application scope) before any stack owned by that generator can be targeted at it. See [Application-Level Isolation via Generator Registration](#application-level-isolation-via-generator-registration).

The two are not the same mechanism and do not substitute for each other; see [Schema-Per-Tenant vs. Application-Level Isolation](#schema-per-tenant-vs-application-level-isolation).

## Behavior

When `database.schema` is set, the broker creates the schema on startup (`CREATE SCHEMA IF NOT EXISTS`), sets `search_path` on every connection checkout, and runs migrations within the schema. Each tenant has its own complete set of tables invisible to other tenants.

## Configuration

### Single Tenant (Default)

No schema configuration needed. All data lives in the `public` schema.

```toml
[database]
url = "postgres://brokkr:password@db:5432/brokkr"
```

### Multi-Tenant

Set the schema for each tenant's broker instance:

```toml
[database]
url = "postgres://brokkr:password@db:5432/brokkr"
schema = "tenant_acme"
```

Or via environment variable:

```bash
BROKKR__DATABASE__SCHEMA=tenant_acme
```

### Schema Name Constraints

Schema names are validated to prevent SQL injection:

- **Allowed characters:** alphanumeric (`a-z`, `A-Z`, `0-9`) and underscores (`_`)
- **Maximum length:** limited by PostgreSQL (63 characters)
- **No special characters**, spaces, or SQL keywords

Valid examples: `tenant_acme`, `org_12345`, `production_v2`

Invalid examples: `tenant-acme` (hyphen), `drop table;` (SQL injection), `my schema` (space)

## Deployment Topology

In a multi-tenant deployment, you run one broker process per tenant, all pointing to the same PostgreSQL server but with different schema configurations:

```
┌──────────────────────────────────────┐
│            PostgreSQL Server          │
│  ┌────────────┐  ┌────────────────┐  │
│  │ tenant_acme│  │ tenant_globex  │  │
│  │  agents    │  │  agents        │  │
│  │  stacks    │  │  stacks        │  │
│  │  ...       │  │  ...           │  │
│  └────────────┘  └────────────────┘  │
└──────────────────────────────────────┘
        ▲                    ▲
        │                    │
┌───────┴───────┐  ┌────────┴────────┐
│ Broker (Acme) │  │ Broker (Globex) │
│ schema=       │  │ schema=         │
│ tenant_acme   │  │ tenant_globex   │
└───────────────┘  └─────────────────┘
```

Each broker instance:
- Has its own admin PAK
- Manages its own agents and generators
- Runs its own migrations on startup
- Operates independently

## Kubernetes Deployment

For the Helm-based per-tenant deployment walkthrough, see [Multi-Tenant Setup](../how-to/multi-tenant-setup.md).

## Application-Level Isolation via Generator Registration

Generators are application scopes. Within a single broker, an agent must be **registered** with a generator before any stack owned by that generator can be targeted at the agent. Registration is the agent's opt-in consent boundary; it is separate from and complementary to schema-per-tenant isolation.

For the concept, see [Security Model](../explanation/security-model.md#generator-registration-and-application-scopes); for operational steps, see [Agent Registration](../how-to/agent-registration.md).

### Enforcement

| Aspect | Behavior |
|--------|----------|
| Gated operations | Creating (`POST /agents/{id}/targets`) and removing (`DELETE /agents/{id}/targets/{stack_id}`) explicit targets |
| Not gated | The read path `GET /agents/{id}/target-state`; an agent's served-stack set is still the union of explicit agent_targets, label matches, and annotation matches |
| Unregistered target write | `403` with error code `agent_not_registered` — see [Error Codes](error-codes.md) |
| Admin override | None. There is no force flag; admin cannot bypass registration |
| Existing targets | Remain valid. Migration 23 back-fills registrations from existing agent_targets |

### System Generator

| Property | Value |
|----------|-------|
| Name | `__system__` (`is_system = true`) |
| Provisioned | At broker startup |
| Auto-registration | Every agent is auto-registered with it at creation (`POST /agents`) |
| Purpose | Carries fleet/system stacks that reach all agents |
| Listing | Excluded from the public `GET /generators` listing |

The system generator is **not** the admin generator. The admin generator is a separate entity tied to the admin role/PAK; agents are not auto-registered with it.

### Agent Self-Registration at Startup

An agent registers itself with generators at startup. Sources, in precedence order (highest first):

1. `--generator-ids` CLI flag
2. `BROKKR__AGENT__GENERATOR_IDS` config (config key `agent.generator_ids`) — see [Environment Variables](environment-variables.md)
3. `BROKKR_GENERATOR_IDS` legacy bare env var (**deprecated**, still honored, logs a warning)

Values are comma-separated UUIDs; malformed entries are skipped with a warning; an empty value means system/fleet scope only. For the Helm chart, set `broker.generatorIds`, which renders to `BROKKR__AGENT__GENERATOR_IDS` in the agent ConfigMap.

Registration can also be managed out of band: pass optional `generator_ids` to `POST /agents`, use the `register`/`deregister`/`registrations` `brokkr` CLI commands (see [CLI Reference](cli.md)), or call the registration endpoints documented in the [API Reference](api/README.md). For generator details, see [Generators Reference](generators.md).

## Schema-Per-Tenant vs. Application-Level Isolation

The two mechanisms are independent. Use either, both, or neither as your isolation needs require.

| Aspect | Schema-Per-Tenant | Application-Level (Generator Registration) |
|--------|-------------------|--------------------------------------------|
| Isolation level | Deployment-level | Application-level (within one broker) |
| Scope | One broker per tenant | Multiple applications (generators) per broker |
| Physical separation | Separate database schemas | Shared schema |
| Configuration | `database.schema` setting | Agent `generator_ids` registration |
| Use case | Separate tenants on shared infrastructure | Multi-application authorization within one broker |

## Connection Pool Behavior

When a schema is configured:

- The connection pool calls `setup_schema(schema)` at initialization
- Every connection acquired from the pool automatically executes `SET search_path TO <schema>` before use
- This happens at the r2d2 pool level, so application code doesn't need schema awareness

The connection pool size is 50 by default. The total connection count across all broker instances on one database server is bounded by PostgreSQL's `max_connections`; each additional tenant adds up to one pool's worth of connections.

## Data Isolation Guarantees

| Aspect | Isolation Level |
|--------|-----------------|
| Tables | Full — each schema has its own tables |
| Sequences | Full — sequence counters are per-schema |
| Migrations | Full — each schema migrates independently |
| Admin PAK | Full — each tenant has its own admin |
| Agents | Full — agents belong to one tenant |
| Generators | Full — generators belong to one tenant |

**Not isolated:**
- PostgreSQL server resources (CPU, memory, disk, connections)
- Network access to the database server
- Database-level settings (e.g., `max_connections`)

Stronger isolation requires separate PostgreSQL databases or servers.

## Migration Behavior

| Startup | What Happens |
|---------|-------------|
| First | All migrations + admin role creation + admin PAK generation |
| Subsequent | Pending migrations only |

Each schema has its own `app_initialization` table. Different tenants can be at different migration versions if their broker instances are updated at different times.

## Limitations

- **No cross-tenant queries**: the broker can only see data in its configured schema
- **No tenant management API**: tenants are created by configuring new broker instances; there's no API to list or manage tenants
- **Shared database resources**: high load on one tenant can affect others on the same database server
- **Schema name is static**: changing a tenant's schema name requires data migration

## Related Documentation

- [Configuration Guide](../getting-started/configuration.md) — database configuration options
- [Installation Guide](../getting-started/installation.md) — deployment options including external PostgreSQL
- [Security Model](../explanation/security-model.md) — authentication and authorization
