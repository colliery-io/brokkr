---
id: add-pak-hash-database-indexes-for
level: task
title: "Add PAK hash database indexes for agents and generators"
short_code: "BROKKR-T-0050"
created_at: 2025-12-29T14:27:12.844157+00:00
updated_at: 2025-12-29T14:59:54.406414+00:00
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

# Add PAK hash database indexes for agents and generators

## Description

Create database indexes on the `pak_hash` columns to enable O(1) lookups instead of full table scans during authentication.

## Files to Modify

- `crates/brokkr-models/migrations/` - Create new migration (e.g., `15_pak_hash_indexes/`)

## Implementation

Create migration `up.sql`:
```sql
-- Index for agent PAK authentication
CREATE INDEX idx_agents_pak_hash ON agents(pak_hash) 
    WHERE deleted_at IS NULL AND pak_hash IS NOT NULL;

-- Index for generator PAK authentication
CREATE INDEX idx_generators_pak_hash ON generators(pak_hash) 
    WHERE deleted_at IS NULL AND pak_hash IS NOT NULL;
```

Create migration `down.sql`:
```sql
DROP INDEX IF EXISTS idx_agents_pak_hash;
DROP INDEX IF EXISTS idx_generators_pak_hash;
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration creates both indexes
- [ ] Indexes are partial (WHERE deleted_at IS NULL)
- [ ] EXPLAIN ANALYZE shows index usage for PAK lookups
- [ ] Migration runs in < 1 second on tables with 10k records

## Verification Query

After migration, verify index usage:
```sql
EXPLAIN ANALYZE 
SELECT * FROM agents 
WHERE pak_hash = 'some_hash_value' AND deleted_at IS NULL;

-- Should show: Index Scan using idx_agents_pak_hash
-- NOT: Seq Scan on agents
```

## Dependencies

- None (independent migration)
- **Required by:** BROKKR-T-0051 (middleware optimization)

## Notes

- Partial indexes exclude soft-deleted records (saves space, faster queries)
- Index creation is non-blocking in PostgreSQL by default
- Migration number should follow existing sequence