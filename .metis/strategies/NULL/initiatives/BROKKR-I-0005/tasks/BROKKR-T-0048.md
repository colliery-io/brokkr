---
id: replace-connection-pool-panics
level: task
title: "Replace connection pool panics with Result error handling"
short_code: "BROKKR-T-0048"
created_at: 2025-12-29T14:27:12.704137+00:00
updated_at: 2025-12-29T14:59:54.258632+00:00
parent: BROKKR-I-0005
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0005
---

# Replace connection pool panics with Result error handling

## Description

All DAL methods currently use `.expect()` when acquiring database connections, causing the entire service to crash if the connection pool is exhausted. Replace with proper Result-based error handling.

## Files to Modify

- `crates/brokkr-broker/src/dal/mod.rs` - Add DalError type
- `crates/brokkr-broker/src/dal/agents.rs` - Replace expects
- `crates/brokkr-broker/src/dal/stacks.rs` - Replace expects
- `crates/brokkr-broker/src/dal/generators.rs` - Replace expects
- `crates/brokkr-broker/src/dal/webhook_subscriptions.rs` - Replace expects
- `crates/brokkr-broker/src/dal/webhook_deliveries.rs` - Replace expects
- `crates/brokkr-broker/src/dal/work_orders.rs` - Replace expects
- `crates/brokkr-broker/src/dal/deployment_objects.rs` - Replace expects
- All other DAL files

## Implementation

```rust
// In dal/mod.rs
#[derive(Debug)]
pub enum DalError {
    ConnectionPool(r2d2::Error),
    Query(diesel::result::Error),
    NotFound,
}

impl From<r2d2::Error> for DalError {
    fn from(e: r2d2::Error) -> Self {
        DalError::ConnectionPool(e)
    }
}

// In each DAL method
pub fn get(&self, id: Uuid) -> Result<Option<Agent>, DalError> {
    let conn = &mut self.dal.pool.get()?;  // Propagates error
    // ...
}
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] No `.expect()` calls on pool.get() in any DAL method
- [ ] DalError enum defined with ConnectionPool variant
- [ ] API handlers return 503 Service Unavailable on pool exhaustion
- [ ] Existing unit tests pass
- [ ] Add test for connection pool exhaustion handling

## API Error Mapping

Add error conversion in API handlers:

```rust
// In api/v1/mod.rs or a shared error module
impl IntoResponse for DalError {
    fn into_response(self) -> Response {
        match self {
            DalError::ConnectionPool(e) => {
                error!("Database connection pool exhausted: {}", e);
                (StatusCode::SERVICE_UNAVAILABLE, 
                 Json(json!({"error": "Service temporarily unavailable"}))).into_response()
            }
            DalError::Query(e) => {
                error!("Database query error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 Json(json!({"error": "Internal server error"}))).into_response()
            }
            DalError::NotFound => {
                (StatusCode::NOT_FOUND,
                 Json(json!({"error": "Resource not found"}))).into_response()
            }
        }
    }
}
```

## Dependencies

- None (this task is foundational, other tasks depend on it)

## Notes

- This change affects the return type of all DAL methods
- Existing tests using `.unwrap()` on DAL calls will need updating
- Consider using `?` operator throughout for cleaner error propagation