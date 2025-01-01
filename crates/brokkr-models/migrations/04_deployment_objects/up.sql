CREATE TABLE deployment_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    sequence_id BIGSERIAL,
    stack_id UUID NOT NULL,
    yaml_content TEXT NOT NULL,
    yaml_checksum TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deletion_marker BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT fk_stack FOREIGN KEY (stack_id) REFERENCES stacks(id),
    CONSTRAINT deployment_objects_sequence_id_key UNIQUE (sequence_id)
);

CREATE INDEX idx_deployment_objects_id ON deployment_objects(id);
CREATE INDEX idx_deployment_objects_stack_id ON deployment_objects (stack_id);
CREATE INDEX idx_deployment_objects_yaml_checksum ON deployment_objects (yaml_checksum);
CREATE INDEX idx_deployment_objects_deleted_at ON deployment_objects(deleted_at);
CREATE INDEX idx_deployment_objects_is_deletion_marker ON deployment_objects (is_deletion_marker);

CREATE TRIGGER update_deployment_object_timestamp
BEFORE UPDATE ON deployment_objects
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Function to prevent deployment object changes
CREATE OR REPLACE FUNCTION prevent_deployment_object_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        -- Allow deletions (they will be handled by the stack deletion trigger)
        RETURN OLD;
    ELSIF (NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL) THEN
        -- Allow setting deleted_at (soft delete)
        RETURN NEW;
    ELSIF (NEW.is_deletion_marker AND OLD.is_deletion_marker) THEN
        -- Allow updates to deletion markers
        RETURN NEW;
    ELSE
        RAISE EXCEPTION 'Deployment objects cannot be modified except for soft deletion or updating deletion markers';
    END IF;
end;
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
        SELECT id
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
BEFORE DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION hard_delete_deployment_objects_on_stack_delete();
