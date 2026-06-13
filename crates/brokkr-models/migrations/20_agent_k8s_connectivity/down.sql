-- Rollback: drop agent-reported Kubernetes connectivity columns.

ALTER TABLE agents
    DROP COLUMN IF EXISTS k8s_reported_at,
    DROP COLUMN IF EXISTS k8s_api_latency_ms,
    DROP COLUMN IF EXISTS k8s_reachable;
