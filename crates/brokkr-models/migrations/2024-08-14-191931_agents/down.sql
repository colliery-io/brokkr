DROP TRIGGER IF EXISTS cascade_soft_delete_agents ON agents;
DROP FUNCTION IF EXISTS cascade_soft_delete_agents();

DROP TRIGGER IF EXISTS update_agent_timestamp ON agents;

DROP INDEX IF EXISTS idx_agents_deleted_at;
DROP INDEX IF EXISTS idx_agent_annotations;
DROP INDEX IF EXISTS idx_agent_labels;
DROP INDEX IF EXISTS idx_agent_status;
DROP INDEX IF EXISTS idx_agent_cluster_name;
DROP INDEX IF EXISTS idx_agent_name;
DROP INDEX IF EXISTS idx_agent_id;

DROP TABLE IF EXISTS agents;