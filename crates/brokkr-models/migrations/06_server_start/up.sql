-- Your SQL goes here
CREATE TABLE IF NOT EXISTS app_initialization (
    id SERIAL PRIMARY KEY,
    initialized_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE admin_role (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    pak_hash TEXT NOT NULL
);

CREATE INDEX idx_admin_role_id ON admin_role(id);

-- Trigger to update the timestamp
CREATE TRIGGER update_admin_role_timestamp
BEFORE UPDATE ON admin_role
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Function to prevent hard deletes
CREATE OR REPLACE FUNCTION prevent_admin_role_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Deleting from admin_role table is not allowed';
END;
$$ LANGUAGE plpgsql;

-- Trigger to prevent hard deletes
CREATE TRIGGER prevent_admin_role_delete
BEFORE DELETE ON admin_role
FOR EACH ROW
EXECUTE FUNCTION prevent_admin_role_delete();