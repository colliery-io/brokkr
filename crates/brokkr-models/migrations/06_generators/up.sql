CREATE TABLE generators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    pak_hash TEXT NOT NULL,
    last_active_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_generators_name ON generators(name);
CREATE INDEX idx_generators_api_key_hash ON generators(pak_hash);
CREATE INDEX idx_generators_deleted_at ON generators(deleted_at);

CREATE TRIGGER update_generators_timestamp
BEFORE UPDATE ON generators
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Add generator_id to stacks table
ALTER TABLE stacks
ADD COLUMN generator_id UUID NOT NULL REFERENCES generators(id);

CREATE INDEX idx_stacks_generator_id ON stacks(generator_id);

-- Add generator_id to deployment_objects table
ALTER TABLE deployment_objects
ADD COLUMN generator_id UUID NOT NULL REFERENCES generators(id);

CREATE INDEX idx_deployment_objects_generator_id ON deployment_objects(generator_id);

-- Function for cascading soft delete of generators
CREATE OR REPLACE FUNCTION cascade_soft_delete_generators()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;

    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for cascading soft delete of generators
CREATE TRIGGER cascade_soft_delete_generators
AFTER UPDATE ON generators
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cascade_soft_delete_generators();

-- Function for handling hard delete of generators
CREATE OR REPLACE FUNCTION handle_generator_hard_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- -- Delete stacks associated with the generator
    -- DELETE FROM stacks
    -- WHERE generator_id = OLD.id;

    -- -- Delete deployment_objects associated with the generator
    -- DELETE FROM deployment_objects
    -- WHERE generator_id = OLD.id;

    -- RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Trigger for handling hard delete of generators
CREATE TRIGGER trigger_generator_hard_delete
BEFORE DELETE ON generators
FOR EACH ROW
EXECUTE FUNCTION handle_generator_hard_delete();
