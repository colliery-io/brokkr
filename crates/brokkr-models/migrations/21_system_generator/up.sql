-- Migration: system generator flag (BROKKR-T-0239)
--
-- Marks one generator as the system generator — provisioned idempotently at
-- broker startup and auto-registered with every agent. Fleet-management stacks
-- live here and reach all agents without any per-agent opt-in.
--
-- Defaults to false so all existing generators are unaffected.

ALTER TABLE generators
    ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT false;
