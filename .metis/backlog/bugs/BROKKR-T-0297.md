---
id: rotate-admin-ignores-brokkr
level: task
title: "rotate admin ignores BROKKR__DATABASE__SCHEMA — per-tenant admin rotation hits the public schema in schema-isolated installs"
short_code: "BROKKR-T-0297"
created_at: 2026-07-27T14:48:57.346383+00:00
updated_at: 2026-07-27T14:48:57.346383+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


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

- [ ] All CLI subcommands honor `BROKKR__DATABASE__SCHEMA` (set `search_path` on their connection identically to `serve`).
- [ ] Integration test covering `rotate admin` against a non-public schema (`angreal tests integration brokkr-broker`).
- [ ] multi-tenant-setup.md recovery guidance re-checked against the fixed behavior.

## Status Updates

*To be added during implementation*
