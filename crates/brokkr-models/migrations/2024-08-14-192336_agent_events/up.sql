-- Create the agent_events table
CREATE TABLE IF NOT EXISTS agent_events (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    deployment_object_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    status VARCHAR(10) NOT NULL,
    message TEXT,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_deployment_object FOREIGN KEY (deployment_object_id) REFERENCES deployment_objects(uuid)
) INHERITS (base_table);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_agent_events_uuid ON agent_events (uuid);
CREATE INDEX IF NOT EXISTS idx_agent_events_agent_id ON agent_events (agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_events_deployment_object_id ON agent_events (deployment_object_id);
CREATE INDEX IF NOT EXISTS idx_agent_events_event_type ON agent_events (event_type);
CREATE INDEX IF NOT EXISTS idx_agent_events_status ON agent_events (status);
CREATE INDEX IF NOT EXISTS idx_agent_events_created_at ON agent_events (created_at);
CREATE INDEX IF NOT EXISTS idx_agent_events_deleted_at ON agent_events(deleted_at);

-- Create trigger for updating timestamp
CREATE TRIGGER update_agent_event_timestamp
BEFORE UPDATE ON agent_events
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Create trigger for soft delete
CREATE TRIGGER soft_delete_agent_events
BEFORE UPDATE ON agent_events
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION soft_delete();


-- Function for cascading soft delete (existing)
CREATE OR REPLACE FUNCTION cascade_soft_delete_agents()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE agent_events
    SET deleted_at = NEW.deleted_at
    WHERE agent_id = NEW.uuid AND deleted_at IS NULL;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for cascading soft delete (existing)
CREATE TRIGGER cascade_soft_delete_agents
AFTER UPDATE ON agents
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cascade_soft_delete_agents();

