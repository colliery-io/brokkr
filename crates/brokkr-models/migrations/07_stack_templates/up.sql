-- Stack Templates Migration
-- Part of BROKKR-I-0002: Stack Templating System

-- =============================================================================
-- Core template table
-- =============================================================================
CREATE TABLE stack_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,

    -- Ownership: NULL = system template (admin-only), non-NULL = generator-owned
    generator_id UUID REFERENCES generators(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    description TEXT,
    version INTEGER NOT NULL DEFAULT 1,

    -- Template content (Tera syntax) and parameters (JSON Schema as TEXT)
    template_content TEXT NOT NULL,
    parameters_schema TEXT NOT NULL,

    -- Integrity check
    checksum VARCHAR(64) NOT NULL,

    -- Version uniqueness: (generator_id, name, version) must be unique
    -- NULL generator_id is treated as distinct value (system templates)
    CONSTRAINT unique_template_version UNIQUE NULLS NOT DISTINCT (generator_id, name, version)
);

-- Indexes for performance
CREATE INDEX idx_stack_templates_generator ON stack_templates(generator_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_stack_templates_name ON stack_templates(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_stack_templates_deleted_at ON stack_templates(deleted_at);

-- Timestamp trigger
CREATE TRIGGER update_stack_templates_timestamp
BEFORE UPDATE ON stack_templates
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- =============================================================================
-- Template labels (mirrors stack_labels pattern)
-- =============================================================================
CREATE TABLE template_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    label VARCHAR(64) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_id, label),
    CONSTRAINT fk_template_labels_template FOREIGN KEY (template_id)
        REFERENCES stack_templates(id) ON DELETE CASCADE
);

CREATE INDEX idx_template_labels_template ON template_labels(template_id);
CREATE INDEX idx_template_labels_label ON template_labels(label);

-- =============================================================================
-- Template annotations (mirrors stack_annotations pattern)
-- =============================================================================
CREATE TABLE template_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    key VARCHAR(64) NOT NULL,
    value VARCHAR(64) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_id, key),
    CONSTRAINT fk_template_annotations_template FOREIGN KEY (template_id)
        REFERENCES stack_templates(id) ON DELETE CASCADE
);

CREATE INDEX idx_template_annotations_template ON template_annotations(template_id);
CREATE INDEX idx_template_annotations_key ON template_annotations(key);

-- =============================================================================
-- Template targets (mirrors agent_targets pattern)
-- Pre-computed template->stack compatibility for efficient querying
-- =============================================================================
CREATE TABLE template_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    stack_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_template_targets_template FOREIGN KEY (template_id)
        REFERENCES stack_templates(id) ON DELETE CASCADE,
    CONSTRAINT fk_template_targets_stack FOREIGN KEY (stack_id)
        REFERENCES stacks(id) ON DELETE CASCADE,
    CONSTRAINT unique_template_stack UNIQUE (template_id, stack_id)
);

CREATE INDEX idx_template_targets_template ON template_targets(template_id);
CREATE INDEX idx_template_targets_stack ON template_targets(stack_id);

-- =============================================================================
-- Rendered deployment objects (provenance tracking)
-- One-to-one relationship with deployment_objects
-- =============================================================================
CREATE TABLE rendered_deployment_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_object_id UUID NOT NULL UNIQUE,
    template_id UUID NOT NULL,
    template_version INTEGER NOT NULL,
    template_parameters TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_rendered_deployment_object FOREIGN KEY (deployment_object_id)
        REFERENCES deployment_objects(id) ON DELETE CASCADE,
    CONSTRAINT fk_rendered_template FOREIGN KEY (template_id)
        REFERENCES stack_templates(id)
);

CREATE INDEX idx_rendered_deployment_objects_template ON rendered_deployment_objects(template_id, template_version);
CREATE INDEX idx_rendered_deployment_objects_deployment ON rendered_deployment_objects(deployment_object_id);

-- =============================================================================
-- Cascade soft delete when generator is deleted
-- =============================================================================
CREATE OR REPLACE FUNCTION cascade_soft_delete_generator_templates()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE stack_templates
    SET deleted_at = NEW.deleted_at
    WHERE generator_id = NEW.id AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cascade_soft_delete_generator_templates
AFTER UPDATE OF deleted_at ON generators
FOR EACH ROW
WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
EXECUTE FUNCTION cascade_soft_delete_generator_templates();

-- =============================================================================
-- Handle hard delete of stack_templates
-- =============================================================================
CREATE OR REPLACE FUNCTION handle_stack_template_hard_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete template labels
    DELETE FROM template_labels WHERE template_id = OLD.id;

    -- Delete template annotations
    DELETE FROM template_annotations WHERE template_id = OLD.id;

    -- Delete template targets
    DELETE FROM template_targets WHERE template_id = OLD.id;

    -- Note: rendered_deployment_objects references are kept for audit trail
    -- They reference the template_id but don't block deletion

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_stack_template_hard_delete
BEFORE DELETE ON stack_templates
FOR EACH ROW
EXECUTE FUNCTION handle_stack_template_hard_delete();
