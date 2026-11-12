-- Migration: a soft delete cleans the child rows (BROKKR-T-0394)
--
-- A stack and an agent are deleted softly: the row stays and gets a
-- `deleted_at`. The foreign keys of the child tables are ON DELETE CASCADE,
-- which acts on a hard delete only. So the child rows stayed, and readers
-- returned them: the health of a deleted stack, a target for a deleted stack,
-- the registration of a deleted agent, a work order that only a deleted agent
-- could claim.
--
-- Rule: the trigger of the soft delete removes the child rows that have no
-- meaning without the parent. Nothing brings a deleted stack or agent back,
-- so nothing needs these rows again.

-- A deleted stack: its health records and its agent targets.
CREATE OR REPLACE FUNCTION cleanup_stack_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM deployment_health
    WHERE deployment_object_id IN (
        SELECT id FROM deployment_objects WHERE stack_id = NEW.id
    );

    DELETE FROM agent_targets
    WHERE stack_id = NEW.id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cleanup_stack_soft_delete
AFTER UPDATE OF deleted_at ON stacks
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cleanup_stack_soft_delete();

-- A deleted agent: its health records, its targets, its registrations, and
-- its place in the work order queue.
CREATE OR REPLACE FUNCTION cleanup_agent_soft_delete()
RETURNS TRIGGER AS $$
DECLARE
    targeted UUID[];
BEGIN
    DELETE FROM deployment_health
    WHERE agent_id = NEW.id;

    DELETE FROM agent_targets
    WHERE agent_id = NEW.id;

    DELETE FROM agent_generator_registrations
    WHERE agent_id = NEW.id;

    -- Give back the work orders the agent had claimed.
    UPDATE work_orders
    SET status = 'PENDING', claimed_by = NULL, claimed_at = NULL
    WHERE status = 'CLAIMED' AND claimed_by = NEW.id;

    -- Take the agent out of the targets of each work order.
    SELECT array_agg(work_order_id) INTO targeted
    FROM work_order_targets
    WHERE agent_id = NEW.id;

    DELETE FROM work_order_targets
    WHERE agent_id = NEW.id;

    -- A waiting work order that now has no target agent, no label and no
    -- annotation can be claimed by no agent: the delete cascades to it.
    DELETE FROM work_orders w
    WHERE w.id = ANY(targeted)
      AND w.status IN ('PENDING', 'RETRY_PENDING')
      AND NOT EXISTS (SELECT 1 FROM work_order_targets t WHERE t.work_order_id = w.id)
      AND NOT EXISTS (SELECT 1 FROM work_order_labels l WHERE l.work_order_id = w.id)
      AND NOT EXISTS (SELECT 1 FROM work_order_annotations a WHERE a.work_order_id = w.id);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cleanup_agent_soft_delete
AFTER UPDATE OF deleted_at ON agents
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cleanup_agent_soft_delete();

-- The rows that earlier soft deletes left behind.
DELETE FROM deployment_health
WHERE agent_id IN (SELECT id FROM agents WHERE deleted_at IS NOT NULL)
   OR deployment_object_id IN (
        SELECT d.id
        FROM deployment_objects d
        JOIN stacks s ON s.id = d.stack_id
        WHERE s.deleted_at IS NOT NULL
   );

DELETE FROM agent_targets
WHERE agent_id IN (SELECT id FROM agents WHERE deleted_at IS NOT NULL)
   OR stack_id IN (SELECT id FROM stacks WHERE deleted_at IS NOT NULL);

DELETE FROM agent_generator_registrations
WHERE agent_id IN (SELECT id FROM agents WHERE deleted_at IS NOT NULL);

UPDATE work_orders
SET status = 'PENDING', claimed_by = NULL, claimed_at = NULL
WHERE status = 'CLAIMED'
  AND claimed_by IN (SELECT id FROM agents WHERE deleted_at IS NOT NULL);

-- The work orders that had a deleted agent as a target, and that no agent can
-- claim once that target is gone.
DELETE FROM work_orders w
WHERE w.status IN ('PENDING', 'RETRY_PENDING')
  AND EXISTS (
        SELECT 1
        FROM work_order_targets t
        JOIN agents a ON a.id = t.agent_id
        WHERE t.work_order_id = w.id AND a.deleted_at IS NOT NULL
  )
  AND NOT EXISTS (
        SELECT 1
        FROM work_order_targets t
        JOIN agents a ON a.id = t.agent_id
        WHERE t.work_order_id = w.id AND a.deleted_at IS NULL
  )
  AND NOT EXISTS (SELECT 1 FROM work_order_labels l WHERE l.work_order_id = w.id)
  AND NOT EXISTS (SELECT 1 FROM work_order_annotations a WHERE a.work_order_id = w.id);

DELETE FROM work_order_targets
WHERE agent_id IN (SELECT id FROM agents WHERE deleted_at IS NOT NULL);
