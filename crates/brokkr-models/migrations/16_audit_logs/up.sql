-- Audit logs for tracking administrative and security-sensitive operations
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- When the event occurred
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Who performed the action
    actor_type VARCHAR(20) NOT NULL,    -- admin, agent, generator, system
    actor_id UUID,                       -- NULL for system actions or unauthenticated

    -- What action was performed
    action VARCHAR(100) NOT NULL,        -- e.g., 'pak.created', 'agent.deleted', 'auth.failed'

    -- What resource was affected
    resource_type VARCHAR(50) NOT NULL,  -- e.g., 'agent', 'stack', 'webhook_subscription'
    resource_id UUID,                    -- NULL if action doesn't target specific resource

    -- Additional context
    details JSONB,                       -- Structured details (changes, metadata, etc.)
    ip_address TEXT,                     -- Client IP address (stored as text for simplicity)
    user_agent TEXT,                     -- Client user agent string

    -- Record metadata (immutable - no updated_at)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT valid_actor_type CHECK (actor_type IN ('admin', 'agent', 'generator', 'system'))
);

-- Indexes for common query patterns
-- Time-based queries (most common)
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);

-- Actor queries (who did what)
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_type, actor_id, timestamp DESC);

-- Resource queries (what happened to X)
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id, timestamp DESC);

-- Action queries (all failed auths, all deletions, etc.)
CREATE INDEX idx_audit_logs_action ON audit_logs(action, timestamp DESC);

-- Retention cleanup (delete old records efficiently)
CREATE INDEX idx_audit_logs_cleanup ON audit_logs(created_at);

-- Comment for documentation
COMMENT ON TABLE audit_logs IS 'Immutable audit trail for compliance and security. Records are never updated or deleted except by retention policy.';
