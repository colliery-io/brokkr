CREATE TABLE stacks (
    name VARCHAR(255) NOT NULL,
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description TEXT,
    labels JSONB,
    annotations JSONB,
    agent_target JSONB,
    CONSTRAINT unique_stack_name UNIQUE (name)
) INHERITS (base_table);

CREATE INDEX idx_stack_id ON stacks(id);
CREATE INDEX idx_stack_name ON stacks (name);
CREATE INDEX idx_stack_labels ON stacks USING gin (labels);
CREATE INDEX idx_stack_annotations ON stacks USING gin (annotations);
CREATE INDEX idx_stack_agent_target ON stacks USING gin (agent_target);

CREATE TRIGGER update_stack_timestamp
BEFORE UPDATE ON stacks
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER soft_delete_stacks
BEFORE UPDATE ON stacks
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION soft_delete();

CREATE VIEW active_stacks AS
SELECT * FROM stacks WHERE deleted_at IS NULL;

