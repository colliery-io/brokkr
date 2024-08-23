CREATE TABLE agents (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    cluster_name VARCHAR(255) NOT NULL,
    labels JSONB,
    annotations JSONB,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'INACTIVE',
    CONSTRAINT unique_agent_cluster UNIQUE (name, cluster_name)
) INHERITS (base_table);

CREATE INDEX idx_agent_uuid ON agents (uuid);
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

CREATE TRIGGER soft_delete_agents
BEFORE UPDATE ON agents
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION soft_delete();
