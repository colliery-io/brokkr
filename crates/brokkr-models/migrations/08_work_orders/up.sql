-- Migration: 08_work_orders
-- Description: Work order system for managing transient operations (builds, tests, backups, etc.)
-- Architecture: Two-table design separating active work routing from permanent audit logging

-- =============================================================================
-- WORK ORDER STATUS
-- =============================================================================
-- Work order queue states (stored as VARCHAR(20) for Diesel compatibility):
--   PENDING: Ready to be claimed by an agent
--   CLAIMED: Currently being processed by an agent
--   RETRY_PENDING: Failed but waiting for retry backoff period

-- =============================================================================
-- WORK ORDERS TABLE (Active Queue)
-- =============================================================================
-- Stores active work orders that are pending, claimed, or awaiting retry.
-- Records are moved to work_order_log upon completion (success or final failure).
CREATE TABLE work_orders (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Work type discriminator (e.g., 'build', 'test', 'backup')
    work_type VARCHAR(50) NOT NULL,

    -- Multi-document YAML content (e.g., Build + WorkOrder definitions)
    yaml_content TEXT NOT NULL,

    -- Queue state management (PENDING, CLAIMED, RETRY_PENDING)
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    claimed_by UUID REFERENCES agents(id) ON DELETE SET NULL,
    claimed_at TIMESTAMP WITH TIME ZONE,

    -- Claim timeout: if claimed_at + claim_timeout_seconds < NOW(), claim is stale
    claim_timeout_seconds INTEGER NOT NULL DEFAULT 3600,

    -- Retry management (broker-side)
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_count INTEGER NOT NULL DEFAULT 0,
    backoff_seconds INTEGER NOT NULL DEFAULT 60,
    next_retry_after TIMESTAMP WITH TIME ZONE,

    -- Constraints
    CONSTRAINT valid_retry_count CHECK (retry_count >= 0 AND retry_count <= max_retries),
    CONSTRAINT valid_max_retries CHECK (max_retries >= 0),
    CONSTRAINT valid_backoff CHECK (backoff_seconds > 0),
    CONSTRAINT valid_claim_timeout CHECK (claim_timeout_seconds > 0),
    CONSTRAINT claimed_requires_agent CHECK (
        (status = 'CLAIMED' AND claimed_by IS NOT NULL AND claimed_at IS NOT NULL) OR
        (status != 'CLAIMED')
    )
);

-- Indexes for work_orders (optimized for queue operations)
CREATE INDEX idx_work_orders_status ON work_orders(status);
CREATE INDEX idx_work_orders_type ON work_orders(work_type);
CREATE INDEX idx_work_orders_claimed_by ON work_orders(claimed_by) WHERE claimed_by IS NOT NULL;
CREATE INDEX idx_work_orders_stale_claims ON work_orders(claimed_at, claim_timeout_seconds)
    WHERE status = 'CLAIMED';
CREATE INDEX idx_work_orders_retry ON work_orders(next_retry_after)
    WHERE status = 'RETRY_PENDING';
CREATE INDEX idx_work_orders_pending_type ON work_orders(work_type, created_at)
    WHERE status = 'PENDING';

-- Trigger for automatic timestamp updates
CREATE TRIGGER update_work_orders_timestamp
    BEFORE UPDATE ON work_orders
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp();

-- =============================================================================
-- WORK ORDER LOG TABLE (Audit Trail)
-- =============================================================================
-- Permanent record of completed work orders (successful or failed after max retries).
-- Provides complete execution history for debugging, analytics, and compliance.
CREATE TABLE work_order_log (
    -- Primary key (matches original work_orders.id)
    id UUID PRIMARY KEY,

    -- Work type discriminator
    work_type VARCHAR(50) NOT NULL,

    -- Timestamps from original work order
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    claimed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Execution details
    claimed_by UUID REFERENCES agents(id) ON DELETE SET NULL,
    success BOOLEAN NOT NULL,
    retries_attempted INTEGER NOT NULL DEFAULT 0,
    result_message TEXT,

    -- Store YAML content for debugging/reconstruction
    yaml_content TEXT NOT NULL,

    -- Constraints
    CONSTRAINT valid_retries_attempted CHECK (retries_attempted >= 0)
);

-- Indexes for work_order_log (optimized for analytics and querying)
CREATE INDEX idx_work_order_log_type ON work_order_log(work_type);
CREATE INDEX idx_work_order_log_success ON work_order_log(success);
CREATE INDEX idx_work_order_log_completed_at ON work_order_log(completed_at);
CREATE INDEX idx_work_order_log_claimed_by ON work_order_log(claimed_by) WHERE claimed_by IS NOT NULL;
CREATE INDEX idx_work_order_log_type_completed ON work_order_log(work_type, completed_at DESC);

-- =============================================================================
-- WORK ORDER TARGETS TABLE (Agent Routing)
-- =============================================================================
-- Junction table linking work orders to eligible agents based on label matching.
-- Mirrors the stack targeting pattern (agent_targets table).
CREATE TABLE work_order_targets (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign keys
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint: one entry per work_order/agent pair
    UNIQUE(work_order_id, agent_id)
);

-- Indexes for work_order_targets
CREATE INDEX idx_work_order_targets_order ON work_order_targets(work_order_id);
CREATE INDEX idx_work_order_targets_agent ON work_order_targets(agent_id);

-- =============================================================================
-- COMMENTS
-- =============================================================================
COMMENT ON TABLE work_orders IS 'Active work order queue for routing and retry management';
COMMENT ON TABLE work_order_log IS 'Permanent audit trail of completed work orders';
COMMENT ON TABLE work_order_targets IS 'Junction table linking work orders to eligible agents';

COMMENT ON COLUMN work_orders.work_type IS 'Type discriminator: build, test, backup, etc.';
COMMENT ON COLUMN work_orders.yaml_content IS 'Multi-document YAML (e.g., Build + WorkOrder definitions)';
COMMENT ON COLUMN work_orders.status IS 'Queue state: PENDING (claimable), CLAIMED (processing), RETRY_PENDING (backoff)';
COMMENT ON COLUMN work_orders.claim_timeout_seconds IS 'Seconds before a claimed work order is considered stale';
COMMENT ON COLUMN work_orders.backoff_seconds IS 'Base backoff for exponential retry calculation';
COMMENT ON COLUMN work_orders.next_retry_after IS 'Timestamp when RETRY_PENDING work order becomes PENDING again';

COMMENT ON COLUMN work_order_log.success IS 'True if work completed successfully, false if failed after max retries';
COMMENT ON COLUMN work_order_log.result_message IS 'Success: image digest, etc. Failure: error details';
