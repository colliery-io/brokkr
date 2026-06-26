-- Rollback: remove all registrations created by the back-fill.
-- At this point in migration history no registrations exist from any other
-- source, so truncating the table is safe.

TRUNCATE agent_generator_registrations;
