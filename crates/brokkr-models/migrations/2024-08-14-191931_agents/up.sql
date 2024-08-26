CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name VARCHAR(255) NOT NULL,
    cluster_name VARCHAR(255) NOT NULL,
    labels JSONB,
    annotations JSONB,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'INACTIVE',
    CONSTRAINT unique_agent_cluster UNIQUE (name, cluster_name)
);

CREATE INDEX idx_agent_id ON agents(id);
CREATE INDEX idx_agent_name ON agents (name);
CREATE INDEX idx_agent_cluster_name ON agents (cluster_name);
CREATE INDEX idx_agent_status ON agents (status);
CREATE INDEX idx_agent_labels ON agents USING gin (labels);
CREATE INDEX idx_agent_annotations ON agents USING gin (annotations);
CREATE INDEX idx_agents_deleted_at ON agents(deleted_at);

CREATE TRIGGER update_agent_timestamp
BEFORE UPDATE ON agents
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Function for cascading soft delete of agents
CREATE OR REPLACE FUNCTION cascade_soft_delete_agents()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE agent_events
    SET deleted_at = NEW.deleted_at
    WHERE agent_id = NEW.id AND deleted_at IS NULL;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for cascading soft delete of agents
CREATE TRIGGER cascade_soft_delete_agents
AFTER UPDATE ON agents
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cascade_soft_delete_agents();
