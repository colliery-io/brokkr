-- Migration: the telemetry of a work order (BROKKR-T-0405, BROKKR-A-0013)
--
-- `agent_k8s_events` and `agent_pod_logs` held the events and the pod log
-- lines of a stack only. The owner of a work order reads the same for the job
-- of its work order, and a work order has no stack.
--
-- Each row now belongs to a stack or to a work order: exactly one of
-- `stack_id` and `work_order_id` is set.
--
-- `work_order_id` has no foreign key. The row of a work order is deleted when
-- the work order ends, and its log entry (`work_order_log`) keeps the same id.
-- The rows here must stay readable after that. The 6 hour ceiling of the
-- eviction worker removes them, as it removes the rows of a stack.

ALTER TABLE agent_k8s_events
    ALTER COLUMN stack_id DROP NOT NULL,
    ADD COLUMN work_order_id UUID,
    ADD CONSTRAINT agent_k8s_events_one_subject
        CHECK ((stack_id IS NULL) <> (work_order_id IS NULL));

ALTER TABLE agent_pod_logs
    ALTER COLUMN stack_id DROP NOT NULL,
    ADD COLUMN work_order_id UUID,
    ADD CONSTRAINT agent_pod_logs_one_subject
        CHECK ((stack_id IS NULL) <> (work_order_id IS NULL));

CREATE INDEX idx_agent_k8s_events_work_order_created
    ON agent_k8s_events (work_order_id, created_at DESC)
    WHERE work_order_id IS NOT NULL;
CREATE INDEX idx_agent_pod_logs_work_order_created
    ON agent_pod_logs (work_order_id, created_at DESC)
    WHERE work_order_id IS NOT NULL;
