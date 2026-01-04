-- Fix the generator soft-delete cascade trigger
-- Bug: Original used WHERE id = NEW.id instead of WHERE generator_id = NEW.id
-- This prevented proper cascade of soft deletes to child stacks and deployment objects

CREATE OR REPLACE FUNCTION cascade_soft_delete_generators()
RETURNS TRIGGER AS $$
BEGIN
    -- Cascade soft delete to stacks owned by this generator
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;

    -- Cascade soft delete to deployment objects in those stacks
    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE stack_id IN (
        SELECT id FROM stacks WHERE generator_id = NEW.id
    ) AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
