-- Migration: 09_work_order_targeting
-- Description: Add label and annotation targeting for work orders
-- Pattern: Mirrors stack targeting system (labels, annotations, hard targets)

-- =============================================================================
-- WORK ORDER LABELS TABLE
-- =============================================================================
-- Labels that a work order is targeting. Work orders with labels will be
-- matched to agents that have ANY of the specified labels (OR logic).
CREATE TABLE work_order_labels (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key to work order
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,

    -- Label value (max 64 characters, no whitespace)
    label VARCHAR(64) NOT NULL,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint: one entry per work_order/label pair
    UNIQUE(work_order_id, label)
);

-- Indexes for work_order_labels
CREATE INDEX idx_work_order_labels_order ON work_order_labels(work_order_id);
CREATE INDEX idx_work_order_labels_label ON work_order_labels(label);

-- =============================================================================
-- WORK ORDER ANNOTATIONS TABLE
-- =============================================================================
-- Annotations (key-value pairs) that a work order is targeting. Work orders
-- with annotations will be matched to agents that have ANY of the specified
-- key-value pairs (OR logic).
CREATE TABLE work_order_annotations (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key to work order
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,

    -- Annotation key-value pair (max 64 characters each, no whitespace)
    key VARCHAR(64) NOT NULL,
    value VARCHAR(64) NOT NULL,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint: one entry per work_order/key/value triple
    UNIQUE(work_order_id, key, value)
);

-- Indexes for work_order_annotations
CREATE INDEX idx_work_order_annotations_order ON work_order_annotations(work_order_id);
CREATE INDEX idx_work_order_annotations_key ON work_order_annotations(key);
CREATE INDEX idx_work_order_annotations_key_value ON work_order_annotations(key, value);

-- =============================================================================
-- COMMENTS
-- =============================================================================
COMMENT ON TABLE work_order_labels IS 'Labels that work orders target; matched to agent labels via OR logic';
COMMENT ON TABLE work_order_annotations IS 'Annotations that work orders target; matched to agent annotations via OR logic';

COMMENT ON COLUMN work_order_labels.label IS 'Label value (max 64 chars, no whitespace)';
COMMENT ON COLUMN work_order_annotations.key IS 'Annotation key (max 64 chars, no whitespace)';
COMMENT ON COLUMN work_order_annotations.value IS 'Annotation value (max 64 chars, no whitespace)';
