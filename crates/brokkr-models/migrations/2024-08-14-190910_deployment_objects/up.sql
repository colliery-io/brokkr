CREATE TABLE deployment_objects (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id BIGSERIAL,
    stack_id UUID NOT NULL,
    yaml_content TEXT NOT NULL,
    yaml_checksum TEXT NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deletion_marker BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT fk_stack FOREIGN KEY (stack_id) REFERENCES stacks(id),
    CONSTRAINT unique_yaml_per_stack UNIQUE (stack_id, yaml_checksum),
    CONSTRAINT deployment_objects_sequence_id_key UNIQUE (sequence_id)
);

-- Create indexes
CREATE INDEX idx_deployment_objects_stack_id ON deployment_objects (stack_id);
CREATE INDEX idx_deployment_objects_yaml_checksum ON deployment_objects (yaml_checksum);
CREATE INDEX idx_deployment_objects_deleted_at ON deployment_objects(deleted_at);
CREATE INDEX idx_deployment_objects_is_deletion_marker ON deployment_objects (is_deletion_marker);

-- Function to handle stack soft deletion
CREATE OR REPLACE FUNCTION handle_stack_soft_delete()
RETURNS TRIGGER AS $$
DECLARE
    new_uuid UUID;
BEGIN
    -- Soft delete all existing deployment objects for the stack
    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE stack_id = NEW.id AND deleted_at IS NULL;

    -- Generate a new UUID for the deletion marker deployment object
    new_uuid := gen_random_uuid();

    -- Insert a new deployment object with blank content as a deletion marker
    INSERT INTO deployment_objects (
        uuid, stack_id, yaml_content, yaml_checksum,
        submitted_at, is_deletion_marker
    ) VALUES (
        new_uuid, NEW.id, '', md5(''),
        NEW.deleted_at, TRUE
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for handling stack soft deletion
CREATE TRIGGER trigger_handle_stack_soft_delete
AFTER UPDATE OF deleted_at ON stacks
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION handle_stack_soft_delete();

-- Function to prevent deployment object changes
CREATE OR REPLACE FUNCTION prevent_deployment_object_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL) THEN
        -- Allow setting deleted_at (soft delete)
        RETURN NEW;
    ELSIF (NEW.is_deletion_marker AND OLD.is_deletion_marker) THEN
        -- Allow updates to deletion markers
        RETURN NEW;
    ELSE
        RAISE EXCEPTION 'Deployment objects cannot be modified except for soft deletion or updating deletion markers';
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Triggers to prevent deployment object changes
CREATE TRIGGER prevent_deployment_object_update
BEFORE UPDATE ON deployment_objects
FOR EACH ROW
EXECUTE FUNCTION prevent_deployment_object_changes();

CREATE TRIGGER prevent_deployment_object_delete
BEFORE DELETE ON deployment_objects
FOR EACH ROW
EXECUTE FUNCTION prevent_deployment_object_changes();

-- Function to handle hard deletion of deployment objects
CREATE OR REPLACE FUNCTION hard_delete_deployment_objects_on_stack_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete all agent events associated with the deleted deployment objects
    DELETE FROM agent_events
    WHERE deployment_object_id IN (
        SELECT uuid 
        FROM deployment_objects 
        WHERE stack_id = OLD.id
    );
    
    -- Delete all deployment objects for the stack, including the deletion marker
    DELETE FROM deployment_objects
    WHERE stack_id = OLD.id;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Trigger for hard deleting deployment objects
CREATE TRIGGER trigger_hard_delete_deployment_objects
AFTER DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION hard_delete_deployment_objects_on_stack_delete();