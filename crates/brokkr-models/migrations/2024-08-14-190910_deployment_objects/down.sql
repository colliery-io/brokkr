DROP TRIGGER IF EXISTS trigger_hard_delete_deployment_objects ON stacks;
DROP FUNCTION IF EXISTS hard_delete_deployment_objects_on_stack_delete();

DROP TRIGGER IF EXISTS prevent_deployment_object_delete ON deployment_objects;
DROP TRIGGER IF EXISTS prevent_deployment_object_update ON deployment_objects;
DROP FUNCTION IF EXISTS prevent_deployment_object_changes();

DROP TRIGGER IF EXISTS update_deployment_object_timestamp ON deployment_objects;

DROP INDEX IF EXISTS idx_deployment_objects_is_deletion_marker;
DROP INDEX IF EXISTS idx_deployment_objects_deleted_at;
DROP INDEX IF EXISTS idx_deployment_objects_yaml_checksum;
DROP INDEX IF EXISTS idx_deployment_objects_stack_id;
DROP INDEX IF EXISTS idx_deployment_objects_id;

DROP TABLE IF EXISTS deployment_objects;