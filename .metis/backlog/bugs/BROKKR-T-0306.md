---
id: create-shared-connection-pool
level: task
title: "create_shared_connection_pool discards the database name from database.url and always connects to 'brokkr'"
short_code: "BROKKR-T-0306"
created_at: 2026-07-28T00:16:12.719642+00:00
updated_at: 2026-07-31T05:23:34.113814+00:00
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

# create_shared_connection_pool discards the database name from database.url and always connects to 'brokkr'

## Objective

Make the broker connect to the database the operator configured, instead of silently rewriting it to `brokkr`.

`create_shared_connection_pool` (`crates/brokkr-broker/src/db.rs:42-65`) takes the configured URL and immediately overwrites its path:

```rust
let mut url = Url::parse(base_url).expect("Invalid base URL");
url.set_path(database_name);
```

Every production caller reaches it through `connection_pool_from_settings` (`cli/commands.rs:56-63`), which passes the string literal `"brokkr"`. So whatever database name appears in `BROKKR__DATABASE__URL` is parsed, discarded, and replaced.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (a documented, chart-exposed setting is silently ignored; the failure is either a confusing connection error or, worse, successful use of the wrong database)

## 2026-07-29 — verified against the code

Confirmed by reading `db.rs:49-50` and `cli/commands.rs:56-63`. The title was the only content this ticket had — it was an unedited template otherwise — and the claim holds exactly as written.

**The chart exposes the setting this bug ignores.** `_helpers.tpl:87-93` defines `brokkr-broker.databaseName` as `postgresql.auth.database` (bundled) or `postgresql.external.database` (external), and `templates/configmap.yaml:17` renders it into `BROKKR__DATABASE__URL`. Both default to `brokkr`, which is why nobody has hit this: the default masks it completely.

An operator pointing at an existing external Postgres with any other database name — the entire point of `postgresql.external` — gets one of two outcomes:

- **No database named `brokkr` on that server** → connection failure naming a database the operator never configured. Loud, but misleading.
- **A database named `brokkr` does exist** → the broker silently runs against the wrong one. Migrations, agents, stacks and audit logs all land somewhere the operator is not looking.

The second is the dangerous case, and it is plausible on a shared Postgres already hosting another Brokkr install.

**Related to but distinct from BROKKR-T-0297.** That was the `search_path`/schema component being lost on bare connections; this is the *database* component of the same URL. Both stem from the broker not simply using the URL it was given.

### Technical Approach

Use the URL as configured. `set_path` should apply only when a caller genuinely needs to override the database — the test fixtures do, to target per-test databases — so the sensible shape is to make the override optional:

- `create_shared_connection_pool(base_url, database_name: Option<&str>, ...)`, applying `set_path` only for `Some`.
- `connection_pool_from_settings` passes `None`, so production honours `database.url` verbatim.
- Test fixtures keep passing an explicit name.

Worth deciding at the same time whether `Url::parse(...).expect("Invalid base URL")` should stop panicking: a typo in `BROKKR__DATABASE__URL` currently aborts the process without naming the offending value.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A `BROKKR__DATABASE__URL` naming a non-`brokkr` database connects to that database.
- [ ] A test asserts the resolved connection URL retains the configured database name — not merely that a pool is constructed.
- [ ] Test fixtures that deliberately target a different database still work (the override path is kept, just no longer forced on production).
- [ ] A chart install with `postgresql.external.database` set to a non-default name reaches that database.
- [ ] Schema handling from BROKKR-T-0297 is verified unaffected — both live on the same connection setup.

## Status Updates

**2026-07-30 — FIXED** on branch `docs/tenancy-review-2026-07` (commit `737c7dd`). 531 integration tests pass, 151 unit (4 new). Details in the commit message and in BROKKR-T-0315's neighbours; this ticket was one of two unedited template shells whose titles carried the entire finding, and both turned out to be real.