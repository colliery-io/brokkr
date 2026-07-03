---
id: broker-get-api-v1-paks-tenant
level: task
title: "Broker: GET /api/v1/paks tenant listing derived from generators"
short_code: "BROKKR-T-0269"
created_at: 2026-07-03T00:08:48.796814+00:00
updated_at: 2026-07-03T02:57:10.120363+00:00
parent: BROKKR-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Broker: GET /api/v1/paks tenant listing derived from generators

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Expose `GET /api/v1/paks` returning a slim tenant listing `[{id, name}]` for the console's scope selector. Per the approved design, tenants ARE generators: the endpoint returns non-deleted, non-system generators (id + name only — no pak_hash, no timestamps).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `GET /api/v1/paks` returns `[{ "id": "<generator uuid>", "name": "<generator name>" }]`, admin-gated (readonly UI PAK qualifies)
- [x] The singleton system generator (`is_system = true`) and soft-deleted generators are excluded
- [x] utoipa annotations added; `angreal openapi export` regenerated `openapi/brokkr-v1.json`; `angreal openapi gen-python` / `gen-typescript` regenerated the SDK clients (lockstep versioning)
- [x] Integration tests: listing shape, system-generator exclusion, agent/generator PAKs get 403, UI PAK gets 200

## Implementation Notes

### Technical Approach
- New `crates/brokkr-broker/src/api/v1/paks.rs` module (mirror `auth.rs` for shape), mounted in `configure_api_routes` (`api/mod.rs` L196).
- DTO `PakSummary { id: Uuid, name: String }` derived from `dal.generators().list()` (reuse existing DAL; add a filtered listing only if the existing list method includes deleted/system rows).
- Naming note: path stays `/paks` per the initiative and approved option text, even though entries are generator-derived — the console is the only intended consumer; revisit naming if it grows more consumers.

### Dependencies
None hard; UI PAK (T-0267) needed for the "readonly can call this" test case.

## Status Updates

**2026-07-02 — implemented, verification pending**
- New `api/v1/paks.rs`: `GET /paks` → `Vec<PakSummary{id,name}>` from `dal.generators().list()` (already excludes system + deleted); admin-gated (readonly UI PAK passes). Also hosts `PakScopeQuery` shared by T-0270.
- Registered in `v1/mod.rs` router + `openapi.rs` (paths + `PakSummary` schema).
- Tests: `tests/integration/api/paks.rs` (shape/slimness, system-generator exclusion, UI PAK 200, generator/agent 403).
- PENDING: `angreal openapi export` + `gen-python` + `gen-typescript` (blocked on Bash availability; batch with T-0270/T-0271 schema changes).

**2026-07-02 — VERIFIED**
- Integration suite: 484 passed, 0 failed (includes all 3 paks tests). OpenAPI spec exported (`/paks` path + `PakSummary` schema present); Python + TypeScript SDKs regenerated; all three `angreal openapi check*` tasks pass (no drift). Done.