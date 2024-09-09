-- Existing agents table creation (no changes)
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name VARCHAR(255) NOT NULL,
    cluster_name VARCHAR(255) NOT NULL,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'INACTIVE',
    pak_hash TEXT NOT NULL DEFAULT '',
    CONSTRAINT unique_agent_cluster UNIQUE (name, cluster_name)
);

-- Existing indexes (no changes)
CREATE INDEX idx_agent_id ON agents(id);
CREATE INDEX idx_agent_name ON agents (name);
CREATE INDEX idx_agent_cluster_name ON agents (cluster_name);
CREATE INDEX idx_agent_status ON agents (status);
CREATE INDEX idx_agents_deleted_at ON agents(deleted_at);

-- Existing trigger (no changes)
CREATE TRIGGER update_agent_timestamp
BEFORE UPDATE ON agents
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Updated function for cascading soft delete of agents
CREATE OR REPLACE FUNCTION cascade_soft_delete_agents()
RETURNS TRIGGER AS $$
BEGIN
    
    UPDATE agent_events
    SET deleted_at = NEW.deleted_at
    WHERE agent_id = NEW.id AND deleted_at IS NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Existing trigger for cascading soft delete of agents (no changes)
CREATE TRIGGER cascade_soft_delete_agents
AFTER UPDATE ON agents
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cascade_soft_delete_agents();

-- Updated function for hard deletion of agents
CREATE OR REPLACE FUNCTION handle_agent_hard_delete()
RETURNS TRIGGER AS $$
BEGIN
    
    -- Delete agent_target rows associated with the agent
    DELETE FROM agent_targets
    WHERE agent_id = OLD.id;

    -- Delete agent_events associated with the agent
    DELETE FROM agent_events
    WHERE agent_id = OLD.id;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Existing trigger for agent hard deletion (no changes)
CREATE TRIGGER trigger_agent_hard_delete
BEFORE DELETE ON agents
FOR EACH ROW
EXECUTE FUNCTION handle_agent_hard_delete();





-- Existing agent_labels table creation (no changes)
CREATE TABLE agent_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    label VARCHAR(64) NOT NULL,
    UNIQUE (agent_id, label),
    CONSTRAINT fk_agent_labels_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);


CREATE INDEX idx_agent_labels_object ON agent_labels (agent_id);
CREATE INDEX idx_agent_labels_label ON agent_labels (label);

-- Existing agent_annotations table creation (no changes)
CREATE TABLE agent_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    key VARCHAR(64) NOT NULL,
    value VARCHAR(64) NOT NULL,
    UNIQUE (agent_id, key),
    CONSTRAINT fk_agent_annotations_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX idx_agent_annotations_object ON agent_annotations (agent_id);
CREATE INDEX idx_agent_annotations_key ON agent_annotations (key);