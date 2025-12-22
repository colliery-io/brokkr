-- Add error tracking columns to work_orders table
-- These columns store the most recent failure information for monitoring purposes

ALTER TABLE work_orders ADD COLUMN last_error TEXT;
ALTER TABLE work_orders ADD COLUMN last_error_at TIMESTAMPTZ;

-- Add index for querying failed work orders
CREATE INDEX idx_work_orders_last_error ON work_orders (last_error_at) WHERE last_error IS NOT NULL;

COMMENT ON COLUMN work_orders.last_error IS 'Most recent error message from failed execution attempt';
COMMENT ON COLUMN work_orders.last_error_at IS 'Timestamp of the most recent failure';
