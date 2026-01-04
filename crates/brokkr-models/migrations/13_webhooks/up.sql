-- Webhook subscriptions for event notifications
CREATE TABLE webhook_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Subscription details
    name VARCHAR(255) NOT NULL,
    url_encrypted BYTEA NOT NULL,           -- Encrypted; may contain embedded tokens
    auth_header_encrypted BYTEA,            -- Encrypted; optional Authorization header value

    -- Event filtering
    event_types TEXT[] NOT NULL,            -- Array of event type patterns (e.g., 'deployment.*')
    filters TEXT,                           -- Optional JSON-encoded filters: agent_id, stack_id, labels

    -- Delivery targeting
    -- NULL/empty = broker delivers; labels = matching agent delivers
    target_labels TEXT[],

    -- Delivery settings
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INT NOT NULL DEFAULT 5,
    timeout_seconds INT NOT NULL DEFAULT 30,

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255)
);

-- Webhook delivery tracking
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE,

    -- Event info
    event_type VARCHAR(100) NOT NULL,
    event_id UUID NOT NULL,                 -- Idempotency key
    payload TEXT NOT NULL,                  -- JSON-encoded event payload

    -- Targeting (copied from subscription at creation time)
    target_labels TEXT[],                   -- NULL = broker delivers; labels = agent delivers

    -- Delivery status
    -- pending:  waiting to be claimed
    -- acquired: claimed, being processed (has TTL via acquired_until)
    -- success:  delivered successfully
    -- failed:   attempt failed, will retry (goes back to pending)
    -- dead:     max retries exceeded
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Claim tracking
    acquired_by UUID REFERENCES agents(id) ON DELETE SET NULL,  -- Agent ID or NULL for broker
    acquired_until TIMESTAMP WITH TIME ZONE,                     -- TTL - if exceeded, release back to pending

    -- Retry tracking
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMP WITH TIME ZONE,
    next_retry_at TIMESTAMP WITH TIME ZONE,             -- When to retry after failure
    last_error TEXT,

    -- Timing
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,

    -- Constraints
    CONSTRAINT valid_delivery_status CHECK (status IN ('pending', 'acquired', 'success', 'failed', 'dead'))
);

-- Indexes for webhook_subscriptions
CREATE INDEX idx_webhook_subscriptions_enabled ON webhook_subscriptions(enabled) WHERE enabled = true;

-- Indexes for webhook_deliveries
-- Pending deliveries ready for claiming (broker or agent)
CREATE INDEX idx_webhook_deliveries_pending
    ON webhook_deliveries(created_at)
    WHERE status = 'pending';

-- Expired acquired deliveries (for TTL recovery)
CREATE INDEX idx_webhook_deliveries_acquired_expired
    ON webhook_deliveries(acquired_until)
    WHERE status = 'acquired';

-- Failed deliveries ready for retry
CREATE INDEX idx_webhook_deliveries_retry
    ON webhook_deliveries(next_retry_at)
    WHERE status = 'failed';

-- Subscription delivery history
CREATE INDEX idx_webhook_deliveries_subscription
    ON webhook_deliveries(subscription_id, created_at DESC);

-- Cleanup index for old deliveries
CREATE INDEX idx_webhook_deliveries_cleanup
    ON webhook_deliveries(created_at)
    WHERE status IN ('success', 'dead');

-- Auto-update timestamp trigger for webhook_subscriptions
CREATE TRIGGER update_webhook_subscriptions_updated_at
    BEFORE UPDATE ON webhook_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp();
