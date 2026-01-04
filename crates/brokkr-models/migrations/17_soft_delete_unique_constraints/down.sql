-- Rollback: Restore original unique constraints (includes soft-deleted records)
-- This migration first removes soft-deleted duplicates to prevent constraint violations

-- =============================================================================
-- Clean up soft-deleted duplicates before restoring constraints
-- For each table, we keep the active record (or oldest) per unique key
-- Using CTEs for better PostgreSQL compatibility
-- =============================================================================

-- Clean up duplicate agents (keep active or oldest per name+cluster)
WITH duplicates AS (
    SELECT id, ROW_NUMBER() OVER (
        PARTITION BY name, cluster_name
        ORDER BY deleted_at ASC NULLS FIRST, created_at ASC
    ) as rn
    FROM agents
)
DELETE FROM agents
WHERE id IN (SELECT id FROM duplicates WHERE rn > 1);

-- Clean up duplicate stacks (keep active or oldest per name)
WITH duplicates AS (
    SELECT id, ROW_NUMBER() OVER (
        PARTITION BY name
        ORDER BY deleted_at ASC NULLS FIRST, created_at ASC
    ) as rn
    FROM stacks
)
DELETE FROM stacks
WHERE id IN (SELECT id FROM duplicates WHERE rn > 1);

-- Clean up duplicate stack_templates (keep active or oldest per generator_id+name+version)
WITH duplicates AS (
    SELECT id, ROW_NUMBER() OVER (
        PARTITION BY generator_id, name, version
        ORDER BY deleted_at ASC NULLS FIRST, created_at ASC
    ) as rn
    FROM stack_templates
)
DELETE FROM stack_templates
WHERE id IN (SELECT id FROM duplicates WHERE rn > 1);

-- Clean up duplicate generators (keep active or oldest per name)
WITH duplicates AS (
    SELECT id, ROW_NUMBER() OVER (
        PARTITION BY name
        ORDER BY deleted_at ASC NULLS FIRST, created_at ASC
    ) as rn
    FROM generators
)
DELETE FROM generators
WHERE id IN (SELECT id FROM duplicates WHERE rn > 1);

-- =============================================================================
-- Generators: Remove unique constraint on name
-- =============================================================================
DROP INDEX IF EXISTS unique_generator_name;

-- =============================================================================
-- Stack Templates: Restore table constraint
-- =============================================================================
DROP INDEX IF EXISTS unique_template_version;

ALTER TABLE stack_templates
ADD CONSTRAINT unique_template_version
UNIQUE NULLS NOT DISTINCT (generator_id, name, version);

-- =============================================================================
-- Stacks: Restore table constraint
-- =============================================================================
DROP INDEX IF EXISTS unique_stack_name;

ALTER TABLE stacks
ADD CONSTRAINT unique_stack_name UNIQUE (name);

-- =============================================================================
-- Agents: Restore table constraint
-- =============================================================================
DROP INDEX IF EXISTS unique_agent_cluster;

ALTER TABLE agents
ADD CONSTRAINT unique_agent_cluster UNIQUE (name, cluster_name);
