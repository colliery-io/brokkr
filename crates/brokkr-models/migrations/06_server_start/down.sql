-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS app_initialization;

DROP TRIGGER IF EXISTS prevent_admin_role_delete ON admin_role;
DROP FUNCTION IF EXISTS prevent_admin_role_delete();
DROP TRIGGER IF EXISTS update_admin_role_timestamp ON admin_role;
DROP INDEX IF EXISTS idx_admin_role_id;
DROP TABLE IF EXISTS admin_role;