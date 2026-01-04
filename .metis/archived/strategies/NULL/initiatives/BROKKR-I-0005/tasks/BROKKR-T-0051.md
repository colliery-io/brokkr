---
id: optimize-pak-authentication
level: task
title: "Optimize PAK authentication middleware with indexed lookups"
short_code: "BROKKR-T-0051"
created_at: 2025-12-29T14:27:12.919175+00:00
updated_at: 2025-12-29T15:04:59.879144+00:00
parent: BROKKR-I-0005
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0005
---

# Optimize PAK authentication middleware with indexed lookups

## Description

Replace the O(n) full table scan authentication with O(1) indexed lookups using new DAL methods.

**Depends on:** BROKKR-T-0050 (PAK hash indexes)

## Files to Modify

- `crates/brokkr-broker/src/dal/agents.rs` - Add get_by_pak_hash method
- `crates/brokkr-broker/src/dal/generators.rs` - Add get_by_pak_hash method
- `crates/brokkr-broker/src/api/v1/middleware.rs:137-170` - Use new methods

## Implementation

DAL methods:
```rust
// In agents.rs
pub fn get_by_pak_hash(&self, pak_hash: &str) -> Result<Option<Agent>, DalError> {
    let conn = &mut self.dal.pool.get()?;
    agents::table
        .filter(agents::pak_hash.eq(pak_hash))
        .filter(agents::deleted_at.is_null())
        .first(conn)
        .optional()
        .map_err(DalError::from)
}

// In generators.rs
pub fn get_by_pak_hash(&self, pak_hash: &str) -> Result<Option<Generator>, DalError> {
    let conn = &mut self.dal.pool.get()?;
    generators::table
        .filter(generators::pak_hash.eq(pak_hash))
        .filter(generators::deleted_at.is_null())
        .first(conn)
        .optional()
        .map_err(DalError::from)
}
```

Middleware update:
```rust
// Replace lines 137-170
// Compute PAK hash once
let pak_hash = pak::hash_pak(&pak);

// Try agent lookup first (most common)
if let Some(agent) = dal.agents().get_by_pak_hash(&pak_hash)? {
    return Ok(AuthPayload { agent: Some(agent.id), admin: false, generator: None });
}

// Try generator lookup
if let Some(generator) = dal.generators().get_by_pak_hash(&pak_hash)? {
    return Ok(AuthPayload { generator: Some(generator.id), admin: false, agent: None });
}

// Check admin PAK last
if pak::verify_admin_pak(&pak) {
    return Ok(AuthPayload { admin: true, agent: None, generator: None });
}
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Authentication completes in < 5ms (previously 100ms+)
- [ ] All existing auth tests pass
- [ ] Add benchmark test comparing old vs new performance
- [ ] Remove full table scan loops from middleware

## Dependencies

- **BROKKR-T-0048**: Uses `DalError` type for error handling
- **BROKKR-T-0050**: Requires PAK hash indexes to be created first

## Existing Code Reference

The PAK hashing function already exists at `crates/brokkr-broker/src/utils/pak.rs:114`:
```rust
pub fn generate_pak_hash(pak: String) -> String {
    let pak = PrefixedApiKey::from_string(pak.as_str()).expect("Failed to parse PAK");
    let controller = create_pak_controller(None).expect("Failed to create PAK controller");
    controller.long_token_hashed(&pak)
}
```

Use this to compute the hash for indexed lookup.

## Notes

- The current middleware at `api/v1/middleware.rs:137-170` iterates over ALL agents/generators
- New approach: compute hash once, do single indexed query per entity type
- Admin PAK check should remain last (least common case)