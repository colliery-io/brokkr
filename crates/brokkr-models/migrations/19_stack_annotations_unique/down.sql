-- Rollback: drop the (stack_id, key) uniqueness constraint on stack_annotations.
-- Deduplicated rows are not restored (the duplicates carried arbitrary values).
ALTER TABLE stack_annotations DROP CONSTRAINT IF EXISTS unique_stack_annotation;
