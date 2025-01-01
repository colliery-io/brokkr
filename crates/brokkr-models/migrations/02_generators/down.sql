-- Drop generators table
DROP TRIGGER IF EXISTS update_generators_timestamp ON generators;
DROP INDEX IF EXISTS idx_generators_api_key_hash;
DROP INDEX IF EXISTS idx_generators_name;
DROP TABLE IF EXISTS generators;
