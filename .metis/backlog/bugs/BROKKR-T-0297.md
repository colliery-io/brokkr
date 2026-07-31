---
id: rotate-admin-ignores-brokkr
level: task
title: "rotate admin ignores BROKKR__DATABASE__SCHEMA — per-tenant admin rotation hits the public schema in schema-isolated installs"
short_code: "BROKKR-T-0297"
created_at: 2026-07-27T14:48:57.346383+00:00
updated_at: 2026-07-28T00:17:06.406147+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# rotate admin ignores BROKKR__DATABASE__SCHEMA — per-tenant admin rotation hits the public schema in schema-isolated installs

## Objective

Fix schema handling in the CLI rotation path. `rotate_admin` (`crates/brokkr-broker/src/cli/commands.rs:~235`) connects with a bare `PgConnection::establish(&config.database.url)` and never applies `BROKKR__DATABASE__SCHEMA` (no `search_path` set), unlike `serve`, which configures the schema on its pool. In a schema-per-tenant install (the pattern documented in `reference/multi-tenancy.md` / `how-to/multi-tenant-setup.md`), running `brokkr-broker rotate admin` "with that tenant's configuration" therefore operates on the `public` schema: at best it fails to find the tenant's `admin_role` row, at worst it rotates a different tenant's admin credential. The same bare-connection pattern likely affects `rotate agent`/`rotate generator` and `create agent`/`create generator` — audit all CLI subcommands that open their own connection.

Found 2026-07-27 while correcting multi-tenant-setup.md's recovery guidance (BROKKR-T-0286), which now documents `rotate admin` as the lost-credential recovery path — making this gap consumer-visible.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (downgraded 2026-07-27: per Dylan, schema-per-tenant is NOT the tenancy model — tenants are at the generator/agent level. `database.schema` is a general deployment/config feature, so this is a config-correctness bug in the CLI, not a cross-tenant hazard. Still worth fixing: any CLI subcommand run against a schema-configured install silently operates on `public`.)

### Impact Assessment
- **Affected Users**: schema-isolated multi-tenant operators using any CLI create/rotate subcommand with `BROKKR__DATABASE__SCHEMA` set.
- **Reproduction sketch**: two schemas A/B each migrated; set `BROKKR__DATABASE__SCHEMA=tenant_a`; run `rotate admin`; inspect which schema's `admin_role` row changed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All CLI subcommands honor `BROKKR__DATABASE__SCHEMA` (set `search_path` on their connection identically to `serve`).
- [ ] Integration test covering `rotate admin` against a non-public schema (`angreal tests integration brokkr-broker`).
- [ ] multi-tenant-setup.md recovery guidance re-checked against the fixed behavior.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`.

**The ticket's speculation was wrong; scope is narrower than filed.** `rotate_admin` was the *only* affected subcommand. `rotate_agent_key`, `rotate_generator_key`, `create_agent`, and `create_generator` already passed `config.database.schema.as_deref()` into `create_shared_connection_pool`, and the DAL routes every connection through `ConnectionPool::get()`, which issues `SET search_path TO <schema>, public` per checkout. `generate_pak` opens no connection at all.

Fix: new `connection_pool_from_settings(config, max_size)` helper in `cli/commands.rs`, used by all six call sites including `serve` (byte-identical behavior). Kept in `commands.rs` rather than `db.rs` deliberately — every caller lives there and `db.rs` has no dependency on `Settings`, so moving it would couple the db layer to config for no gain. The doc comment notes the schema is applied per checkout, so callers must not reach past it to `pool.pool.get()`.

Tests: new `tests/integration/db/cli_schema.rs` — `test_rotate_admin_writes_to_configured_schema` provisions two non-`public` schemas, seeds distinct sentinel hashes, calls `rotate_admin` against schema A, asserts A rotated while B and `public` are untouched (a genuine end-to-end call, not a helper proxy); plus `test_connection_pool_from_settings_applies_configured_schema`. Clippy clean, integration target compiles. **Compile-verified only** — no Postgres in the agent's environment; they run with the full suite before commit.

**Second bug found, filed as BROKKR-T-0306:** `create_shared_connection_pool` calls `url.set_path("brokkr")`, discarding the database name from `database.url`. The old bare `PgConnection::establish` honored it, so routing through the helper is technically a behavior change — but `serve` has always hardcoded the same name, so any install where `serve` works already uses a database called `brokkr`. No practical regression; the hardcoded name is a pre-existing wart affecting everything equally.

**Acceptance criterion 3** — verified rather than edited: `how-to/multi-tenant-setup.md` documents `rotate admin` run "with that tenant's configuration", which is now accurate rather than aspirational.