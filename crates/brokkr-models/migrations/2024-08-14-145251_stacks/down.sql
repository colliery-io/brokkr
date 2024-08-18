-- Drop the view
DROP VIEW IF EXISTS active_stacks;

-- Drop the triggers
DROP TRIGGER IF EXISTS soft_delete_stacks ON stacks;
DROP TRIGGER IF EXISTS update_stack_timestamp ON stacks;

-- Drop the indexes
DROP INDEX IF EXISTS idx_stack_name;
DROP INDEX IF EXISTS idx_stack_id;

-- Drop the table
DROP TABLE IF EXISTS stacks;