---
id: multi-tenant-database-architecture
level: initiative
title: "Multi-Tenant Database Architecture"
short_code: "BROKKR-I-0004"
created_at: 2025-10-22T00:02:38.961232+00:00
updated_at: 2025-10-22T18:16:47.607070+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: multi-tenant-database-architecture
---

# Multi-Tenant Database Architecture Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

Currently, brokkr requires a dedicated PostgreSQL database per deployment instance. This creates significant operational overhead when managing multiple environments (dev/staging/prod) or deploying multiple broker instances for different customers/organizations.

The cloacina project has successfully implemented a schema-per-tenant multi-tenancy pattern that enables multiple application instances to share a single PostgreSQL server while maintaining complete data isolation through PostgreSQL schemas. Each tenant's data lives in its own schema, with `SET search_path` automatically routing queries to the correct tenant without requiring application-level filtering.

This initiative will port the proven cloacina implementation to brokkr, specifically focusing on **Option A: Schema from Configuration** - where each broker deployment connects to a specific PostgreSQL schema determined by an environment variable, enabling multiple broker deployments to share a single PostgreSQL instance.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Enable multiple broker deployments to share a single PostgreSQL instance with complete data isolation
- Port the schema-per-tenant pattern from cloacina to brokkr with minimal changes to existing DAL code
- Support schema configuration via environment variables for flexible deployment patterns
- Maintain zero-downtime deployment capability and backward compatibility with single-tenant deployments
- Provide automated schema provisioning and migration tooling
- Document deployment patterns for multi-environment and multi-customer scenarios

**Non-Goals:**
- Dynamic schema routing based on runtime authentication (Option B) - this is deferred for future consideration
- Database-per-tenant or separate PostgreSQL instances (existing pattern already supports this)
- Row-Level Security (RLS) policies - schema isolation provides sufficient separation
- Cross-tenant queries or data aggregation
- Multi-region or geo-distributed database architecture

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

### System Requirements

**Functional Requirements:**
- REQ-001: Database connection pool must support optional schema configuration
- REQ-002: All database connections must automatically execute `SET search_path` when schema is configured
- REQ-003: Schema provisioning must create schema and run all migrations automatically
- REQ-004: Configuration system must accept `BROKKR__DATABASE__SCHEMA` environment variable
- REQ-005: System must maintain backward compatibility with non-schema deployments (schema=None)
- REQ-006: DAL method signatures must remain unchanged (only internal implementation changes)

**Non-Functional Requirements:**
- NFR-001: Schema switching must not introduce measurable query performance degradation
- NFR-002: Data isolation must be PostgreSQL-enforced (not application-enforced)
- NFR-003: Schema configuration must be set at broker startup (not runtime-changeable)
- NFR-004: Connection pool behavior must remain unchanged except for search_path execution
- NFR-005: Migration tooling must be idempotent and safe to re-run

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

### Use Case 1: Multi-Environment Deployment (Dev/Staging/Prod)
- **Actor**: Platform Operations Engineer
- **Scenario**:
  1. Provision single PostgreSQL instance with three schemas: `brokkr_dev`, `brokkr_staging`, `brokkr_prod`
  2. Deploy three broker instances with respective schema configurations
  3. Each broker operates independently with complete data isolation
  4. Simplified database management with one PostgreSQL to monitor, backup, and maintain
- **Expected Outcome**: Reduced operational overhead while maintaining environment isolation

### Use Case 2: Multi-Customer SaaS Deployment
- **Actor**: SaaS Provider
- **Scenario**:
  1. Provision PostgreSQL with customer-specific schemas: `customer_a`, `customer_b`, `customer_c`
  2. Deploy dedicated broker instance per customer with their schema name
  3. Each customer's data is completely isolated from others
  4. Simplified database infrastructure management
- **Expected Outcome**: Cost-efficient multi-customer deployment with PostgreSQL-enforced data isolation

### Use Case 3: Migration from Separate Databases
- **Actor**: Infrastructure Engineer
- **Scenario**:
  1. Currently running multiple broker deployments with separate PostgreSQL instances
  2. Create schemas in consolidated PostgreSQL instance
  3. Migrate data from separate databases to respective schemas
  4. Update broker configurations to point to new PostgreSQL with schema names
  5. Decommission old PostgreSQL instances
- **Expected Outcome**: Reduced infrastructure costs and operational complexity

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

### Overview

The architecture follows the proven cloacina pattern:

```
PostgreSQL Instance (postgres://host:5432/brokkr_db)
├── public schema (default, can remain unused)
├── customer_a schema
│   ├── agents table
│   ├── stacks table
│   ├── generators table
│   └── deployment_objects table
├── customer_b schema
│   ├── agents table
│   ├── stacks table
│   └── ... (same schema structure)
└── customer_c schema
    └── ... (same schema structure)

Broker Instance A → BROKKR__DATABASE__SCHEMA=customer_a
Broker Instance B → BROKKR__DATABASE__SCHEMA=customer_b
Broker Instance C → BROKKR__DATABASE__SCHEMA=customer_c
```

### Key Components

**1. Database/ConnectionPool Struct Changes:**
```rust
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub schema: Option<String>,  // NEW: Optional schema name
}
```

**2. Schema-Aware Connection Method:**
```rust
pub async fn get_connection_with_schema(&self) -> Result<Connection> {
    let conn = self.pool.get()?;

    if let Some(ref schema) = self.schema {
        // Automatically set search path for tenant isolation
        conn.execute(&format!("SET search_path TO {}, public", schema))?;
    }

    Ok(conn)
}
```

**3. DAL Layer Updates:**
All DAL methods will use `get_connection_with_schema()` instead of direct pool access. This ensures every query executes in the correct schema context.

**4. Schema Provisioning:**
```rust
pub async fn setup_schema(&self, schema: &str) -> Result<()> {
    // 1. CREATE SCHEMA IF NOT EXISTS
    // 2. SET search_path
    // 3. Run all migrations within the schema
}
```

### Component Interaction Flow

```
Application Request
    ↓
DAL Method Call (e.g., agents().create())
    ↓
get_connection_with_schema()
    ↓
Connection Pool → Get Connection
    ↓
Execute: SET search_path TO {schema}, public
    ↓
Execute Query (automatically scoped to schema)
    ↓
Return Result
```

### Data Isolation Guarantee

PostgreSQL schemas provide strong isolation:
- Each schema has its own namespace for tables, views, indexes
- `SET search_path` ensures queries only see specified schema
- No application-level filtering required
- PostgreSQL enforces permissions at schema level
- Impossible to accidentally query wrong tenant's data

## Detailed Design **[REQUIRED]**

### Phase 1: Core Database Infrastructure (brokkr-broker crate)

**File: `crates/brokkr-broker/src/db.rs`**

1. Add `schema` field to `ConnectionPool`:
```rust
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub schema: Option<String>,
}
```

2. Add `create_shared_connection_pool_with_schema()` constructor:
```rust
pub fn create_shared_connection_pool_with_schema(
    base_url: &str,
    database_name: &str,
    max_size: u32,
    schema: Option<&str>,
) -> ConnectionPool
```

3. Implement `get_connection_with_schema()` method:
- Get connection from pool
- If schema is Some, execute `SET search_path TO {schema}, public`
- Return connection ready for use

4. Implement `setup_schema()` for provisioning:
- Create schema if not exists
- Run migrations in schema context

### Phase 2: DAL Layer Updates (brokkr-broker crate)

**File: `crates/brokkr-broker/src/dal/mod.rs` and sub-modules**

Update all DAL implementations to use schema-aware connections:

**Before:**
```rust
let mut conn = self.dal.pool.get()?;
diesel::insert_into(agents::table).execute(&mut conn)
```

**After:**
```rust
let mut conn = self.dal.pool.get_connection_with_schema()?;
diesel::insert_into(agents::table).execute(&mut conn)
```

Files to update:
- `dal/agents.rs` - AgentsDAL methods
- `dal/stacks.rs` - StacksDAL methods
- `dal/generators.rs` - GeneratorsDAL methods
- `dal/deployment_objects.rs` - DeploymentObjectsDAL methods
- `dal/agent_events.rs` - AgentEventsDAL methods
- All other DAL implementations

### Phase 3: Configuration Support (brokkr-utils crate)

**File: `crates/brokkr-utils/src/config.rs`**

Add schema field to Database configuration:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Database {
    pub url: String,
    pub schema: Option<String>,  // NEW
}
```

Environment variable: `BROKKR__DATABASE__SCHEMA`

Default: `None` (backward compatible - no schema, use public)

### Phase 4: Broker Initialization (brokkr-broker crate)

**File: `crates/brokkr-broker/src/main.rs` or broker startup**

Update broker initialization to pass schema to connection pool:

```rust
let pool = create_shared_connection_pool_with_schema(
    &config.database.url,
    &database_name,
    max_size,
    config.database.schema.as_deref(),
);
```

### Phase 5: Helm Chart Configuration

**File: `charts/brokkr-broker/values.yaml`**

Add schema configuration option:
```yaml
database:
  url: "postgres://user:pass@host:5432"
  schema: ""  # Optional: schema name for multi-tenancy
```

**File: `charts/brokkr-broker/templates/deployment.yaml`**

Add environment variable:
```yaml
env:
  - name: BROKKR__DATABASE__SCHEMA
    value: {{ .Values.database.schema | quote }}
```

### Phase 6: Schema Provisioning Tooling

Create schema management utilities:

**Option 1: CLI command in broker**
```bash
brokkr-broker schema create --name customer_a
brokkr-broker schema migrate --name customer_a
```

**Option 2: Kubernetes Job**
Create Helm chart template for schema provisioning job that runs before broker deployment

### Testing Strategy Integration

**Unit Tests:**
- Test `get_connection_with_schema()` with and without schema
- Test backward compatibility with schema=None
- Mock connection pool behavior

**Integration Tests:**
- Create two schemas in test database
- Insert data in schema A
- Query from schema B context, verify isolation
- Verify schema A data not visible from schema B
- Test migration runs correctly in each schema

**Performance Tests:**
- Measure query latency with/without schema switching
- Verify no connection pool degradation
- Benchmark under concurrent multi-schema load



## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

### Unit Testing
- **Strategy**: Test all new schema-aware methods in isolation with mocked connections
- **Coverage Target**: 100% coverage for new `get_connection_with_schema()`, `setup_schema()`, and configuration parsing
- **Tools**: Rust's built-in test framework, mockall for connection mocking

### Integration Testing
- **Strategy**: Multi-schema isolation tests using real PostgreSQL
- **Test Environment**: Docker-based PostgreSQL with multiple schemas
- **Test Cases**:
  - Create two schemas with identical table structure
  - Insert different data in each schema
  - Verify queries from schema A context only see schema A data
  - Verify queries from schema B context only see schema B data
  - Test migration runs correctly in each schema independently
  - Test backward compatibility (no schema = public schema)
- **Data Management**: Test data seeded per-schema, torn down after tests

### System Testing
- **Strategy**: Deploy multiple broker instances against same PostgreSQL with different schemas
- **Validation**:
  - Each broker can only see its own schema's data
  - Concurrent operations across brokers don't interfere
  - Schema provisioning works end-to-end
- **Performance Testing**:
  - Benchmark query performance with schema switching vs without
  - Load test with concurrent requests across multiple schemas
  - Verify connection pool efficiency under multi-tenant load

## Alternatives Considered **[REQUIRED]**

### Alternative 1: Shared Tables with tenant_id Column
**Approach:** Add `tenant_id` column to all tables and filter queries by tenant.

**Rejected because:**
- Requires significant schema changes to all existing tables
- Requires updating every single query to include tenant_id filtering
- Risk of accidentally querying across tenants (application-level bug)
- More complex Row-Level Security policies needed
- Performance impact from additional filtering on every query
- More difficult to audit and verify isolation

### Alternative 2: Separate Databases Per Tenant
**Approach:** Each tenant gets a completely separate PostgreSQL database.

**Rejected because:**
- Already possible with current architecture (just deploy with different database names)
- Higher operational overhead (more databases to backup, monitor, maintain)
- Less efficient resource utilization (can't share connection pools)
- More expensive at scale (database-level isolation has higher overhead)
- Schema-per-tenant provides same isolation with better operational efficiency

### Alternative 3: Dynamic Schema Routing (Option B)
**Approach:** Single broker instance that switches schemas based on authentication context at runtime.

**Deferred because:**
- Significantly more complex implementation
- Requires passing tenant context through entire request lifecycle
- Risk of schema-switching bugs leading to data leaks
- Option A (config-based) satisfies immediate requirements
- Can be added later if runtime multi-tenancy is needed
- Prefer simpler deployment model first

### Alternative 4: Using Different PostgreSQL Users Per Tenant
**Approach:** Each broker connects as different PostgreSQL user with access to specific schema only.

**Rejected because:**
- Requires managing multiple database credentials
- Connection pool can't be shared across users
- More complex credential rotation and management
- PostgreSQL user management overhead
- Schema-based approach is simpler and sufficient

### Selected Approach: Schema-Per-Tenant with Config
**Why this is best:**
- Proven in production (cloacina)
- Minimal code changes required
- PostgreSQL-enforced isolation
- Backward compatible
- Operationally efficient
- Simple deployment model
- Easy to provision and migrate

## Implementation Plan **[REQUIRED]**

### Phase 1: Database Infrastructure Foundation
**Deliverables:**
- Update `ConnectionPool` struct with `schema: Option<String>` field
- Implement `create_shared_connection_pool_with_schema()` constructor
- Implement `get_connection_with_schema()` method with SET search_path logic
- Implement `setup_schema()` for schema provisioning and migrations
- Add unit tests for all new methods

**Exit Criteria:**
- [ ] Schema field added to ConnectionPool
- [ ] New constructor accepts optional schema parameter
- [ ] get_connection_with_schema() correctly sets search_path
- [ ] setup_schema() creates schema and runs migrations
- [ ] All unit tests passing
- [ ] Backward compatibility verified (schema=None works)

### Phase 2: DAL Layer Migration
**Deliverables:**
- Update all DAL methods to use `get_connection_with_schema()`
- Verify all queries work with schema context
- Add integration tests for schema isolation

**Exit Criteria:**
- [ ] All DAL files updated (agents, stacks, generators, deployment_objects, agent_events)
- [ ] No direct pool.get() calls remaining in DAL layer
- [ ] Integration tests verify data isolation between schemas
- [ ] All existing tests still passing

### Phase 3: Configuration and Initialization
**Deliverables:**
- Add `schema: Option<String>` to Database config struct
- Support `BROKKR__DATABASE__SCHEMA` environment variable
- Update broker initialization to pass schema to connection pool
- Update configuration documentation

**Exit Criteria:**
- [ ] Configuration accepts schema parameter
- [ ] Environment variable properly parsed
- [ ] Broker initializes with schema configuration
- [ ] Default behavior (no schema) works correctly

### Phase 4: Helm Chart Integration
**Deliverables:**
- Add `database.schema` to values.yaml
- Add `BROKKR__DATABASE__SCHEMA` environment variable to deployment
- Create schema provisioning Job template (optional)
- Update Helm chart documentation

**Exit Criteria:**
- [ ] Helm values support schema configuration
- [ ] Environment variable properly templated
- [ ] Documentation updated with multi-tenant deployment examples
- [ ] Tested deployment with and without schema configuration

### Phase 5: Schema Management Tooling
**Deliverables:**
- Schema provisioning utility or documentation
- Migration guide for existing deployments
- Operational runbook for schema management

**Exit Criteria:**
- [ ] Documented process for creating new tenant schemas
- [ ] Migration guide from separate databases to schemas
- [ ] Backup and restore procedures for schema-based deployments
- [ ] Monitoring and alerting recommendations

### Phase 6: Testing and Validation
**Deliverables:**
- Comprehensive integration test suite
- Performance benchmarks
- Multi-schema deployment validation
- Documentation review

**Exit Criteria:**
- [ ] Integration tests verify complete data isolation
- [ ] Performance tests show acceptable overhead
- [ ] End-to-end deployment tested with multiple schemas
- [ ] All documentation complete and reviewed

### Phase 7: ADR and Knowledge Transfer
**Deliverables:**
- Architectural Decision Record documenting schema-per-tenant choice
- Knowledge sharing session or documentation
- Example deployment configurations

**Exit Criteria:**
- [x] ADR created and reviewed (BROKKR-A-0004: Schema-Per-Tenant Multi-Tenancy Architecture)
- [ ] Deployment examples added to documentation
- [ ] Team briefed on multi-tenant architecture

### Success Metrics
- Zero breaking changes to existing single-tenant deployments
- Data isolation verified by integration tests (100% isolation)
- Schema switching overhead < 5ms per query
- Documentation complete for all deployment patterns
- At least one production deployment using schema-based multi-tenancy
