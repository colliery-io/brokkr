
-- Drop the view
DROP VIEW IF EXISTS active_agents;

-- Drop the triggers
DROP TRIGGER IF EXISTS soft_delete_agents ON agents;
DROP TRIGGER IF EXISTS update_agent_timestamp ON agents;

-- Drop the indexes
DROP INDEX IF EXISTS idx_agents_deleted_at;
DROP INDEX IF EXISTS idx_agent_annotations;
DROP INDEX IF EXISTS idx_agent_labels;
DROP INDEX IF EXISTS idx_agent_status;
DROP INDEX IF EXISTS idx_agent_cluster_name;
DROP INDEX IF EXISTS idx_agent_name;
DROP INDEX IF EXISTS idx_agent_uuid;

-- Drop the table
DROP TABLE IF EXISTS agents;