-- Migration: agent_k8s_events + agent_pod_logs
--
-- Short-lived operational buffers populated by `brokkr-agent` via the
-- internal /internal/ws/agent channel (BROKKR-I-0019).
--
-- Retention is bounded by a hard 6-hour ceiling enforced in-process by
-- the broker's eviction worker (see crates/brokkr-broker/src/ws/eviction.rs).
-- The ceiling is intentional product stance, not just a default — Brokkr
-- is NOT a log warehouse. Long-term log centralisation belongs in Datadog
-- or equivalent. See project_log_retention_stance memory.
--
-- The retention worker uses `created_at` (server-side ingestion time) so
-- a misbehaving agent that sends timestamps from the past can't keep
-- ancient data alive past the ceiling.

CREATE TABLE agent_k8s_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    stack_id UUID NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    observed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reason TEXT NOT NULL,
    message TEXT NOT NULL,
    event_type TEXT NOT NULL,
    source TEXT,
    involved_object JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_agent_k8s_events_stack_created ON agent_k8s_events (stack_id, created_at DESC);
CREATE INDEX idx_agent_k8s_events_created ON agent_k8s_events (created_at);

CREATE TABLE agent_pod_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    stack_id UUID NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    pod TEXT NOT NULL,
    container TEXT NOT NULL,
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    line TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_agent_pod_logs_stack_created ON agent_pod_logs (stack_id, created_at DESC);
CREATE INDEX idx_agent_pod_logs_created ON agent_pod_logs (created_at);
