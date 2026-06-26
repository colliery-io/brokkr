-- Migration: back-fill generator registrations from existing agent_targets (BROKKR-T-0248)
--
-- For each existing agent that already has agent_targets rows pointing at a
-- generator's stacks, insert a registration so the new enforcement gate
-- (authorize_target_mutation) does not break existing deployments.
--
-- System-generator registrations are NOT back-filled here because the system
-- generator does not exist yet at migration time — it is provisioned by
-- provision_system_generator() immediately after migrations complete, which
-- also registers all existing agents at that point.
--
-- ON CONFLICT DO NOTHING makes this idempotent: safe to run multiple times.

INSERT INTO agent_generator_registrations (id, agent_id, generator_id)
SELECT DISTINCT gen_random_uuid(), at.agent_id, s.generator_id
FROM   agent_targets at
JOIN   stacks s ON s.id = at.stack_id
ON CONFLICT (agent_id, generator_id) DO NOTHING;
