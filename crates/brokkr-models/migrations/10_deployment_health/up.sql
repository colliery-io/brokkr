-- Migration: Add deployment_health table for tracking health status per agent+deployment
--
-- Health status is tracked per agent+deployment_object combination.
-- This preserves the immutability of deployment_objects while properly modeling
-- that different agents may have different health for the same deployment.

CREATE TABLE deployment_health (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    deployment_object_id UUID NOT NULL REFERENCES deployment_objects(id) ON DELETE CASCADE,

    -- Health status
    status VARCHAR(20) NOT NULL,              -- healthy, degraded, failing, unknown
    summary TEXT,                              -- JSON-encoded health summary

    -- Timing
    checked_at TIMESTAMP WITH TIME ZONE NOT NULL,  -- when agent checked health
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- One health record per agent+deployment combination
    CONSTRAINT unique_agent_deployment_health UNIQUE (agent_id, deployment_object_id),

    -- Validate status values
    CONSTRAINT valid_health_status CHECK (status IN ('healthy', 'degraded', 'failing', 'unknown'))
);

-- Indexes for common query patterns
CREATE INDEX idx_deployment_health_agent ON deployment_health(agent_id);
CREATE INDEX idx_deployment_health_deployment ON deployment_health(deployment_object_id);
CREATE INDEX idx_deployment_health_status ON deployment_health(status);
CREATE INDEX idx_deployment_health_checked_at ON deployment_health(checked_at);

-- Auto-update timestamp on modifications
CREATE TRIGGER update_deployment_health_timestamp
BEFORE UPDATE ON deployment_health
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
