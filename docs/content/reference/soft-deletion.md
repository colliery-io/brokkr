---
title: "Soft Deletion Pattern"
weight: 10
---

# Soft Deletion Pattern

Brokkr uses soft deletion as the default deletion strategy for primary entities. Rather than removing records from the database, soft deletion marks records with a `deleted_at` timestamp while preserving all data. This design enables audit compliance, accidental deletion recovery, and maintains referential integrity across related resources.

## How Soft Deletion Works

When you delete a resource through the Brokkr API, the system sets the `deleted_at` timestamp to the current time rather than executing a SQL DELETE. This approach has several consequences:

- The record remains in the database with all its data intact
- Standard API queries filter out records where `deleted_at IS NOT NULL`
- Related resources may be cascade soft-deleted depending on the entity type
- The record can potentially be recovered by clearing the `deleted_at` field
- Unique constraints are scoped to only active (non-deleted) records

## Entities Supporting Soft Deletion

| Entity | Cascade Behavior | API Endpoint |
|--------|------------------|--------------|
| Agents | Soft deletes agent; events preserved | `DELETE /api/v1/agents/{id}` |
| Stacks | Cascades to deployment objects; creates deletion marker | `DELETE /api/v1/stacks/{id}` |
| Generators | Cascades to stacks and deployment objects | `DELETE /api/v1/generators/{id}` |
| Templates | Soft deletes template only | `DELETE /api/v1/templates/{id}` |
| Deployment Objects | Soft deletes object only | Generally not exposed directly |

## Cascade Behavior

### Stack Deletion

When a stack is soft-deleted, the system triggers several cascading operations through database triggers:

1. All deployment objects belonging to the stack are soft-deleted
2. A special deletion marker deployment object is created with `is_deletion_marker: true`
3. The deletion marker notifies agents to remove the resources from their clusters

```sql
-- Simplified trigger logic
UPDATE deployment_objects
SET deleted_at = NEW.deleted_at
WHERE stack_id = NEW.id AND deleted_at IS NULL;

INSERT INTO deployment_objects (stack_id, yaml_content, is_deletion_marker)
VALUES (NEW.id, '', TRUE);
```

### Generator Deletion

When a generator is soft-deleted, the cascade propagates to all resources created by that generator:

1. All stacks owned by the generator are soft-deleted
2. All deployment objects in those stacks are soft-deleted
3. Each stack's soft deletion also creates deletion markers

This ensures that deleting a generator properly cleans up resources across all clusters that received deployments from that generator.

### Agent Deletion

Agent soft deletion is simpler—only the agent record itself is marked deleted. Agent events are preserved for audit purposes, maintaining the historical record of what the agent did before deletion.

## Unique Constraints

Brokkr uses partial unique indexes that exclude soft-deleted records. This design allows you to reuse names after deleting resources:

```sql
-- Example: Stack name uniqueness
CREATE UNIQUE INDEX unique_stack_name
ON stacks (name)
WHERE deleted_at IS NULL;
```

This means:
- You cannot create two active stacks with the same name
- After soft-deleting a stack, you can create a new stack with the same name
- The original soft-deleted stack remains in the database with its historical data

Entities with partial unique constraints:

| Entity | Unique Fields |
|--------|---------------|
| Agents | (name, cluster_name) |
| Stacks | name |
| Generators | name |
| Templates | (generator_id, name, version) |

## Querying Deleted Records

Standard API endpoints automatically filter out soft-deleted records. However, some DAL methods allow querying including deleted records for administrative purposes:

```rust
// Standard query - excludes deleted
dal.stacks().get(vec![stack_id])

// Include deleted records
dal.stacks().get_including_deleted(stack_id)

// List all including deleted
dal.stacks().list_all()
```

These methods are primarily used internally for:
- Audit trail queries
- Database maintenance
- Recovery operations

## Hard Deletion

Hard deletion (permanent removal from database) is available but used sparingly. When a hard delete occurs, additional cleanup is performed through BEFORE DELETE triggers:

### Stack Hard Delete

```sql
-- Cleanup performed before hard delete
DELETE FROM agent_targets WHERE stack_id = OLD.id;
DELETE FROM agent_events WHERE deployment_object_id IN (
    SELECT id FROM deployment_objects WHERE stack_id = OLD.id
);
DELETE FROM deployment_objects WHERE stack_id = OLD.id;
```

Hard deletion is typically reserved for:
- Data retention compliance after the retention period
- Cleaning up test data
- Emergency data removal scenarios

## Recovery Considerations

While soft-deleted records remain in the database, Brokkr does not currently expose a recovery API. Manual recovery requires database access:

```sql
-- Example: Recover a soft-deleted stack
UPDATE stacks SET deleted_at = NULL WHERE id = 'stack-uuid';

-- Note: Cascade-deleted children must also be recovered
UPDATE deployment_objects SET deleted_at = NULL WHERE stack_id = 'stack-uuid';
```

Recovery is complicated by several factors:
- Cascade-deleted children must be individually recovered
- Deletion markers created during soft deletion remain
- Unique constraint conflicts may arise if a new resource was created with the same name

## Database Schema Details

### deleted_at Column

All soft-deletable entities include:

```sql
deleted_at TIMESTAMP WITH TIME ZONE
```

This column is:
- `NULL` for active records
- Set to the deletion timestamp for deleted records
- Indexed for query performance: `CREATE INDEX idx_entity_deleted_at ON entity(deleted_at);`

### Trigger Functions

Soft deletion cascades are implemented as PostgreSQL trigger functions:

| Trigger | Table | Event | Function |
|---------|-------|-------|----------|
| `trigger_handle_stack_soft_delete` | stacks | AFTER UPDATE of deleted_at | `handle_stack_soft_delete()` |
| `cascade_soft_delete_generators` | generators | AFTER UPDATE | `cascade_soft_delete_generators()` |
| `trigger_stack_hard_delete` | stacks | BEFORE DELETE | `handle_stack_hard_delete()` |

## Performance Implications

Soft deletion has modest performance implications:

**Query overhead**: Every query must include `WHERE deleted_at IS NULL`. This is mitigated by indexing the `deleted_at` column.

**Table growth**: Soft-deleted records accumulate over time. For high-churn environments, periodic hard deletion of old soft-deleted records may be necessary.

**Index size**: Partial unique indexes only include active records, keeping index size proportional to active data rather than total data.

## Best Practices

**Prefer soft deletion**: Use the standard DELETE endpoints which perform soft deletion. This preserves audit trails and enables recovery.

**Monitor table growth**: Track the ratio of soft-deleted to active records. Consider periodic cleanup of very old soft-deleted records.

**Test recovery procedures**: If recovery is important for your use case, establish and test recovery procedures before you need them.

**Understand cascade effects**: Before deleting generators or stacks, understand the cascade implications for dependent resources.

## Related Documentation

- [Data Model Design](/explanation/data-model) - Entity relationships and design philosophy
- [Audit Logs](/reference/audit-logs) - Tracking actions for compliance
- [Managing Stacks](/how-to/managing-stacks) - Stack lifecycle including deletion
