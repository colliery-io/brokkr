-- Down script

-- Drop cascading soft delete triggers and functions
DROP TRIGGER IF EXISTS cascade_soft_delete_agents ON agents;
DROP FUNCTION IF EXISTS cascade_soft_delete_agents();

DROP TRIGGER IF EXISTS cascade_soft_delete_deployment_objects ON deployment_objects;
DROP FUNCTION IF EXISTS cascade_soft_delete_deployment_objects();

DROP TRIGGER IF EXISTS cascade_soft_delete_stacks ON stacks;
DROP FUNCTION IF EXISTS cascade_soft_delete_stacks();

-- Drop view
DROP VIEW IF EXISTS active_agent_events;

-- Drop triggers on agent_events
DROP TRIGGER IF EXISTS soft_delete_agent_events ON agent_events;
DROP TRIGGER IF EXISTS update_agent_event_timestamp ON agent_events;

-- Drop indexes on agent_events
DROP INDEX IF EXISTS idx_agent_events_deleted_at;
DROP INDEX IF EXISTS idx_agent_events_created_at;
DROP INDEX IF EXISTS idx_agent_events_status;
DROP INDEX IF EXISTS idx_agent_events_event_type;
DROP INDEX IF EXISTS idx_agent_events_deployment_object_id;
DROP INDEX IF EXISTS idx_agent_events_agent_id;
DROP INDEX IF EXISTS idx_agent_events_uuid;

-- Drop agent_events table
DROP TABLE IF EXISTS agent_events;

-- Drop agent_event_status type
DROP TYPE IF EXISTS agent_event_status;