-- Rollback: drop agent telemetry tables.

DROP TABLE IF EXISTS agent_pod_logs;
DROP TABLE IF EXISTS agent_k8s_events;
