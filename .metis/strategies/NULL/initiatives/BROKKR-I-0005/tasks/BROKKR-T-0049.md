---
id: fix-generator-cascade-soft-delete
level: task
title: "Fix generator cascade soft-delete trigger bug"
short_code: "BROKKR-T-0049"
created_at: 2025-12-29T14:27:12.772035+00:00
updated_at: 2025-12-29T14:59:54.331579+00:00
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

# Fix generator cascade soft-delete trigger bug

## Description

The `handle_generator_soft_delete()` trigger incorrectly uses `WHERE id = NEW.id` instead of `WHERE generator_id = NEW.id`, preventing proper cascade of soft deletes to child stacks.

## Files to Modify

- `crates/brokkr-models/migrations/` - Create new migration (e.g., `14_fix_generator_cascade/`)

## Implementation

Create migration `up.sql`:
```sql
-- Fix the generator soft-delete cascade trigger
CREATE OR REPLACE FUNCTION handle_generator_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Cascade soft delete to stacks owned by this generator
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;
    
    -- Cascade soft delete to deployment objects in those stacks
    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE stack_id IN (
        SELECT id FROM stacks WHERE generator_id = NEW.id
    ) AND deleted_at IS NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

Create migration `down.sql`:
```sql
-- Revert to original (buggy) trigger - for rollback only
CREATE OR REPLACE FUNCTION handle_generator_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE id = NEW.id AND deleted_at IS NULL;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration created and applies cleanly
- [ ] Soft-deleting a generator cascades to its stacks
- [ ] Soft-deleting a generator cascades to deployment objects in those stacks
- [ ] Add integration test verifying cascade behavior
- [ ] Verify no orphaned records exist in current data

## Original Bug Location

The buggy trigger is defined at:
`crates/brokkr-models/migrations/02_generators/up.sql:28-30`

```sql
-- BUGGY CODE (current):
UPDATE stacks
SET deleted_at = NEW.deleted_at
WHERE id = NEW.id AND deleted_at IS NULL;  -- BUG: should be generator_id = NEW.id
```

## Verification Query

Run before migration to check for orphaned records:
```sql
-- Find stacks whose generator is soft-deleted but stack is not
SELECT s.id, s.name, g.id as generator_id, g.deleted_at as generator_deleted
FROM stacks s
JOIN generators g ON s.generator_id = g.id
WHERE g.deleted_at IS NOT NULL AND s.deleted_at IS NULL;
```

## Dependencies

- None (independent task)

## Notes

- Migration number should follow existing sequence (check latest in `migrations/` folder)
- The down migration restores buggy behavior intentionally (for rollback only)