-- Existing stacks table creation (no changes)
CREATE TABLE stacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    CONSTRAINT unique_stack_name UNIQUE (name)
);

-- Existing indexes (no changes)
CREATE INDEX idx_stack_id ON stacks(id);
CREATE INDEX idx_stack_name ON stacks (name);

-- Existing trigger (no changes)
CREATE TRIGGER update_stack_timestamp
BEFORE UPDATE ON stacks
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Existing function and trigger for soft deletion (no changes)
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

CREATE TRIGGER trigger_handle_stack_soft_delete
AFTER UPDATE OF deleted_at ON stacks
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION handle_stack_soft_delete();

-- Updated function for hard deletion (includes new cascading deletes)
CREATE OR REPLACE FUNCTION handle_stack_hard_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete labels associated with the stack
    DELETE FROM stack_labels
    WHERE object_id = OLD.id AND object_type = 'stack';

    -- Delete annotations associated with the stack
    DELETE FROM stack_annotations
    WHERE object_id = OLD.id AND object_type = 'stack';

    -- Delete stack_labels associated with the stack
    DELETE FROM stack_labels
    WHERE stack_id = OLD.id;

    -- Delete stack_annotations associated with the stack
    DELETE FROM stack_annotations
    WHERE stack_id = OLD.id;

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

-- Existing trigger for stack hard deletion (no changes)
CREATE TRIGGER trigger_stack_hard_delete
BEFORE DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION handle_stack_hard_delete();

-- New function and trigger to cascade delete stack_labels
CREATE OR REPLACE FUNCTION cascade_delete_stack_labels()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM stack_labels WHERE stack_id = OLD.id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cascade_delete_stack_labels
BEFORE DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION cascade_delete_stack_labels();

-- New function and trigger to cascade delete stack_annotations
CREATE OR REPLACE FUNCTION cascade_delete_stack_annotations()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM stack_annotations WHERE stack_id = OLD.id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cascade_delete_stack_annotations
BEFORE DELETE ON stacks
FOR EACH ROW
EXECUTE FUNCTION cascade_delete_stack_annotations();

-- Existing stack_labels table creation (no changes)
CREATE TABLE stack_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stack_id UUID NOT NULL,
    label VARCHAR(255) NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (stack_id, label)
);

CREATE INDEX idx_stack_labels_object ON stack_labels (stack_id);
CREATE INDEX idx_stack_labels_label ON stack_labels (label);

-- Existing stack_annotations table creation (no changes)
CREATE TABLE stack_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stack_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (stack_id, key)
);

CREATE INDEX idx_stack_annotations_object ON stack_annotations (stack_id);
CREATE INDEX idx_stack_annotations_key ON stack_annotations (key);

-- Create the agent_targets table
CREATE TABLE agent_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    stack_id UUID NOT NULL,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    CONSTRAINT fk_stack FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE,
    CONSTRAINT unique_agent_stack UNIQUE (agent_id, stack_id)
);

-- Create indexes
CREATE INDEX idx_agent_targets_agent_id ON agent_targets (agent_id);
CREATE INDEX idx_agent_targets_stack_id ON agent_targets (stack_id);
