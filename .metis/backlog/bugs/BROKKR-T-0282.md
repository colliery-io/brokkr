---
id: docs-tmp-brokkr-keys-key-txt
level: task
title: "Docs: /tmp/brokkr-keys/key.txt guidance omits first-boot-only + deleted-on-shutdown semantics (unrecoverable admin PAK loss)"
short_code: "BROKKR-T-0282"
created_at: 2026-07-27T14:19:55.723809+00:00
updated_at: 2026-07-27T14:19:55.723809+00:00
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

# Docs: /tmp/brokkr-keys/key.txt guidance omits first-boot-only + deleted-on-shutdown semantics (unrecoverable admin PAK loss)

## Objective

Warn readers that the fallback admin-PAK bootstrap file is a one-shot artifact. `installation.md` Step 3 "Get the Admin PAK" (~lines 74-78) and `multi-tenant-setup.md` Step 3 tell readers to fetch the PAK from `/tmp/brokkr-keys/key.txt`, but omit that (a) the file is written only on the genuinely first startup — gated by the empty `app_initialization` table (`crates/brokkr-broker/src/cli/commands.rs:76-97`) — and (b) it is deleted on graceful shutdown (`crates/brokkr-broker/src/utils/mod.rs:34-43`). A pod restart/reschedule before capture loses the admin PAK unrecoverably (recovery = `rotate admin`, which readers at this point don't know about).

Found in the 2026-07-27 auth-drift sweep; code-verified.

**Related blocker, same root cause** (full-tree review): `pak-management.md` claims you can rotate the admin PAK by clearing `BROKKR__BROKER__PAK_HASH` and restarting the broker ("or restart the broker — startup runs the same upsert"). False — `serve()` runs `upsert_admin` only inside the first-run branch (`cli/commands.rs:92-97`); a restart on an existing DB rotates nothing and key.txt is never written. The only real rotation paths are `brokkr-broker rotate admin` (CLI) — noting CLI rotation can't invalidate a running broker's auth cache until TTL (default 60s).

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (missing critical caveat in install path)

### Priority
- [x] P1 - High (silent, unrecoverable credential loss during first install)

### Impact Assessment
- **Affected Users**: Anyone using the no-preset-hash bootstrap path (the default flow when `BROKKR__BROKER__PAK_HASH` is unset).
- **Reproduction**: Install broker without a preset hash → pod restarts (OOM, node drain, `helm upgrade`) before the operator reads key.txt → admin PAK gone; key.txt not rewritten on subsequent boots.

## Acceptance Criteria

- [ ] installation.md and multi-tenant-setup.md warn that key.txt is first-boot-only and removed on graceful shutdown, and say to capture it immediately.
- [ ] pak-management.md's rotate-by-restart claim is removed; `rotate admin` documented as the only rotation path, with the auth-cache TTL caveat for CLI rotation.
- [ ] Both pages point to `brokkr-broker rotate admin` as the recovery path and to the `generate-pak` + `BROKKR__BROKER__PAK_HASH` preset flow (and Secret-based sourcing, BROKKR-T-0278) as the recommended way to avoid the race entirely.

## Status Updates

*To be added during implementation*
