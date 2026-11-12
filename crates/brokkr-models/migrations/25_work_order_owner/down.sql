-- Rollback: work orders have no owner again. Each one is then what it was
-- before: created and read by an admin only.

DROP TRIGGER IF EXISTS trigger_cleanup_generator_soft_delete ON generators;
DROP FUNCTION IF EXISTS cleanup_generator_soft_delete();

DROP INDEX IF EXISTS idx_work_order_log_owner;
DROP INDEX IF EXISTS idx_work_orders_owner;

ALTER TABLE work_order_log DROP COLUMN IF EXISTS owner_generator_id;
ALTER TABLE work_orders DROP COLUMN IF EXISTS owner_generator_id;
