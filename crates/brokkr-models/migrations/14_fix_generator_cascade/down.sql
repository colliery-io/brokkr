-- Revert to original (buggy) trigger - for rollback only
-- WARNING: This restores the bug where soft deletes don't cascade properly

CREATE OR REPLACE FUNCTION cascade_soft_delete_generators()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE stacks
    SET deleted_at = NEW.deleted_at
    WHERE id = NEW.id AND deleted_at IS NULL;

    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE id = NEW.id AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
