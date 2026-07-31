---
id: docs-cli-md-generate-pak-example
level: task
title: "Docs: cli.md generate-pak example shows sha256:-prefixed hash the broker rejects; serve endpoint table stale"
short_code: "BROKKR-T-0281"
created_at: 2026-07-27T14:19:53.937122+00:00
updated_at: 2026-07-27T14:19:53.937122+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: cli.md generate-pak example shows sha256:-prefixed hash the broker rejects; serve endpoint table stale

## Objective

Fix CLI reference errors that break the day-zero bootstrap path (2026-07-27 auth-drift sweep, code-verified):

1. **Major — generate-pak output example (~lines 149-159)** shows the hash as `sha256:9f86...`. The command prints a bare 64-char hex hash (`crates/brokkr-broker/src/cli/commands.rs:277-278`), and a `sha256:`-prefixed value set as `BROKKR__BROKER__PAK_HASH` is **rejected at startup** — `validate_pak_hash` requires exactly 64 hex chars (`crates/brokkr-broker/src/utils/mod.rs:169-173`, "Invalid PAK hash provided in configuration"). A consumer who pattern-matches the documented format locks themselves out of first boot. `pak-management.md:23` shows the correct bare form — the two pages contradict.
2. **Minor — serve "Endpoints exposed" table (~lines 19-27)** omits the operator console served at `/` (embed-ui builds; `api/mod.rs:276-279`, `docker/Dockerfile.broker:23`) and the `/docs/openapi.json` route (`api/v1/openapi.rs:336-337`).
3. **Minor — pak-management.md day-zero note (~line 47)** points to the CLI reference "for the full generate-pak flag set", but `generate-pak` takes no flags (`cli/mod.rs:40`, unit variant).

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (documented output format bricks first startup if copied)

## Acceptance Criteria

- [ ] cli.md example hash is a bare 64-hex value matching the actual println output.
- [ ] serve endpoint table lists console root/SPA fallback (with embed-ui note) and /docs/openapi.json.
- [ ] pak-management.md "flag set" pointer corrected.

## Status Updates

*To be added during implementation*
