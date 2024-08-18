-- Drop triggers
DROP TRIGGER IF EXISTS trigger_hard_delete_deployment_objects ON stacks;
DROP TRIGGER IF EXISTS prevent_deployment_object_delete ON deployment_objects;
DROP TRIGGER IF EXISTS prevent_deployment_object_update ON deployment_objects;
DROP TRIGGER IF EXISTS trigger_handle_stack_soft_delete ON stacks;

-- Drop functions
DROP FUNCTION IF EXISTS hard_delete_deployment_objects_on_stack_delete();
DROP FUNCTION IF EXISTS prevent_deployment_object_changes();
DROP FUNCTION IF EXISTS handle_stack_soft_delete();

-- Drop indexes
DROP INDEX IF EXISTS idx_deployment_objects_is_deletion_marker;
DROP INDEX IF EXISTS idx_deployment_objects_deleted_at;
DROP INDEX IF EXISTS idx_deployment_objects_yaml_checksum;
DROP INDEX IF EXISTS idx_deployment_objects_agent_annotations;
DROP INDEX IF EXISTS idx_deployment_objects_agent_labels;
DROP INDEX IF EXISTS idx_deployment_objects_agent_target;
DROP INDEX IF EXISTS idx_deployment_objects_stack_id;

-- Drop table
DROP TABLE IF EXISTS deployment_objects;