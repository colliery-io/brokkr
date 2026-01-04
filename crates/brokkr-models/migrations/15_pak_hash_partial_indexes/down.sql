-- Revert PAK hash partial indexes

DROP INDEX IF EXISTS idx_agents_pak_hash;
DROP INDEX IF EXISTS idx_generators_pak_hash;

-- Restore original non-partial index on generators
CREATE INDEX idx_generators_api_key_hash ON generators(pak_hash);
