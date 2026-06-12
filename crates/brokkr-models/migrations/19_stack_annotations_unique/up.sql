-- Migration: enforce UNIQUE (stack_id, key) on stack_annotations
-- BROKKR-T-0223: stack_annotations lacked the (stack_id, key) uniqueness that
-- agent_annotations and template_annotations already enforce. Without it,
-- duplicate keys accumulated silently and stack/template matching then saw an
-- arbitrary one-of-N value for a given key.

-- Deduplicate any existing rows before adding the constraint. Keep-rule: the
-- row with the greatest id per (stack_id, key) wins. stack_annotations has no
-- created_at column, so greatest-id is the deterministic tie-breaker; the
-- value of a duplicate key was already arbitrary, so this loses no meaningful
-- information.
DELETE FROM stack_annotations a
USING stack_annotations b
WHERE a.stack_id = b.stack_id
  AND a.key = b.key
  AND a.id < b.id;

-- Enforce uniqueness going forward (mirrors agent_annotations.UNIQUE(agent_id, key)).
ALTER TABLE stack_annotations
    ADD CONSTRAINT unique_stack_annotation UNIQUE (stack_id, key);
