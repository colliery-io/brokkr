DROP TRIGGER IF EXISTS trigger_handle_stack_soft_delete ON stacks;
DROP FUNCTION IF EXISTS handle_stack_soft_delete();

DROP TRIGGER IF EXISTS update_stack_timestamp ON stacks;

DROP INDEX IF EXISTS idx_stack_agent_target;
DROP INDEX IF EXISTS idx_stack_annotations;
DROP INDEX IF EXISTS idx_stack_labels;
DROP INDEX IF EXISTS idx_stack_name;
DROP INDEX IF EXISTS idx_stack_id;

DROP TABLE IF EXISTS stacks;
DROP TABLE IF EXISTS agent_targets;
