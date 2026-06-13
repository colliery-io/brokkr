-- Migration: agent-reported Kubernetes connectivity (BROKKR-T-0227)
--
-- The one fleet signal the broker cannot compute on its own: whether each
-- agent can reach its own Kubernetes API. The agent self-reports it on the
-- heartbeat cycle; the broker stores the latest snapshot per agent on the
-- agents row and surfaces it in the fleet record.
--
-- All three columns are nullable so agents that never report (or cannot
-- determine reachability) leave them NULL — "trust the agent", graceful
-- degradation, no backfill required. `k8s_reported_at` records the
-- server-side ingestion time of the most recent report so readers can judge
-- the freshness of `k8s_reachable` / `k8s_api_latency_ms`.

ALTER TABLE agents
    ADD COLUMN k8s_reachable BOOLEAN NULL,
    ADD COLUMN k8s_api_latency_ms INTEGER NULL,
    ADD COLUMN k8s_reported_at TIMESTAMP WITH TIME ZONE NULL;
