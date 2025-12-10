-- Migration: 08_work_orders (rollback)
-- Description: Remove work order system tables and types

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS work_order_targets;
DROP TABLE IF EXISTS work_order_log;
DROP TABLE IF EXISTS work_orders;

-- Drop enum type
DROP TYPE IF EXISTS work_order_status;
