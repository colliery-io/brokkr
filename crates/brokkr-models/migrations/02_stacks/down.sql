-- Drop triggers first
DROP TRIGGER IF EXISTS trigger_handle_stack_soft_delete ON stacks;
DROP TRIGGER IF EXISTS update_stack_timestamp ON stacks;

-- Drop functions
DROP FUNCTION IF EXISTS handle_stack_soft_delete();

-- Drop indexes
DROP INDEX IF EXISTS idx_agent_targets_agent_id;
DROP INDEX IF EXISTS idx_agent_targets_stack_id;
DROP INDEX IF EXISTS idx_stack_name;
DROP INDEX IF EXISTS idx_stack_id;

-- Drop tables
DROP TABLE IF EXISTS agent_targets;
DROP TABLE IF EXISTS stacks;