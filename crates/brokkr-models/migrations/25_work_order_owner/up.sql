-- Migration: a work order has an owner (BROKKR-T-0402, BROKKR-A-0013)
--
-- The owner is who created the work order: a generator, or an admin.
-- `owner_generator_id` names the generator. NULL means an admin owns it. The
-- work orders and the log entries that exist are admin-owned.
--
-- The log entry of a work order keeps the owner, because the work order row
-- is deleted when it ends.
--
-- A hard delete of the generator deletes both. The owner must never fall
-- back to NULL: that would give a generator's work order to the admins.

ALTER TABLE work_orders
    ADD COLUMN owner_generator_id UUID REFERENCES generators(id) ON DELETE CASCADE;

ALTER TABLE work_order_log
    ADD COLUMN owner_generator_id UUID REFERENCES generators(id) ON DELETE CASCADE;

CREATE INDEX idx_work_orders_owner ON work_orders(owner_generator_id)
    WHERE owner_generator_id IS NOT NULL;
CREATE INDEX idx_work_order_log_owner ON work_order_log(owner_generator_id)
    WHERE owner_generator_id IS NOT NULL;

-- A generator soft delete: the work orders it owns that still wait are
-- deleted (the rule of migration 24: the trigger cleans the child rows). A
-- claimed work order is left to end. The log entries stay.
CREATE OR REPLACE FUNCTION cleanup_generator_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM work_orders
    WHERE owner_generator_id = NEW.id
      AND status IN ('PENDING', 'RETRY_PENDING');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cleanup_generator_soft_delete
AFTER UPDATE OF deleted_at ON generators
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cleanup_generator_soft_delete();
