# Multi-Tenancy Reference

Brokkr supports multi-tenant deployments through PostgreSQL schema isolation. Each tenant gets a separate database schema, providing logical separation of all data while sharing a single database server, with one broker instance per tenant.

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
