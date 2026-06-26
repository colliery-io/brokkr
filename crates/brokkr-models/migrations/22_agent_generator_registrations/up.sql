-- Migration: agent generator registrations (BROKKR-T-0240)
--
-- Subscription join table: an agent must be registered with a generator before
-- any of that generator's stacks can be targeted at it. Agents opt in; the
-- broker enforces the check in authorize_target_mutation.
--
-- FK cascades ensure registrations are cleaned up automatically when either
-- the agent or generator is hard-deleted.
--
-- The UNIQUE constraint on (agent_id, generator_id) serves double duty as the
-- index backing the hot-path is_registered() lookup — no separate unique index
-- is needed.

CREATE TABLE agent_generator_registrations (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id    UUID        NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    generator_id UUID       NOT NULL REFERENCES generators(id) ON DELETE CASCADE,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (agent_id, generator_id)
);

CREATE INDEX idx_agr_agent_id     ON agent_generator_registrations (agent_id);
CREATE INDEX idx_agr_generator_id ON agent_generator_registrations (generator_id);
