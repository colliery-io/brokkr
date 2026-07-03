---
id: broker-pak-id-scoped-filtering-on
level: task
title: "Broker: pak_id scoped filtering on fleet, stacks, and agent-events"
short_code: "BROKKR-T-0270"
created_at: 2026-07-03T00:08:53.912420+00:00
updated_at: 2026-07-03T02:57:34.369158+00:00
parent: BROKKR-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Broker: pak_id scoped filtering on fleet, stacks, and agent-events

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Accept `?pak_id=<generator uuid>` on `GET /api/v1/fleet`, `GET /api/v1/stacks`, and `GET /api/v1/agent-events`, filtering server-side to resources belonging to that tenant (generator). This is a view filter, not an authorization boundary (initiative non-goal).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `GET /fleet?pak_id=X` returns only agents with a row in `agent_generator_registrations` for generator X
- [x] `GET /stacks?pak_id=X` returns only stacks with `generator_id = X`
- [x] `GET /agent-events?pak_id=X` returns only events whose agent is registered to generator X
- [x] Omitting `pak_id` preserves current (unscoped) behavior byte-for-byte
- [x] Unknown/malformed `pak_id`: malformed UUID → 400; unknown UUID → empty list (not an error)
- [x] OpenAPI spec + Python/TypeScript SDK clients regenerated (`angreal openapi export / gen-python / gen-typescript`)
- [x] Integration tests for each endpoint: scoped, unscoped, unknown-id cases

## Implementation Notes

### Technical Approach
- Fleet handler: `crates/brokkr-broker/src/api/v1/fleet.rs` (find via routes in `api/mod.rs`); stacks: `api/v1/stacks.rs` `list_stacks`; agent-events: `api/v1/agent_events.rs` `list_agent_events` (L42).
- DAL: `agent_generator_registrations` DAL (BROKKR-T-0241) already has registration lookups; add `list_agent_ids_for_generator(generator_id)` if missing. Stacks DAL likely already filters by generator (`get_by_generator` or similar) — reuse.
- Keep filtering in SQL (join/`eq_any`), not in-handler Vec filtering, to honor the "avoid over-fetching" architecture note.

### Dependencies
None on other tasks in this initiative (composes with T-0269 output on the console side).

## Status Updates

**2026-07-02 — implemented, verification pending**
- `PakScopeQuery { pak_id: Option<Uuid> }` in `paks.rs` (utoipa IntoParams), used by all three endpoints; malformed UUID → axum Query rejection (400); unknown UUID → empty list.
- Fleet (`fleet.rs list_fleet`): post-build `retain` against a HashSet from `agent_generator_registrations().list_for_generator` (aggregates are whole-fleet grouped queries regardless).
- Stacks (`stacks.rs list_stacks`): admin path branches to existing `stacks().list_for_generator`; `stacks_total` gauge now set only on unscoped admin listings.
- Agent events: new DAL `agent_events().list_for_generator` (eq_any subselect on registrations).
- Tests: `tests/integration/api/pak_scoping.rs` — two-tenant seed; scoped/unscoped/unknown/malformed cases per endpoint.
- PENDING: OpenAPI + SDK regen (batched, see T-0269).

**2026-07-02 — VERIFIED**
- Integration suite: 484 passed, 0 failed (includes all 3 pak_scoping tests: fleet/stacks/agent-events scoped, unscoped, unknown-id, malformed-uuid cases). OpenAPI shows `pak_id` on all three GETs; SDKs regenerated drift-clean. Done.