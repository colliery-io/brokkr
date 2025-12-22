-- Remove error tracking columns from work_orders table

DROP INDEX IF EXISTS idx_work_orders_last_error;

ALTER TABLE work_orders DROP COLUMN IF EXISTS last_error_at;
ALTER TABLE work_orders DROP COLUMN IF EXISTS last_error;
