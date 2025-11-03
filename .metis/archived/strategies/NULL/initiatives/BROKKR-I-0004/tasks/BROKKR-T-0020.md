---
id: implement-database-infrastructure
level: task
title: "Implement database infrastructure foundation for schema-per-tenant"
short_code: "BROKKR-T-0020"
created_at: 2025-10-22T17:41:21.232834+00:00
updated_at: 2025-10-29T16:20:10.289069+00:00
parent: BROKKR-I-0004
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0004
---

# Implement database infrastructure foundation for schema-per-tenant

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0004]]

## Objective **[REQUIRED]**

Implement the core database infrastructure to support schema-per-tenant multi-tenancy. This includes adding schema awareness to the ConnectionPool, implementing automatic search_path configuration, and creating schema provisioning utilities.

**Phase:** Phase 1 - Database Infrastructure Foundation

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Schema field added to ConnectionPool struct in `crates/brokkr-broker/src/db.rs`
- [ ] Existing `create_shared_connection_pool()` modified to accept optional schema parameter
- [ ] `get_connection()` method (or pool.get() wrapper) automatically sets search_path when schema is configured
- [ ] `setup_schema()` function creates schema and runs migrations in schema context
- [ ] Schema name validation to prevent SQL injection
- [ ] All unit tests passing for schema functionality
- [ ] Backward compatibility verified (schema=None works with existing behavior)

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**File: `crates/brokkr-broker/src/db.rs`**

**Core Principle:** Extend existing methods directly - no parallel versions or backward compatibility concerns needed since this is internal broker code.

1. Add `schema` field to `ConnectionPool`:
```rust
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub schema: Option<String>,  // Store optional schema name
}
```

2. Modify existing `create_shared_connection_pool()` to accept schema:
```rust
pub fn create_shared_connection_pool(
    base_url: &str,
    database_name: &str,
    max_size: u32,
    schema: Option<&str>,  // NEW PARAMETER
) -> ConnectionPool {
    // Build URL and create pool as before
    ConnectionPool {
        pool,
        schema: schema.map(|s| s.to_string()),
    }
}
```

3. Add `get_connection()` method on ConnectionPool that wraps `pool.get()` and automatically sets search_path:
```rust
impl ConnectionPool {
    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        let mut conn = self.pool.get()?;
        if let Some(ref schema) = self.schema {
            // Validate schema name to prevent SQL injection
            validate_schema_name(schema).expect("Invalid schema name");
            // Set search_path for this connection
            diesel::sql_query(format!("SET search_path TO {}, public", schema))
                .execute(&mut conn)
                .expect("Failed to set search_path");
        }
        Ok(conn)
    }
}
```

This approach is cleanest because:
- Single point of schema handling
- Transparent to DAL layer (just change `pool.get()` to `pool.get_connection()`)
- No need for per-query schema management

4. Add schema name validation:
```rust
fn validate_schema_name(schema: &str) -> Result<(), Error> {
    // Schema names must start with letter, contain only alphanumeric and underscore
    let re = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(schema) {
        return Err(Error::InvalidSchemaName);
    }
    Ok(())
}
```

5. Add `setup_schema()` for provisioning:
```rust
pub fn setup_schema(&self, schema: &str) -> Result<(), Error> {
    validate_schema_name(schema)?;
    let mut conn = self.pool.get()?;

    // Create schema
    diesel::sql_query(format!("CREATE SCHEMA IF NOT EXISTS {}", schema))
        .execute(&mut conn)?;

    // Run migrations in schema context
    diesel::sql_query(format!("SET search_path TO {}, public", schema))
        .execute(&mut conn)?;

    // Run migrations using embedded_migrations or diesel_migrations
    // (migration logic here)

    Ok(())
}
```

### Dependencies
- None (first phase, foundational work)

### Risk Considerations
- SQL injection in schema name - validate schema names match `^[a-zA-Z][a-zA-Z0-9_]*$`
- Connection pool behavior must remain unchanged except for search_path
- Ensure search_path is set on every connection acquisition

## Status Updates **[REQUIRED]**

### 2025-10-22: Task Re-scoped for Simplicity

**Approach Simplified:**
- Rather than creating parallel "with_schema" methods, we're modifying existing method signatures
- This is acceptable since all these methods are internal to the broker
- Simpler API surface: `create_shared_connection_pool()` takes optional schema param
- Simpler usage: `pool.get_connection()` automatically handles search_path

**Key Changes from Original Plan:**
- `create_shared_connection_pool()` modified to accept `schema: Option<&str>` parameter
- `get_connection()` method added to ConnectionPool (not `get_connection_with_schema`)
- No duplicate API methods needed

**Next Steps:**
- Implement the simplified approach
- Update all call sites to pass `None` for schema (backward compatibility)
- Add integration tests for schema isolation