-- Drop the functions
DROP FUNCTION IF EXISTS soft_delete();
DROP FUNCTION IF EXISTS update_timestamp();

-- Drop the index
DROP INDEX IF EXISTS idx_base_table_id;

-- Drop the table
DROP TABLE IF EXISTS base_table;