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


-- Create labels table
CREATE TABLE labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_object_id UUID NOT NULL,
    label VARCHAR(255) NOT NULL,
    UNIQUE (external_object_id, label)
);

-- Create indexes for labels
CREATE INDEX idx_labels_object ON labels (external_object_id);
CREATE INDEX idx_labels_label ON labels (label);

-- Create annotations table
CREATE TABLE annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_object_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    UNIQUE (external_object_id, key)
);

-- Create indexes for labels
CREATE INDEX idx_annotations_object ON annotations (external_object_id);
CREATE INDEX idx_annotations_label ON annotations (key);

