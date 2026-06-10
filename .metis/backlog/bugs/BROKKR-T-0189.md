---
id: audit-actions-defined-but-never
level: task
title: "Audit actions defined but never emitted (auth.failed, config.reloaded)"
short_code: "BROKKR-T-0189"
created_at: 2026-06-10T03:04:10.114868+00:00
updated_at: 2026-06-10T03:04:10.114868+00:00
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

# Audit actions defined but never emitted (auth.failed, config.reloaded)

## Objective

Audit action constants exist that are never emitted: `ACTION_AUTH_FAILED` is defined and even queried by the DAL (`crates/brokkr-broker/src/dal/audit_logs.rs:278-283`) but no code path logs it; `POST /api/v1/admin/config/reload` emits no audit event despite config changes being audit-relevant. Today only `agents.rs`, `stacks.rs`, and `webhooks.rs` call `audit::log_action`. The audit-log reference previously documented these phantom events (now corrected).

## Acceptance Criteria

- [ ] Auth failures emit `auth.failed` audit entries (mind write amplification on unauthenticated spray — consider rate limiting/sampling)
- [ ] Config reload emits an audit entry with the change set
- [ ] Survey remaining documented action constants vs. actual emission; close or remove gaps
- [ ] `docs/src/reference/audit-logs.md` examples updated to match

## Status Updates

- 2026-06-09: Found during /docs-diataxis accuracy review.
- 2026-06-09: PARTIALLY IMPLEMENTED (uncommitted, unit tests green): middleware emits `auth.failed` (system actor, reason/path/ip/user-agent) on 401 credential rejections; `reload_config` emits `config.reloaded` with the change set. Remaining AC: survey other documented constants; consider rate-limiting auth-failure writes under spray.
- 2026-06-09 (addendum): accuracy re-review confirmed generator PAK rotation (REST) emits NO audit event and CLI rotations bypass the audit logger entirely — both in scope for the remaining work here. Docs now document the emitted-vs-defined split (reference/audit-logs.md).
- 2026-06-10: residue IMPLEMENTED: generator REST rotation now emits `pak.rotated` (actor admin or the generator itself); CLI agent/generator rotations write synchronous audit rows via the DAL with `details.via="cli"` (async logger is serve-only). Docs updated. Remaining: lifecycle constants for generators/templates/work orders still unemitted (documented as such); auth-failure spray rate-limiting consideration.
- 2026-06-10 (closure pass): ALL remaining constants now emitted — generator/template lifecycle, workorder created/claimed/completed/failed/retry, webhook.delivery_failed (both broker-worker and agent-report dead transitions), pak.created (REST + CLI creation). auth.success and pak.deleted documented as intentionally unrecorded (volume; no standalone PAK deletion). Docs catalog updated to emitted-vs-intentionally-not.
