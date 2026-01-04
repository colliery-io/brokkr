-- Migration: Fix unique constraints to exclude soft-deleted records
-- BROKKR-T-0093: Unique constraints should use partial indexes with WHERE deleted_at IS NULL
-- This allows recreating resources with the same name after soft deletion

-- =============================================================================
-- Agents: unique_agent_cluster (name, cluster_name)
-- =============================================================================
-- Drop the existing table constraint
ALTER TABLE agents DROP CONSTRAINT IF EXISTS unique_agent_cluster;

-- Create partial unique index that excludes soft-deleted records
CREATE UNIQUE INDEX unique_agent_cluster
ON agents (name, cluster_name)
WHERE deleted_at IS NULL;

-- =============================================================================
-- Stacks: unique_stack_name (name)
-- =============================================================================
-- Drop the existing table constraint
ALTER TABLE stacks DROP CONSTRAINT IF EXISTS unique_stack_name;

-- Create partial unique index that excludes soft-deleted records
CREATE UNIQUE INDEX unique_stack_name
ON stacks (name)
WHERE deleted_at IS NULL;

-- =============================================================================
-- Stack Templates: unique_template_version (generator_id, name, version)
-- =============================================================================
-- Drop the existing table constraint
ALTER TABLE stack_templates DROP CONSTRAINT IF EXISTS unique_template_version;

-- Create partial unique index that excludes soft-deleted records
-- Note: NULLS NOT DISTINCT behavior is preserved for generator_id
CREATE UNIQUE INDEX unique_template_version
ON stack_templates (generator_id, name, version)
NULLS NOT DISTINCT
WHERE deleted_at IS NULL;

-- =============================================================================
-- Generators: Add missing unique constraint on name
-- =============================================================================
-- Generators previously had no uniqueness constraint on name, allowing duplicates
-- Add partial unique index to prevent duplicate names among active generators
CREATE UNIQUE INDEX unique_generator_name
ON generators (name)
WHERE deleted_at IS NULL;
