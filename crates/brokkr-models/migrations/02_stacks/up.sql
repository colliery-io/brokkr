CREATE TABLE stacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    CONSTRAINT unique_stack_name UNIQUE (name)
);

CREATE INDEX idx_stack_id ON stacks(id);
CREATE INDEX idx_stack_name ON stacks (name);


-- Create targets table
CREATE TABLE agent_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stack_id UUID NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (stack_id, agent_id)
);

CREATE INDEX idx_agent_targets_stack_id ON agent_targets(stack_id);
CREATE INDEX idx_agent_targets_agent_id ON agent_targets(agent_id);



CREATE TRIGGER update_stack_timestamp
BEFORE UPDATE ON stacks
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Function to handle stack soft deletion
CREATE OR REPLACE FUNCTION handle_stack_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Soft delete all existing deployment objects for the stack
    UPDATE deployment_objects
    SET deleted_at = NEW.deleted_at
    WHERE stack_id = NEW.id AND deleted_at IS NULL;

    -- Insert a new deployment object with blank content as a deletion marker
    INSERT INTO deployment_objects (
        stack_id, yaml_content, yaml_checksum,
        submitted_at, is_deletion_marker
    ) VALUES (
        NEW.id, '', md5(''),
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

CREATE OR REPLACE FUNCTION handle_stack_hard_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete labels associated with the stack
    DELETE FROM labels
    WHERE object_id = OLD.id AND object_type = 'stack';

    -- Delete annotations associated with the stack
    DELETE FROM annotations
    WHERE object_id = OLD.id AND object_type = 'stack';

    -- Delete agent_target rows associated with the stack
    DELETE FROM agent_targets
    WHERE stack_id = OLD.id;

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

-- Trigger for stack hard deletion
CREATE TRIGGER trigger_stack_hard_delete
BEFORE DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION handle_stack_hard_delete();