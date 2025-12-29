---
id: phase-1-critical-security-fixes
level: initiative
title: "Phase 1: Critical Security Fixes"
short_code: "BROKKR-I-0005"
created_at: 2025-12-29T14:23:21.380380+00:00
updated_at: 2025-12-29T14:34:38.057301+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: phase-1-critical-security-fixes
---

# Phase 1: Critical Security Fixes

## Overview

Address critical security vulnerabilities that must be resolved before production deployment. These issues expose the system to data breaches, denial of service, and unauthorized access.

## Scope

### 1. Replace XOR Encryption with AES-256-GCM
- **Location:** `crates/brokkr-broker/src/utils/encryption.rs:76-133`
- **Issue:** Webhook URLs and auth headers use XOR-based obfuscation instead of cryptographic encryption
- **Action:** Implement proper AES-256-GCM using the `aes-gcm` crate
- **Migration:** Re-encrypt existing webhook data with new algorithm

### 2. Fix CORS Configuration
- **Location:** `crates/brokkr-broker/src/api/v1/mod.rs:37-41`
- **Issue:** Allows all origins, methods, and headers (CSRF vulnerability)
- **Action:** Configure allowed origins from environment/config, restrict methods to required set

### 3. Add PAK Hash Index for O(1) Authentication
- **Location:** `crates/brokkr-broker/src/api/v1/middleware.rs:137-170`
- **Issue:** Every request triggers full table scans of agents/generators tables
- **Action:** Add database index on pak_hash, create `get_by_pak_hash()` DAL method

### 4. Handle Connection Pool Errors Gracefully
- **Locations:** All DAL files using `.expect()` on pool.get()
- **Issue:** Connection pool exhaustion crashes the entire service
- **Action:** Return Result types, implement graceful degradation

### 5. Fix Generator Cascade Delete Bug
- **Location:** `crates/brokkr-models/migrations/02_generators/up.sql:28-30`
- **Issue:** Uses `WHERE id = NEW.id` instead of `WHERE generator_id = NEW.id`
- **Action:** Create migration to fix trigger, verify no orphaned records

## Success Criteria

- All webhook data encrypted with AES-256-GCM
- CORS restricted to configured origins only
- Authentication queries execute in O(1) time
- No panic on connection pool exhaustion
- Generator deletion properly cascades to child records

## Risk Assessment

- **Data Migration:** Existing encrypted webhook data needs re-encryption
- **Breaking Change:** CORS restrictions may affect existing UI deployments

---

## Technical Design

### 1. AES-256-GCM Encryption Implementation

**Dependencies:**
```toml
aes-gcm = "0.10"
```

**Architecture:**
```rust
pub struct AesEncryption {
    key: [u8; 32],  // 256-bit key
}

impl AesEncryption {
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Generate random 96-bit nonce
        // Encrypt with AES-256-GCM
        // Return nonce || ciphertext || tag
    }
    
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Extract nonce (first 12 bytes)
        // Decrypt and verify tag
        // Return plaintext
    }
}
```

**Migration Strategy:**
1. Add new `url_encrypted_v2` and `auth_header_encrypted_v2` columns
2. Create migration script to re-encrypt existing data
3. Update DAL to read from v2 columns
4. Drop v1 columns after verification

### 2. CORS Configuration

**Configuration Schema:**
```toml
[cors]
allowed_origins = ["https://ui.brokkr.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Authorization", "Content-Type"]
max_age = 3600
```

**Implementation:**
```rust
fn build_cors_layer(config: &CorsConfig) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(config.allowed_origins.iter().map(|o| o.parse().unwrap()))
        .allow_methods(config.allowed_methods.clone())
        .allow_headers(config.allowed_headers.clone())
        .max_age(Duration::from_secs(config.max_age))
}
```

### 3. PAK Authentication Optimization

**Database Changes:**
```sql
CREATE INDEX idx_agents_pak_hash ON agents(pak_hash) WHERE deleted_at IS NULL;
CREATE INDEX idx_generators_pak_hash ON generators(pak_hash) WHERE deleted_at IS NULL;
```

**DAL Methods:**
```rust
impl AgentsDal {
    pub fn get_by_pak_hash(&self, pak_hash: &str) -> Result<Option<Agent>, Error> {
        agents::table
            .filter(agents::pak_hash.eq(pak_hash))
            .filter(agents::deleted_at.is_null())
            .first(conn)
            .optional()
    }
}
```

**Middleware Update:**
- Remove full table scan loops
- Single indexed query per authentication type
- Early return on first match

### 4. Connection Pool Error Handling

**Pattern:**
```rust
// Before (panics)
let conn = self.pool.get().expect("Failed to get connection");

// After (returns Result)
let conn = self.pool.get().map_err(|e| {
    error!("Connection pool exhausted: {}", e);
    DalError::ConnectionPool(e)
})?;
```

**Error Type:**
```rust
pub enum DalError {
    ConnectionPool(r2d2::Error),
    Query(diesel::result::Error),
    NotFound,
}
```

### 5. Generator Cascade Delete Fix

**Migration:**
```sql
CREATE OR REPLACE FUNCTION handle_generator_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;  -- Fixed: generator_id
    
    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE stack_id IN (
        SELECT id FROM stacks WHERE generator_id = NEW.id
    ) AND deleted_at IS NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

## Dependencies Between Tasks

```
┌─────────────────────┐
│ Connection Pool Fix │ (independent, do first)
└─────────────────────┘

┌─────────────────────┐
│ Generator Cascade   │ (independent, do first)
└─────────────────────┘

┌─────────────────────┐     ┌─────────────────────┐
│ PAK Index Migration │────▶│ PAK Auth Middleware │
└─────────────────────┘     └─────────────────────┘

┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│ AES-GCM Encryption  │────▶│ Data Migration      │────▶│ Drop Old Columns    │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘

┌─────────────────────┐
│ CORS Configuration  │ (independent)
└─────────────────────┘
```

## Testing Strategy

- Unit tests for encryption round-trips with edge cases
- Integration tests for CORS with various origins
- Load tests for PAK authentication performance
- Migration tests on copy of production data Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

{Describe the context and background for this initiative}

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- {Primary objective 1}
- {Primary objective 2}

**Non-Goals:**
- {What this initiative will not address}

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design **[REQUIRED]**

{Technical approach and implementation details}

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

{Alternative approaches and why they were rejected}

## Implementation Plan **[REQUIRED]**

{Phases and timeline for execution}