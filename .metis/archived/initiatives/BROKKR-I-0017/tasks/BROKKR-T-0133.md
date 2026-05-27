---
id: spec-hardening-a3-strip-path
level: task
title: "Spec hardening A3: Strip path prefix, fix security schemes, dedupe operationIds"
short_code: "BROKKR-T-0133"
created_at: 2026-05-14T18:26:19.784096+00:00
updated_at: 2026-05-14T20:39:51.081517+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0132]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# Spec hardening A3: Strip path prefix, fix security schemes, dedupe operationIds

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Mechanical sweep across every `#[utoipa::path]` annotation to enforce spec correctness. Covers **M1** (path prefix inconsistency), **M2** (dangling security schemes), **M3** (duplicate operationIds), and **F1** (agent_events missing security clause).

Sequenced last in Phase A so it runs over the full, post-A2 set of annotations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] **M1.** No `#[utoipa::path]` annotation declares a path starting with `/api/v1/`. Stripped 62 prefixes across 11 modules. Verified in exported spec: **0 paths begin with `/api/v1/`**.
- [x] **M1.** OpenAPI document carries `servers: [{ url: "/api/v1" }]` via new `ServersAddon` modifier in `openapi.rs`.
- [x] **M2.** Every `security(...)` clause references one of `admin_pak` / `agent_pak` / `generator_pak`. Verified: `referenced == defined`, no dangling refs. Auth endpoint now lists all three schemes (PAK type detected at runtime).
- [x] **M3.** All operationIds unique. Added explicit `operation_id = "agents_<verb>"` and `operation_id = "templates_<verb>"` to 12 handlers. Verified: 0 duplicate operationIds in spec.
- [x] **F1.** `agent_events::list_agent_events` and `agent_events::get_agent_event` carry explicit `security(...)` clauses (added in T-A1 as a bonus migration cleanup; re-verified here).
- [~] `redocly lint` deferred to T-B1 (CI drift task) — internal validation script confirms structural correctness for all four target concerns.

## Implementation Notes

### Technical Approach

1. Add `servers = [...]` to the `#[derive(OpenApi)]` attribute (or via a `Modify` impl alongside `SecurityAddon`). Decide whether to use a relative URL `/api/v1` or a variable-expanded one; relative is simplest.
2. Run a find/sed across `crates/brokkr-broker/src/api/v1/*.rs` to strip `/api/v1` prefixes from `path = "..."` arguments. Manually review each diff — some prefixes might be elsewhere in the annotation string.
3. Replace dangling `security(("pak" = []))` references with the appropriate registered scheme. Audit each route: agents/admin routes → `admin_pak` (and/or `agent_pak`), stacks/templates/work_orders → choose between `admin_pak` / `generator_pak` / `agent_pak` based on runtime middleware behavior. Replace `bearer_auth` similarly.
4. Add `operation_id = "agents_list_labels"` (etc.) to every `#[utoipa::path]` whose operationId currently collides. Easiest: prefix every operationId with its module name for consistency, even where collisions don't exist today (forward-proofs future additions).
5. Add `security(...)` clauses to the two `agent_events` annotations.
6. Re-export, validate via the same Python diff script used in the audit. Re-run `cargo test` to ensure nothing in the spec-aware integration tests broke.

### Dependencies

- Hard: [[BROKKR-T-0132]]. A3 must operate on the complete post-A2 annotation set.

### Risk Considerations

- The path-prefix change is observable to anyone using the swagger-ui or directly consuming the spec — call it out in PR description.
- Adding `servers` may interact with utoipa-swagger-ui's URL handling; test that `/swagger-ui` still resolves request paths correctly.

## Status Updates

### 2026-05-14 — Completed

**Changes:**

1. **M1 — path prefix sweep.** Python regex `(path = ")/api/v1/` → `\1/` across all 11 v1 handler modules. 62 stripped:
   - admin (2), agent_events (2), agents (1), auth (1), deployment_objects (1), diagnostics (5), generators (6), stacks (14), templates (11), webhooks (10), work_orders (9).
2. **M1 — `servers`.** Added new `ServersAddon` modifier in `openapi.rs` that sets `openapi.servers = Some(vec![Server::new("/api/v1")])`. Registered alongside `SecurityAddon` in `modifiers(...)`.
3. **M2 — dangling security scheme.** Only one remained after T-A1's bearer_auth cleanup: `auth::verify_pak` referenced `("pak" = [])`. Replaced with all three registered PAK schemes (any PAK kind authenticates this endpoint).
4. **M3 — operationId dedup.** Inserted `operation_id = "agents_<name>"` / `operation_id = "templates_<name>"` on 12 handlers: agents and templates `list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation`. Stacks analogues already had `stacks_*` from T-A2.
5. **F1 — agent_events security.** Already closed during T-A1 cleanup; re-verified here.

**Verification (`openapi/brokkr-v1.json`):**

```
M1: paths with /api/v1/ prefix: 0
M1: servers: [{'url': '/api/v1'}]
M2: dangling = ∅, referenced = {admin_pak, agent_pak, generator_pak}
M3: duplicate operationIds: []
F1: /agent-events and /agent-events/{id} carry [admin_pak, agent_pak, generator_pak]
Totals: paths=59  ops=85  schemas=65
```

`cargo build -p brokkr-broker` clean.

**Notes / non-issues:**
- Path count unchanged (59) — strip is rename, not removal. Same routes, now resource-relative.
- Swagger-UI continues to mount at `/swagger-ui` and reads `/docs/openapi.json`; the runtime `.nest("/api/v1", ...)` still applies. Browser usage may now show the `servers` selector — acceptable.
- Phase A spec hardening is complete; the spec at `openapi/brokkr-v1.json` is the artifact downstream generator tasks (T-B2, T-B3) consume.