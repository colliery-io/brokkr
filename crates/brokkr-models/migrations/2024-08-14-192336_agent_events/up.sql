CREATE TABLE agent_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    agent_id UUID NOT NULL,
    deployment_object_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    status VARCHAR(10) NOT NULL,
    message TEXT,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    CONSTRAINT fk_deployment_object FOREIGN KEY (deployment_object_id) REFERENCES deployment_objects(id)
);

CREATE INDEX idx_agent_events_id ON agent_events(id);
CREATE INDEX idx_agent_events_agent_id ON agent_events (agent_id);
CREATE INDEX idx_agent_events_deployment_object_id ON agent_events (deployment_object_id);
CREATE INDEX idx_agent_events_event_type ON agent_events (event_type);
CREATE INDEX idx_agent_events_status ON agent_events (status);
CREATE INDEX idx_agent_events_created_at ON agent_events (created_at);
CREATE INDEX idx_agent_events_deleted_at ON agent_events(deleted_at);

CREATE TRIGGER update_agent_event_timestamp
BEFORE UPDATE ON agent_events
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();