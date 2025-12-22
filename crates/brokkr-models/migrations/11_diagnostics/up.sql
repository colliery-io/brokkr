-- Diagnostic requests table for on-demand diagnostics
-- Operators request diagnostics, agents pick them up and execute
CREATE TABLE diagnostic_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    deployment_object_id UUID NOT NULL REFERENCES deployment_objects(id) ON DELETE CASCADE,

    -- Request state
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    requested_by VARCHAR(255),

    -- Timing
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    claimed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,

    -- Constraints
    CONSTRAINT valid_diagnostic_request_status CHECK (status IN ('pending', 'claimed', 'completed', 'failed', 'expired'))
);

-- Indexes for diagnostic_requests
CREATE INDEX idx_diagnostic_requests_agent_pending ON diagnostic_requests(agent_id) WHERE status = 'pending';
CREATE INDEX idx_diagnostic_requests_expires ON diagnostic_requests(expires_at);
CREATE INDEX idx_diagnostic_requests_deployment ON diagnostic_requests(deployment_object_id);

-- Diagnostic results table for storing collected diagnostic data
CREATE TABLE diagnostic_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES diagnostic_requests(id) ON DELETE CASCADE,

    -- Diagnostic data (stored as TEXT, JSON-encoded)
    pod_statuses TEXT NOT NULL,
    events TEXT NOT NULL,
    log_tails TEXT,

    -- Metadata
    collected_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for diagnostic_results
CREATE INDEX idx_diagnostic_results_request ON diagnostic_results(request_id);

-- Auto-update timestamp trigger for diagnostic_requests (reusing existing function)
-- Note: We don't add updated_at to diagnostic_requests as status changes are tracked via claimed_at/completed_at
