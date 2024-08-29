-- Common functions
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    NEW.deleted_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Labels table
CREATE TABLE labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL,
    object_type VARCHAR(50) NOT NULL,
    label VARCHAR(255) NOT NULL,
    UNIQUE (object_id, object_type, label)
);

CREATE INDEX idx_labels_object ON labels (object_id, object_type);
CREATE INDEX idx_labels_label ON labels (label);

-- Annotations table
CREATE TABLE annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL,
    object_type VARCHAR(50) NOT NULL,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    UNIQUE (object_id, object_type, key)
);

CREATE INDEX idx_annotations_object ON annotations (object_id, object_type);
CREATE INDEX idx_annotations_key ON annotations (key);

