-- Rollback: the telemetry tables hold the rows of stacks only. The rows of
-- work orders are deleted: they have no stack, and `stack_id` is NOT NULL
-- again.

DROP INDEX IF EXISTS idx_agent_pod_logs_work_order_created;
DROP INDEX IF EXISTS idx_agent_k8s_events_work_order_created;

DELETE FROM agent_pod_logs WHERE stack_id IS NULL;
DELETE FROM agent_k8s_events WHERE stack_id IS NULL;

ALTER TABLE agent_pod_logs
    DROP CONSTRAINT IF EXISTS agent_pod_logs_one_subject,
    DROP COLUMN IF EXISTS work_order_id,
    ALTER COLUMN stack_id SET NOT NULL;

ALTER TABLE agent_k8s_events
    DROP CONSTRAINT IF EXISTS agent_k8s_events_one_subject,
    DROP COLUMN IF EXISTS work_order_id,
    ALTER COLUMN stack_id SET NOT NULL;
