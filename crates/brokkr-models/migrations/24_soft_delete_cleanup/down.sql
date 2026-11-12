-- Rollback: the soft deletes stop cleaning the child rows. The rows that the
-- triggers and the migration removed do not come back.

DROP TRIGGER IF EXISTS trigger_cleanup_agent_soft_delete ON agents;
DROP FUNCTION IF EXISTS cleanup_agent_soft_delete();

DROP TRIGGER IF EXISTS trigger_cleanup_stack_soft_delete ON stacks;
DROP FUNCTION IF EXISTS cleanup_stack_soft_delete();
