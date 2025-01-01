-- Drop new triggers
DROP TRIGGER IF EXISTS trigger_cascade_delete_stack_annotations ON stacks;
DROP TRIGGER IF EXISTS trigger_cascade_delete_stack_labels ON stacks;

-- Drop new functions
DROP FUNCTION IF EXISTS cascade_delete_stack_annotations();
DROP FUNCTION IF EXISTS cascade_delete_stack_labels();

-- Existing drops (no changes)
DROP TRIGGER IF EXISTS trigger_handle_stack_soft_delete ON stacks;
DROP TRIGGER IF EXISTS trigger_stack_hard_delete ON stacks;
DROP TRIGGER IF EXISTS update_stack_timestamp ON stacks;
DROP TRIGGER IF EXISTS update_agent_target_timestamp ON agent_targets;

DROP FUNCTION IF EXISTS handle_stack_soft_delete();
DROP FUNCTION IF EXISTS handle_stack_hard_delete();

DROP INDEX IF EXISTS idx_stack_name;
DROP INDEX IF EXISTS idx_stack_id;
DROP INDEX IF EXISTS idx_agent_targets_deleted_at;
DROP INDEX IF EXISTS idx_agent_targets_stack_id;
DROP INDEX IF EXISTS idx_agent_targets_agent_id;

DROP TABLE IF EXISTS stacks CASCADE;
DROP TABLE IF EXISTS agent_targets;
DROP TABLE IF EXISTS stack_labels;
DROP TABLE IF EXISTS stack_annotations;
