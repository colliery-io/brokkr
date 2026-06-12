---
id: config-file-layer-never-wired-into
level: task
title: "Config-file layer never wired into shipped binaries (Settings::new(None))"
short_code: "BROKKR-T-0187"
created_at: 2026-06-10T03:04:02.733075+00:00
updated_at: 2026-06-10T04:57:59.330663+00:00
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

# Config-file layer never wired into shipped binaries (Settings::new(None))

## Objective

The optional config-file layer in `brokkr_utils::config::Settings` is never reachable in the shipped binaries: both call `Settings::new(None)` (`crates/brokkr-broker/src/bin.rs:28`, `crates/brokkr-agent/src/cli/commands.rs:75`). `BROKKR_CONFIG_FILE` only arms the change watcher (`utils/config_watcher.rs`), and the watcher/manual reload path re-reads with `config_file = None`, so file contents are never applied. Consequences: documented config-file workflows silently no-op (the prior multi-tenant guide's file-based `database.schema` would have run both tenants against `public`).

Decide: wire `BROKKR_CONFIG_FILE` (and/or a `--config` flag) into `Settings::new` and the reload path, or remove the watcher/env surface entirely.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Config file loading works end to end (startup + hot reload), or the surface is removed
- [ ] Integration test covering whichever behavior is chosen
- [ ] `docs/src/getting-started/configuration.md` and `docs/src/reference/cli.md` updated (both currently document the limitation)

## Status Updates

- 2026-06-09: Found during /docs-diataxis accuracy review.
- 2026-06-09: IMPLEMENTED (uncommitted, unit tests green): `BROKKR_CONFIG_FILE` now feeds `Settings::new(...)` in both binaries and the broker reload path (`ReloadableConfig::from_settings(.., env)`), so the file layer works at startup and on hot reload. Docs updated (configuration sources now three layers; cli + env-vars references). Remaining AC: integration test.
- 2026-06-10 (closure pass): added brokkr-utils integration test exercising the exact binary load expression (BROKKR_CONFIG_FILE env → Settings::new) — passing. Also fixed pre-existing stale assertion in test_settings_default (5432 → 5433).