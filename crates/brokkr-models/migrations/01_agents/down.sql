-- Drop new triggers
DROP TRIGGER IF EXISTS trigger_cascade_delete_agent_annotations ON agents;
DROP TRIGGER IF EXISTS trigger_cascade_delete_agent_labels ON agents;

-- Drop new functions
DROP FUNCTION IF EXISTS cascade_delete_agent_annotations();
DROP FUNCTION IF EXISTS cascade_delete_agent_labels();

-- Existing drops (no changes)
DROP TRIGGER IF EXISTS cascade_soft_delete_agents ON agents;
DROP TRIGGER IF EXISTS trigger_agent_hard_delete ON agents;
DROP TRIGGER IF EXISTS update_agent_timestamp ON agents;

DROP FUNCTION IF EXISTS cascade_soft_delete_agents();
DROP FUNCTION IF EXISTS handle_agent_hard_delete();

DROP INDEX IF EXISTS idx_agents_deleted_at;
DROP INDEX IF EXISTS idx_agent_status;
DROP INDEX IF EXISTS idx_agent_cluster_name;
DROP INDEX IF EXISTS idx_agent_name;
DROP INDEX IF EXISTS idx_agent_id;

DROP INDEX IF EXISTS idx_agent_annotations_key;
DROP INDEX IF EXISTS idx_agent_annotations_object;
DROP INDEX IF EXISTS idx_agent_labels_label;
DROP INDEX IF EXISTS idx_agent_labels_object;

DROP TABLE IF EXISTS agent_annotations CASCADE;
DROP TABLE IF EXISTS agent_labels CASCADE;
DROP TABLE IF EXISTS agents CASCADE;
