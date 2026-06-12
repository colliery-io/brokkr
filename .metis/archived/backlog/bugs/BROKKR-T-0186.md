---
id: cli-rotate-agent-generator
level: task
title: "CLI rotate agent/generator discards the new PAK — credential unrecoverable"
short_code: "BROKKR-T-0186"
created_at: 2026-06-10T03:03:53.125495+00:00
updated_at: 2026-06-10T11:19:05.602248+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# CLI rotate agent/generator discards the new PAK — credential unrecoverable

## Objective

`brokkr-broker rotate agent --uuid <id>` and `rotate generator --uuid <id>` generate a new PAK, store only its hash, and never output the key (`crates/brokkr-broker/src/cli/commands.rs:206-250`: `let new_pak_hash = utils::pak::create_pak()?.1;`). After CLI rotation nobody possesses the credential — the entity is locked out until rotated again via the REST API. The API endpoints (`POST /api/v1/{agents,generators}/{id}/rotate-pak`) return the PAK and are unaffected.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] CLI rotation outputs the new PAK (stdout, consistent with `create agent`/`create generator`) or is removed in favor of the API
- [ ] Test covering the output
- [ ] `docs/src/how-to/pak-management.md` and `docs/src/reference/cli.md` warnings updated to match new behavior

## Status Updates

- 2026-06-09: Found during /docs-diataxis accuracy review; docs currently warn users away from the CLI path.
- 2026-06-09: IMPLEMENTED (uncommitted, unit tests green): `rotate agent`/`rotate generator` now print `New PAK:` to stdout like `create`; docs (pak-management, cli reference, sdks README) updated to match, including the auth-cache-TTL caveat for CLI rotation.
- 2026-06-10 (closure pass): rotate_agent_key/rotate_generator_key now return the PAK; integration tests added (tests/integration/cli.rs) asserting the returned PAK verifies against the stored hash and the CLI audit row (details.via=cli) exists.