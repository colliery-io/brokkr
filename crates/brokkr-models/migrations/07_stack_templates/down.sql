-- Rollback Stack Templates Migration

-- Drop triggers first
DROP TRIGGER IF EXISTS trigger_stack_template_hard_delete ON stack_templates;
DROP TRIGGER IF EXISTS trigger_cascade_soft_delete_generator_templates ON generators;
DROP TRIGGER IF EXISTS update_stack_templates_timestamp ON stack_templates;

-- Drop functions
DROP FUNCTION IF EXISTS handle_stack_template_hard_delete();
DROP FUNCTION IF EXISTS cascade_soft_delete_generator_templates();

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS rendered_deployment_objects;
DROP TABLE IF EXISTS template_targets;
DROP TABLE IF EXISTS template_annotations;
DROP TABLE IF EXISTS template_labels;
DROP TABLE IF EXISTS stack_templates;
