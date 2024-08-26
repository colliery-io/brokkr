-- Drop trigger
DROP TRIGGER IF EXISTS update_agent_event_timestamp ON agent_events;

-- Drop indexes
DROP INDEX IF EXISTS idx_agent_events_deleted_at;
DROP INDEX IF EXISTS idx_agent_events_created_at;
DROP INDEX IF EXISTS idx_agent_events_status;
DROP INDEX IF EXISTS idx_agent_events_event_type;
DROP INDEX IF EXISTS idx_agent_events_deployment_object_id;
DROP INDEX IF EXISTS idx_agent_events_agent_id;
DROP INDEX IF EXISTS idx_agent_events_id;

-- Drop table
DROP TABLE IF EXISTS agent_events;
