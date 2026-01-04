-- Add partial indexes on pak_hash for O(1) authentication lookups
-- Partial indexes exclude soft-deleted records (saves space, faster queries)

-- Index for agent PAK authentication
CREATE INDEX idx_agents_pak_hash ON agents(pak_hash)
    WHERE deleted_at IS NULL AND pak_hash IS NOT NULL AND pak_hash != '';

-- Drop existing non-partial index on generators and replace with partial index
DROP INDEX IF EXISTS idx_generators_api_key_hash;

CREATE INDEX idx_generators_pak_hash ON generators(pak_hash)
    WHERE deleted_at IS NULL AND pak_hash IS NOT NULL;
