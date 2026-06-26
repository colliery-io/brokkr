-- Rollback: drop system generator flag.

ALTER TABLE generators
    DROP COLUMN IF EXISTS is_system;
