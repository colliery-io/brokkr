---
id: add-caching-layer-to-auth
level: task
title: "Add caching layer to auth middleware to reduce per-request database queries"
short_code: "BROKKR-T-0128"
created_at: 2026-03-14T01:51:48.466167+00:00
updated_at: 2026-03-14T01:54:28.736668+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


exit_criteria_met: false
initiative_id: NULL
---

# Add caching layer to auth middleware to reduce per-request database queries

## Objective

Introduce a TTL-based in-memory cache for PAK-to-identity lookups in the auth middleware (`crates/brokkr-broker/src/api/v1/middleware.rs`) to eliminate redundant database queries on every HTTP request.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [ ] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: The `verify_pak()` function in `middleware.rs` (lines 108-173) executes 2-3 database queries on **every authenticated request** with zero caching:
  1. `admin_role` table SELECT for PAK hash (lines 116-123)
  2. `agents` table SELECT by PAK hash via `dal.agents().get_by_pak_hash()` (line 138, impl in `dal/agents.rs:510-517`)
  3. `generators` table SELECT by PAK hash via `dal.generators().get_by_pak_hash()` (line 155, impl in `dal/generators.rs:249-259`)

  The only cached component is the PAK controller singleton (`utils/pak.rs:22`, via `OnceCell`), but auth *results* are never cached. No `moka`, `lru`, `dashmap`, or any TTL cache crate is present in the dependency tree.

- **Benefits of Fixing**:
  - Reduces database load proportional to request volume (agents poll frequently)
  - Lower p99 latency on auth-gated endpoints
  - PAK hashes rarely change — cache hit rates should be very high
  - Reduces connection pool pressure under load

- **Risk Assessment**: Low risk if left as-is for small deployments (queries are indexed, O(1)). Becomes a meaningful bottleneck at scale with many agents polling concurrently.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Auth results (PAK hash → `AuthPayload`) are cached in memory with a configurable TTL
- [ ] Cache is invalidated when PAKs are rotated or roles change (admin CRUD, agent/generator deletion)
- [ ] No measurable regression in auth correctness (same 401/403 behavior)
- [ ] Cache TTL is configurable via broker config (sensible default, e.g. 60s)
- [ ] Existing unit/integration tests continue to pass

## Implementation Notes

### Technical Approach
- Add `moka` (or similar async-compatible TTL cache) as a dependency
- Create a `HashMap<String, AuthPayload>`-style cache keyed on PAK hash, stored in app state alongside the DAL
- On cache miss, fall through to existing DB lookup chain and populate cache
- Invalidate relevant cache entries on PAK rotation endpoints and admin/agent/generator mutations
- Consider a short TTL (30-60s) to balance freshness vs. query reduction

### Key Files
- `crates/brokkr-broker/src/api/v1/middleware.rs` — main auth middleware, `verify_pak()`
- `crates/brokkr-broker/src/utils/pak.rs` — PAK hashing/verification
- `crates/brokkr-broker/src/dal/agents.rs` — agent PAK lookup
- `crates/brokkr-broker/src/dal/generators.rs` — generator PAK lookup
- `crates/brokkr-broker/src/api/v1/admin.rs` — admin PAK mutation endpoints (need cache invalidation)

### Risk Considerations
- Stale cache could briefly allow a revoked PAK to authenticate — mitigate with short TTL and explicit invalidation on revocation
- Cache must be `Send + Sync` for use across Tokio tasks

## Status Updates

### 2026-03-14 — Implementation complete
- Added `moka` (v0.12, sync feature) to workspace and brokkr-broker dependencies
- Added `auth_cache_ttl_seconds` config field to `Broker` struct (`config.rs`) with default 60s in `default.toml`
- Extended `DAL` with `auth_cache: Option<Cache<String, AuthPayload>>` and `new_with_auth_cache()` constructor
- Added `invalidate_auth_cache()` and `invalidate_all_auth_cache()` methods to DAL
- Updated broker startup (`cli/commands.rs`) to use `new_with_auth_cache` with configurable TTL
- Updated `verify_pak()` in middleware to check cache on entry, populate on miss
- Added targeted cache invalidation to:
  - `rotate_agent_pak` (agents.rs) — invalidates old PAK hash after rotation
  - `rotate_generator_pak` (generators.rs) — invalidates old PAK hash after rotation
  - `delete_agent` (agents.rs) — invalidates old PAK hash before soft-delete
  - `delete_generator` (generators.rs) — invalidates old PAK hash before soft-delete
- All 84 brokkr-broker unit tests pass
- All 24 brokkr-utils unit tests pass
- Cache can be disabled by setting `auth_cache_ttl_seconds = 0`
