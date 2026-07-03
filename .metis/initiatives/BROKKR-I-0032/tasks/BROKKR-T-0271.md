---
id: broker-audit-log-responses
level: task
title: "Broker: audit log responses enriched with actor names"
short_code: "BROKKR-T-0271"
created_at: 2026-07-03T00:08:58.909766+00:00
updated_at: 2026-07-03T02:57:57.566934+00:00
parent: BROKKR-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Broker: audit log responses enriched with actor names

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Audit log list responses surface a human-readable `actor_name` alongside `actor_type`/`actor_id`, so entries read "team-payments rotated PAK" instead of a bare UUID. Names resolve from the owning entity: generator → `generators.name`, agent → `agents.name`, admin → `"admin"`, system → `"system"`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `GET /api/v1/admin/audit-logs` entries include `actor_name: Option<String>` resolved at read time (no schema change to `audit_logs` — names stay live if an entity is renamed)
- [x] Resolution covers generator and agent actors; admin/system actors get their static labels; dangling IDs (deleted entities) yield `null` without erroring
- [x] Name resolution is batched (one lookup per entity type per page), not per-row N+1
- [x] OpenAPI spec + SDKs regenerated if the response schema is annotated
- [x] Integration test: entries by a generator actor carry the generator's name

## Implementation Notes

### Technical Approach
- `crates/brokkr-broker/src/api/v1/admin.rs` — `list_audit_logs` (L341) and `AuditLogListResponse` (L100). Wrap `AuditLog` in an enriched DTO (`AuditLogEntry { #[serde(flatten)] log, actor_name }`) rather than changing the model crate.
- Collect distinct actor IDs per type from the page, batch-fetch names via existing DAL list/get-by-ids methods (add `get_names_by_ids` helpers if absent).

### Dependencies
None.

## Status Updates

**2026-07-02 — implemented, verification pending**
- `admin.rs`: `AuditLogEntry { #[serde(flatten)] log, actor_name }`; `enrich_with_actor_names()` batches one `get_names_by_ids` per entity type (new DAL helpers on agents + generators, including deleted rows so history resolves); admin/system → static labels; dangling → null. `AuditLogListResponse.logs` now `Vec<AuditLogEntry>`.
- OpenAPI: `AuditLogEntry` registered.
- Test: `test_audit_logs_actor_name_enrichment` in `tests/integration/api/audit_logs.rs` (generator name + admin label).

**2026-07-02 — VERIFIED**
- Integration suite: 484 passed, 0 failed (includes the enrichment test). `AuditLogEntry` schema present in the regenerated OpenAPI spec; SDKs drift-clean. Done.