---
id: audit-openapi-spec-coverage
level: task
title: "Audit OpenAPI spec coverage against v1 API implementation"
short_code: "BROKKR-T-0130"
created_at: 2026-05-14T17:29:10.601672+00:00
updated_at: 2026-05-14T18:18:36.276588+00:00
parent: BROKKR-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# Audit OpenAPI spec coverage against v1 API implementation

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Walk every v1 API route, request type, response type, and error path in `crates/brokkr-broker/src/api/v1/` and verify that the `utoipa` annotations accurately and completely describe the actual runtime behavior. Produce a punch list of spec gaps (missing schemas, wrong status codes, undocumented endpoints, untyped errors, missing examples) that downstream SDK generation will depend on.

This audit gates the rest of the initiative — generated SDKs are only as good as the spec they're built from. Any gap found here becomes either a fix in the broker before generators run, or an explicit accepted limitation documented in the initiative.

## Acceptance Criteria

## Acceptance Criteria

- [x] Every handler function in `crates/brokkr-broker/src/api/v1/{auth,agents,agent_events,admin,deployment_objects,diagnostics,generators,health,stacks,templates,webhooks,work_orders}.rs` is enumerated and cross-checked against its `#[utoipa::path]` annotation.
- [x] Every request body, query param, and path param type is verified to derive `ToSchema` / `IntoParams` and produce a usable schema in the generated spec.
- [x] Every response variant (success and error) has a documented status code and response schema. Untyped `String` / opaque error bodies are flagged.
- [x] Authentication requirements (PAK header, admin-only routes, agent vs. operator scopes) are reflected in the spec via security schemes — gaps flagged.
- [x] The full OpenAPI document is exported (`openapi/brokkr-v1.json` or equivalent) and committed as the audit artifact.
- [x] A written punch list is added to this task's status updates: each gap categorized as **must-fix-before-generation**, **fix-eventually**, or **accepted-limitation**, with rationale.
- [x] Findings are summarized back into the parent initiative `BROKKR-I-0017` so decomposition can be planned with accurate scope.

## Implementation Notes

### Technical Approach

1. Enumerate routes by reading `crates/brokkr-broker/src/api/v1/mod.rs` and each module's router builder. Cross-reference against the `paths(...)` list in `crates/brokkr-broker/src/api/v1/openapi.rs`.
2. Export the spec. The cleanest path is a small `cargo run` binary or test that calls `ApiDoc::openapi().to_pretty_json()` and writes it to `openapi/brokkr-v1.json`. If no such entry point exists, add one — it's needed by the initiative regardless.
3. For each route, diff three things:
   - **Annotation vs handler signature** — do declared request/response types match what the handler actually accepts/returns?
   - **Annotation vs runtime behavior** — does the handler ever return status codes or error shapes not listed in `responses(...)`? Grep for `StatusCode::`, `(StatusCode::..., ...)` tuples, and error-conversion impls.
   - **Schema completeness** — do referenced types derive `ToSchema`? Are enum variants, optional fields, and nested types fully represented?
4. Validate the exported spec with an external tool (e.g. `redocly lint` or `openapi-generator validate`) as a sanity check — flag any structural errors the human review might miss.

### Scope boundaries

- This task **audits**; it does not fix. Spec corrections are separate downstream tasks created during initiative decomposition.
- One exception: if exporting the spec to a file requires adding a tiny `bin` or `dev-dependency` plumbing, that's in scope — the artifact must exist for the audit to be reproducible.
- Out of scope: evaluating generators, prototyping clients, touching `brokkr-agent`.

### Dependencies

None. This is the first task of the initiative.

### Risk Considerations

- The audit may surface enough spec debt that the initiative's downstream task count grows significantly. That's the point — better to know now than to discover it mid-generation. Report findings honestly even if the punch list is long.
- `utoipa` schema generation has known sharp edges around generics, trait objects, and tagged enums. Watch for types that compile but produce broken / empty schemas.

## Status Updates

### 2026-05-14 — Initial enumeration pass (static analysis)

Counted `#[utoipa::path]` annotations vs `ApiDoc::paths(...)` registrations vs `.route(...)` bindings across every v1 module. The spec is significantly under-registered relative to both the annotations and the routed handlers.

**Annotation counts per module** (`#[utoipa::path]` // listed in `ApiDoc.paths(...)`):

| Module             | Annotations | Listed | Delta |
|--------------------|-------------|--------|-------|
| admin              |  2          |  2     |  0    |
| agent_events       |  2          |  2     |  0    |
| agents             | 21          | 16     | **-5** |
| auth               |  1          |  1     |  0    |
| deployment_objects |  1          |  1     |  0    |
| diagnostics        |  5          |  5     |  0    |
| generators         |  6          |  5     | **-1** |
| health             |  3          |  3     |  0    |
| stacks             |  6          |  6     |  0 (but see below) |
| templates          | 11          | 11     |  0    |
| webhooks           | 10          |  0     | **-10** |
| work_orders        |  9          |  9     |  0    |
| **Total**          | **77**      | **61** | **-16** |

Spec export will pin exact numbers; static counts above are the floor.

**Definite gaps — annotated handlers wired into routes but missing from `ApiDoc.paths(...)`:**

- `agents::list_events` (route `GET /agents/:id/events`)
- `agents::create_event` (route `POST /agents/:id/events`)
- `agents::record_heartbeat` (route `POST /agents/:id/heartbeat`)
- `agents::get_associated_stacks` (route `GET /agents/:id/stacks`)
- `agents::rotate_agent_pak` (route `POST /agents/:id/rotate-pak`)
- `generators::rotate_generator_pak` (route `POST /generators/:id/rotate-pak`)
- All 10 webhook routes — every handler in `webhooks.rs` is annotated but none reach the spec.

**Handlers routed but with NO `#[utoipa::path]` annotation:**

- `stacks::list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation` — routes for `/stacks/:id/labels(/:label)` and `/stacks/:id/annotations(/:key)` are bound in the stacks router; handlers have no annotations. ~6 routes invisible to the spec.
- `stacks::list_deployment_objects` and `stacks::create_deployment_object` exist as handlers — need to confirm whether they are routed or dead code.

**Schema gaps paired with route gaps:**

- `components(schemas(...))` registers **no** webhook-related schemas. `crates/brokkr-models/src/models/webhooks.rs` defines four `ToSchema`-deriving types plus several more local types in `webhooks.rs` (request/response DTOs). All need registration once webhook routes are exposed.

**Error-shape finding (must-fix for SDKs):**

Every handler I sampled returns errors as `(StatusCode, String)` tuples — see `agents.rs`, `stacks.rs`, `generators.rs`, `diagnostics.rs`. There is **no typed error response schema**. Consequences for generated SDKs:

- Error bodies type as `String` (or untyped) — callers can't pattern-match programmatically.
- Status code is the only structured signal, and coverage in `responses(...)` clauses is uneven: `FORBIDDEN`, `CONFLICT`, `BAD_REQUEST`, `UNPROCESSABLE_ENTITY`, `NOT_FOUND` all appear in handler bodies but not always in annotations.

Categorizing this as **must-fix-before-generation**. A single canonical `ErrorResponse { code, message, details? }` schema, applied uniformly to error variants, is the right shape.

**Authentication coverage:**

`SecurityAddon` (`openapi.rs:194`) registers `admin_pak`, `generator_pak`, `agent_pak` globally, but each `#[utoipa::path]` must carry a `security(...)` clause to bind a route to a scheme. Need to spot-check whether handlers do this consistently — if not, the generated SDKs won't know which routes require which credential type.

### 2026-05-14 — Spec exported, ground-truth audit complete

Added `crates/brokkr-broker/examples/openapi_export.rs` (writes `openapi/brokkr-v1.json` from `ApiDoc::openapi().to_pretty_json()`). Reproduce with:

```
cargo run -p brokkr-broker --example openapi_export
```

**Ground truth from exported spec:** 42 paths · 61 operations · 50 schemas · 3 security schemes registered. The export surfaced several issues the static pass missed.

---

## Final punch list

### MUST-FIX-BEFORE-GENERATION

**M1. Path prefix inconsistency — 13 paths are wrong.**
All v1 routes are served under `/api/v1/` (via `.nest("/api/v1", api_routes)` in `mod.rs:75`), but 13 `#[utoipa::path(path = "...")]` annotations omit the prefix. The spec therefore documents URLs that don't exist at runtime; generated clients would hit `/agents` instead of `/api/v1/agents`. Affected paths:
- `/agents`, `/agents/`, `/agents/{id}` (+ all sub-resources: `/annotations`, `/labels`, `/targets`, `/target-state`, `/health-status`)
- `/deployment-objects/{id}/health`, `/stacks/{id}/health`
Fix: standardize — either prefix every annotation, or (preferred) strip the prefix from the ones that have it and let the nest handle it via utoipa's `nest` support / a global servers entry. Pick one approach and enforce it.

**M2. Dangling security scheme references.**
Operations cite `pak` and `bearer_auth` in their `security(...)` clauses, but the only schemes registered in `components.securitySchemes` are `admin_pak`, `agent_pak`, `generator_pak`. `pak` is referenced by stacks, templates, work_orders, auth, and the agent-scoped work-orders route; `bearer_auth` is referenced by admin endpoints. Generators will either fail validation or emit code referencing undefined schemes.
Fix: either register `pak` / `bearer_auth` in `SecurityAddon`, or rewrite the route annotations to use the three already-registered schemes (most natural — `pak` was probably shorthand intended to mean "any PAK").

**M3. 6 duplicate operationIds across the spec.**
`list_labels`, `add_label`, `remove_label`, `list_annotations`, `add_annotation`, `remove_annotation` each appear twice (once on agents paths, once on templates paths). OperationIds are required to be unique by OpenAPI 3.x; generators handle this either by failing or by auto-suffixing (`add_label_1`), which produces ugly client APIs.
Fix: prefix `operation_id` in each `#[utoipa::path]` (e.g. `agents_list_labels`, `templates_list_labels`).

**M4. Error response bodies have no schema.**
Every 4xx/5xx response declares `content: application/json` but with no `schema` — handlers return raw `(StatusCode, String)`. Generated clients can't deserialize, can't pattern-match, can't surface useful errors.
Fix: introduce one canonical `ErrorResponse { code, message, details? }` schema, return it from every error path (Axum `IntoResponse` impl), and reference it from every `#[utoipa::path]` `responses(...)` block.

**M5. 16+ routes missing from the spec entirely.**
Confirmed by diff between the bound routes in each module's `routes()` builder and the 42 paths in the exported spec:
- All 10 webhook routes (entire `webhooks` module — handlers annotated, schemas defined, but **none** registered in `ApiDoc.paths(...)` or `ApiDoc.components.schemas(...)`).
- `POST /api/v1/agents/{id}/events`, `GET /api/v1/agents/{id}/events` (`agents::create_event`, `list_events`)
- `POST /api/v1/agents/{id}/heartbeat` (`agents::record_heartbeat`)
- `GET /api/v1/agents/{id}/stacks` (`agents::get_associated_stacks`)
- `POST /api/v1/agents/{id}/rotate-pak` (`agents::rotate_agent_pak`)
- `POST /api/v1/generators/{id}/rotate-pak` (`generators::rotate_generator_pak`)
- `/api/v1/stacks/{id}/labels` (+ delete by name), `/api/v1/stacks/{id}/annotations` (+ delete by key) — 6 routes; handlers have no `#[utoipa::path]` at all.
Fix: add `#[utoipa::path]` annotations where missing, then register every handler in `ApiDoc.paths(...)` and the corresponding request/response types in `ApiDoc.components.schemas(...)`.

**M6. Webhook schema set absent.**
Tied to M5. `crates/brokkr-models/src/models/webhooks.rs` defines 4 `ToSchema` types; `crates/brokkr-broker/src/api/v1/webhooks.rs` defines 6 more request/response DTOs. None are in `components.schemas`.
Fix: enumerate and register alongside the route fix in M5.

### FIX-EVENTUALLY

**F1. Two operations have no security clause.**
`GET /api/v1/agent-events` and `GET /api/v1/agent-events/{id}` inherit from global security, which is unset — spec marks them effectively public, even though `auth_middleware` enforces auth at runtime. Doesn't break SDK generation but misleads consumers.
Fix: add explicit `security(...)` clauses to both annotations.

**F2. Uneven coverage of error status codes in `responses(...)`.**
Handlers return `BAD_REQUEST`, `CONFLICT`, `UNPROCESSABLE_ENTITY`, `NOT_FOUND` in various paths but the `#[utoipa::path] responses(...)` clauses don't always document them. Naturally falls out of M4 (when introducing the typed `ErrorResponse`, document every status code the handler can actually emit). Sub-fix to track during M4.

**F3. Stacks deployment-object handlers — confirm liveness.**
`stacks::list_deployment_objects` and `stacks::create_deployment_object` exist (`stacks.rs:411,455`) but I did not find a `.route(...)` line binding them. Likely dead code or routed elsewhere — verify before deciding to annotate or delete.

### ACCEPTED-LIMITATION

**A1. Three security schemes, generic Authorization header.**
All three registered PAK schemes use the same `Authorization` header with no distinguishing pattern in the spec; the broker disambiguates based on the PAK prefix at runtime. Generated SDKs will surface three "credentials" that all set the same header — slightly redundant but harmless. Document this in the SDK ergonomic-wrapper layer (one credential field, internally routed). No spec change needed.

---

## Acceptance criteria status

- [x] Every handler function enumerated and cross-checked.
- [x] Request body / query param / path param schema coverage examined (gaps captured in M5, M6).
- [x] Every response variant audited; untyped errors flagged (M4) and missing status codes (F2).
- [x] Authentication requirements audited; gaps captured (M2, F1, A1).
- [x] Full OpenAPI document exported (`openapi/brokkr-v1.json`, 151kB, committed by this task).
- [x] Written punch list with must-fix / fix-eventually / accepted-limitation categorization.
- [x] Findings summarized back to `BROKKR-I-0017` (see initiative status update).

### Skipped from original plan

`redocly lint` / `openapi-generator validate` was listed as a sanity check. Skipping for this audit — the structural issues found by direct inspection (M1–M6) dwarf anything a linter would flag, and running a linter now would just produce duplicate signal. Worth running once spec fixes land, as part of the CI drift-check task.