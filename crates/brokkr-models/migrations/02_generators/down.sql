-- Remove generator_id from deployment_objects table
DROP INDEX IF EXISTS idx_deployment_objects_generator_id;
ALTER TABLE deployment_objects DROP COLUMN IF EXISTS generator_id;

-- Remove generator_id from stacks table
DROP INDEX IF EXISTS idx_stacks_generator_id;
ALTER TABLE stacks DROP COLUMN IF EXISTS generator_id;

-- Drop generators table
DROP TRIGGER IF EXISTS update_generators_timestamp ON generators;
DROP INDEX IF EXISTS idx_generators_api_key_hash;
DROP INDEX IF EXISTS idx_generators_name;
DROP TABLE IF EXISTS generators;
