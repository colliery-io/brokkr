---
id: 004-schema-per-tenant-multi-tenancy
level: adr
title: "Schema-Per-Tenant Multi-Tenancy Architecture"
number: 4
short_code: "BROKKR-A-0004"
created_at: 2025-10-22T00:00:00.000000+00:00
updated_at: 2025-10-22T00:00:00.000000+00:00
decision_date: 2025-10-22
decision_maker: Dylan Storey
parent:
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0004
---

# ADR-4: Schema-Per-Tenant Multi-Tenancy Architecture

> **Implementation Status: Not Implemented**
> This ADR documents the decision to use schema-per-tenant multi-tenancy. Implementation has not yet begun. The current broker operates in single-tenant mode using the public schema.

## Context

Brokkr currently requires a dedicated PostgreSQL database per deployment instance. This creates significant operational overhead when managing:

**Multiple environments:**
- Development, staging, production deployments
- Each requires separate PostgreSQL instance
- Increased infrastructure costs
- More databases to backup, monitor, maintain

**Multi-customer SaaS scenarios:**
- Each customer deployment needs isolated database
- Dedicated PostgreSQL per customer is costly
- Operational complexity scales linearly with customers

**Current limitations:**
- No support for sharing PostgreSQL infrastructure
- Each broker deployment = one database requirement
- Database consolidation not possible
- Increased operational burden for multi-environment setups

**Proven solution exists:**
The cloacina project has successfully implemented a schema-per-tenant multi-tenancy pattern using PostgreSQL schemas. This pattern enables:
- Multiple application instances sharing one PostgreSQL server
- Complete data isolation through PostgreSQL schema separation
- `SET search_path` automatically routing queries to correct tenant
- Production-proven in cloacina with no isolation issues

## Decision

Implement **schema-per-tenant multi-tenancy** using PostgreSQL schemas, porting the proven cloacina implementation to brokkr.

**Architecture choice:** Option A - Schema from Configuration
- Each broker deployment connects to specific PostgreSQL schema
- Schema name determined by environment variable (`BROKKR__DATABASE__SCHEMA`)
- Multiple broker deployments share single PostgreSQL instance
- Each broker operates in isolated schema context

**Implementation approach:**
```rust
// ConnectionPool gains schema awareness
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub schema: Option<String>,  // NEW: tenant schema
}

// Automatic schema routing on connection
pub fn get_connection_with_schema(&self) -> Result<Connection> {
    let conn = self.pool.get()?;
    if let Some(ref schema) = self.schema {
        conn.execute(&format!("SET search_path TO {}, public", schema))?;
    }
    Ok(conn)
}
```

**Configuration:**
```yaml
# Helm values.yaml
database:
  url: "postgres://host:5432/brokkr_db"
  schema: "customer_a"  # Optional: enables multi-tenancy
```

**Backward compatibility:**
- `schema: None` → uses public schema (current behavior)
- `schema: Some("tenant_a")` → uses tenant_a schema
- Zero breaking changes to existing deployments

## Alternatives Analysis

### Alternative 1: Shared Tables with tenant_id Column

**Approach:** Add `tenant_id` column to all tables, filter queries by tenant.

**Rejected because:**
- Application-level isolation (risky - bugs can leak data)
- Every query must include tenant_id filtering
- Requires schema changes to ALL existing tables
- Performance impact from additional filtering
- Complex Row-Level Security policies needed
- Difficult to audit and verify isolation

### Alternative 2: Separate Databases Per Tenant

**Approach:** Each tenant gets completely separate PostgreSQL database.

```
postgres://localhost:5432/tenant_a
postgres://localhost:5432/tenant_b
postgres://localhost:5432/tenant_c
```

**Rejected because:**

**Resource Overhead:**
- Each database has its own system catalogs (pg_catalog tables)
- Higher memory overhead per tenant
- More file handles and connections consumed
- 1000 tenants = 1000 databases with 1000 sets of system tables

**Cost at Scale:**
- Some cloud providers (RDS, Azure) charge per database
- Can get expensive with many tenants
- Example: 100 small tenants might cost same as 10 large ones

**Connection Pooling Complexity:**
- Need separate connection pool per tenant database
- OR complex dynamic database switching logic
- Cannot share connection pool across tenants efficiently

**Migration Overhead:**
- Must run migrations against EACH database separately
- Provisioning new tenant requires full database creation (slower)
- Backup/restore requires per-database operations

**Operational Complexity:**
- More databases to monitor, backup, and maintain
- Scales linearly with tenant count
- Schema-per-tenant provides same isolation with better efficiency

**When database-per-tenant IS appropriate:**
- Very few tenants (< 10)
- Each tenant is very large
- Extreme isolation requirements (PCI, HIPAA with physical separation)
- Need to physically separate tenant data to different servers
- Tenants have different SLAs or performance requirements

**For Brokkr:** Our target is many small-to-medium tenants where operational efficiency and cost matter, making schema-per-tenant the better choice.

### Alternative 3: Dynamic Schema Routing (Option B)

**Approach:** Single broker switches schemas based on authentication at runtime.

**Deferred because:**
- Significantly more complex implementation
- Requires passing tenant context through entire request lifecycle
- Risk of schema-switching bugs causing data leaks
- Option A satisfies immediate requirements
- Can be added later if runtime multi-tenancy needed
- Prefer simpler deployment model first

### Alternative 4: Different PostgreSQL Users Per Tenant

**Approach:** Each broker connects as different PostgreSQL user with schema-specific access.

**Rejected because:**
- Requires managing multiple database credentials
- Connection pool cannot be shared across users
- Complex credential rotation and management
- PostgreSQL user management overhead
- Schema-based approach is simpler and sufficient

## Rationale

### Quick Comparison: Schema vs Database Per Tenant

| Aspect | Schema-per-Tenant (Chosen) | Database-per-Tenant |
|--------|---------------------------|---------------------|
| **Resource Overhead** | Low (shared catalogs) | High (separate catalogs per DB) |
| **Cost at Scale** | Linear with data size | Linear with tenant count |
| **Connection Pooling** | Single shared pool | Pool per tenant or complex switching |
| **Migrations** | Once, iterate schemas | Once per database |
| **Backup/Restore** | Single backup for all | Separate backup per tenant |
| **Tenant Provisioning** | Fast (CREATE SCHEMA) | Slower (CREATE DATABASE) |
| **Cross-Tenant Analytics** | Easy (same DB) | Complex (requires dblink) |
| **Isolation Strength** | Good (search_path) | Strongest (physical separation) |
| **Best For** | Many small-medium tenants | Few large tenants |

Schema-per-tenant was chosen for these reasons:

### 1. Proven in Production (De-Risked)

Cloacina project has used this pattern successfully:
- Production-tested data isolation
- Known implementation patterns
- Existing code to port from
- No surprises or unknowns

### 2. PostgreSQL-Enforced Isolation

**Strong isolation guarantee:**
```sql
-- Set search path to tenant schema
SET search_path TO customer_a, public;

-- All queries automatically scoped to customer_a
SELECT * FROM agents;  -- Only sees customer_a.agents
INSERT INTO stacks ... -- Inserts into customer_a.stacks
```

Benefits:
- Impossible to accidentally query wrong tenant's data
- No application-level filtering required
- PostgreSQL enforces permissions at schema level
- Clear separation of concerns

### 3. Minimal Code Changes

**DAL layer changes are localized:**

Before:
```rust
let mut conn = self.dal.pool.get()?;
diesel::insert_into(agents::table).execute(&mut conn)
```

After:
```rust
let mut conn = self.dal.pool.get_connection_with_schema()?;
diesel::insert_into(agents::table).execute(&mut conn)  // Same query
```

- No query changes needed
- No schema changes to tables
- No migration of existing data
- Just connection acquisition changes

### 4. Operational Efficiency

**Single PostgreSQL for multiple deployments:**
```
PostgreSQL Instance
├── dev_schema (dev broker)
├── staging_schema (staging broker)
└── prod_schema (prod broker)
```

Benefits:
- One database to backup
- One database to monitor
- Shared connection pool efficiency
- Lower infrastructure costs
- Simplified operations

### 5. Flexible Deployment Patterns

Schema-per-tenant **adds** flexibility without removing existing options. The broker is lightweight enough to run multiple instances cost-effectively.

**Use case 1: Multi-environment (shared DB)**
```
postgres://db:5432/brokkr_db?schema=dev
postgres://db:5432/brokkr_db?schema=staging
postgres://db:5432/brokkr_db?schema=prod
```

**Use case 2: Multi-customer small tenants (shared DB)**
```
postgres://db:5432/brokkr_db?schema=customer_a
postgres://db:5432/brokkr_db?schema=customer_b
postgres://db:5432/brokkr_db?schema=customer_c
```

**Use case 3: Large customers (dedicated DBs)**
```
# Customer X gets dedicated database
postgres://db-customerx:5432/brokkr_db

# Customer Y gets dedicated database
postgres://db-customery:5432/brokkr_db
```

**Use case 4: Hybrid "Choose Your Own Adventure"**
```
# Small customers share a database with schemas
postgres://shared-db:5432/brokkr_db?schema=customer_a
postgres://shared-db:5432/brokkr_db?schema=customer_b

# Enterprise customer gets dedicated database
postgres://enterprise-db:5432/brokkr_db

# Compliance-critical customer gets isolated everything
postgres://hipaa-db:5432/brokkr_db  # On separate server
```

**Key insight:** You're not choosing schema-per-tenant OR database-per-tenant. You're gaining the ability to use schemas when it makes sense, while still being able to use dedicated databases when needed. The broker is cheap to run, so mix and match freely.

## Consequences

### Positive

**Operational Benefits:**
- Reduced infrastructure costs (fewer PostgreSQL instances)
- Simplified database management (one DB to operate)
- Better resource utilization (shared connections)
- Easier backup and restore (one database)
- Consolidated monitoring and alerting

**Data Isolation:**
- PostgreSQL-enforced separation (not application-enforced)
- No risk of cross-tenant data leakage
- Clear security boundary
- Auditable and verifiable isolation

**Development:**
- Minimal code changes required
- Backward compatible (schema=None works)
- Well-understood pattern (from cloacina)
- Low implementation risk
- Clear migration path

**Flexibility:**
- Supports multiple deployment patterns
- Users choose single or multi-tenant
- Can mix approaches (some tenants in shared DB, some dedicated)
- Future-proof (can add dynamic routing later)

**Not Either/Or - Choose Your Own Adventure:**
- Schema-per-tenant doesn't prevent database-per-tenant
- The broker is very inexpensive to run (lightweight binary)
- Can easily run multiple broker instances, each with dedicated database
- Mix strategies: shared database for small tenants, dedicated for large ones
- Operators choose the right approach for each tenant's needs
- This feature adds options, it doesn't remove any

### Negative

**Implementation Complexity:**
- New schema configuration in config system
- Connection pool changes required
- All DAL methods must be updated
- Schema provisioning tooling needed
- Migration documentation required

**Operational Considerations:**
- Schema-level backup/restore more complex than database-level
- Need schema provisioning process
- Schema naming conventions required
- Monitoring must track per-schema metrics

**Migration Work:**
- Existing deployments must be migrated to use schema configuration
- Documentation needed for migration path
- Testing required for backward compatibility

**Performance:**
- Minimal overhead from `SET search_path` (~1ms per connection)
- Connection pool must handle schema parameter
- Trade-off: Acceptable given operational benefits

**PostgreSQL Limitations:**
- All schemas share same PostgreSQL resource limits
- Cannot independently scale per tenant (need separate DB for that)
- Schema count limits (PostgreSQL can handle thousands, but not unlimited)

## References

**Pattern Source:**
- Cloacina project: `colliery/cloacina` - production implementation
- PostgreSQL Schema Documentation: https://www.postgresql.org/docs/current/ddl-schemas.html

**Related Initiatives:**
- BROKKR-I-0004: Multi-Tenant Database Architecture Initiative
- BROKKR-I-0001: Ephemeral Work System (will benefit from multi-tenancy)
- BROKKR-I-0002: Stack Templating System (will benefit from multi-tenancy)

**Multi-Tenancy Patterns:**
- Schema-per-tenant (chosen)
- Database-per-tenant (alternative)
- Row-level multi-tenancy (alternative)

## Status Updates

### 2025-10-22: Decision Made

**Decision:** Implement schema-per-tenant multi-tenancy using PostgreSQL schemas, porting the proven cloacina pattern.

**Key decision drivers:**
- Operational efficiency (fewer PostgreSQL instances to manage)
- Strong data isolation (PostgreSQL-enforced)
- Proven pattern (de-risked from cloacina)
- Minimal code changes (backward compatible)
- Flexible deployment options (multi-env, multi-customer)

**Trade-off accepted:** Additional implementation complexity and schema management overhead in exchange for operational efficiency and cost reduction.

**Implementation tracking:** See BROKKR-I-0004 for implementation work (7 phases, ~10-12 weeks).

**Next steps:**
1. Transition BROKKR-I-0004 to ready phase
2. Decompose initiative into tasks
3. Begin Phase 1: Database Infrastructure Foundation
