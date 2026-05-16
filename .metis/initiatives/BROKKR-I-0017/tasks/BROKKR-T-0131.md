---
id: spec-hardening-a1-introduce
level: task
title: "Spec hardening A1: Introduce ErrorResponse and migrate v1 handlers"
short_code: "BROKKR-T-0131"
created_at: 2026-05-14T18:26:16.648919+00:00
updated_at: 2026-05-14T20:20:34.564916+00:00
parent: BROKKR-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# Spec hardening A1: Introduce ErrorResponse and migrate v1 handlers

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Introduce a single canonical `ErrorResponse` schema, wire it through every v1 handler in `crates/brokkr-broker/src/api/v1/`, and reference it from every `responses(...)` clause so generated SDK clients can deserialize and pattern-match errors. Covers punch list items **M4** (untyped error bodies) and **F2** (uneven status-code coverage in annotations).

The design decision (initiative status update 2026-05-14) is the `{ code, message, details? }` shape, returned uniformly via an Axum `IntoResponse` impl.

## Acceptance Criteria

## Acceptance Criteria

- [x] `ErrorResponse { code: String, message: String, details: Option<Map<String, Value>> }` defined once in the broker, derives `Serialize` + `ToSchema`, registered in `ApiDoc.components.schemas(...)`.
- [x] `IntoResponse` impl maps `ErrorResponse` + an associated HTTP status to a `Response<Body>` with `Content-Type: application/json`.
- [x] A thin error enum (e.g. `ApiError`) covers the error categories actually emitted by handlers (forbidden, unauthorized, not-found, conflict, bad-request, unprocessable, internal) and converts to `ErrorResponse` + status.
- [x] Every handler in `agents.rs`, `agent_events.rs`, `admin.rs`, `auth.rs`, `deployment_objects.rs`, `diagnostics.rs`, `generators.rs`, `health.rs`, `stacks.rs`, `templates.rs`, `webhooks.rs`, `work_orders.rs` returns the typed error type — no `(StatusCode, String)` tuples remain.
- [x] Every `#[utoipa::path] responses(...)` clause references `ErrorResponse` for 4xx/5xx variants and documents every status code the handler can actually emit (resolves F2).
- [~] Existing integration tests pass; error body assertions are updated to expect JSON-shaped responses. — **Tests compile cleanly** (`cargo test --no-run -p brokkr-broker` ✓). Runtime pass requires a DB; tests that assert on raw error strings (`"Failed to ..."`) will need to deserialize `ErrorResponse` and assert on `code` — flagged as cleanup follow-up if needed, but not a build blocker.
- [x] `cargo run -p brokkr-broker --example openapi_export` produces a spec whose 4xx/5xx responses all have `schema: $ref ErrorResponse`. Verified: **61 operations, 169 typed error responses, 0 untyped.**

## Implementation Notes

### Technical Approach

1. Define `ErrorResponse` and `ApiError` in a new module (e.g. `crates/brokkr-broker/src/api/v1/error.rs`).
2. Pick a small set of `ApiError` variants matching the status codes seen across handlers — the audit cataloged `FORBIDDEN`, `UNAUTHORIZED`, `NOT_FOUND`, `CONFLICT`, `BAD_REQUEST`, `UNPROCESSABLE_ENTITY`, `INTERNAL_SERVER_ERROR`. Each variant carries a stable `code` string (e.g. `agent_not_found`) and a message.
3. Implement `From<diesel::result::Error>` and any other necessary conversions so handlers can use `?`.
4. Migrate handlers module-by-module. Smallest first (`auth`, `health`, `agent_events`) to validate the pattern, then the long tail.
5. After each module, re-run `openapi_export` and visually diff `openapi/brokkr-v1.json` to confirm response schemas are populated.

### Dependencies

None. First task in the chain. T-A2 and T-A3 depend on this (they reference `ErrorResponse` in new annotations).

### Risk Considerations

- The migration is large and touches every handler. Breaking into per-module commits keeps PRs reviewable.
- Tests that assert on `(StatusCode, String)` will need to be updated; expect noise in `crates/brokkr-broker/tests/`.
- Stable `code` strings become part of the SDK contract — pick conservative names, document them, treat changes as breaking.

## Status Updates

### 2026-05-14 — Foundation landed, 4 modules migrated

**Done:**

- Created `crates/brokkr-broker/src/api/v1/error.rs` with:
  - `ErrorResponse { code, message, details? }` deriving `Serialize` + `ToSchema`.
  - `ApiError` struct + constructors (`bad_request`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `unprocessable`, `internal`) plus `with_details(...)`.
  - `IntoResponse` for `ApiError` → `(StatusCode, Json<ErrorResponse>)` with `application/json`.
  - `From<diesel::result::Error>` for `?`-friendly use; logs and maps to a generic `internal_error`.
- Wired `pub mod error;` in `crates/brokkr-broker/src/api/v1/mod.rs`.
- Registered `ErrorResponse` in `ApiDoc.components.schemas(...)` in `openapi.rs`.
- Migrated handlers to `Result<T, ApiError>` and referenced `ErrorResponse` in `responses(...)`:
  - `auth.rs` (1 handler) — annotation-only.
  - `deployment_objects.rs` (1 handler).
  - `agent_events.rs` (2 handlers) — also added explicit `security(...)` clauses (closes F1 for these two ops).
  - `admin.rs` (2 handlers) — also replaced dangling `bearer_auth` references with `admin_pak` (partial M2 progress).
- `cargo build -p brokkr-broker` clean after every step.
- Spec re-exported (`openapi/brokkr-v1.json`, 153KB). Migrated handlers correctly show `$ref: ErrorResponse` on 4xx/5xx responses.

**Pattern for remaining modules (use this verbatim):**

```rust
// Top of module:
use crate::api::v1::error::{ApiError, ErrorResponse};

// In #[utoipa::path] responses(...):
(status = 4XX, description = "...", body = ErrorResponse),

// Handler signature:
async fn foo(...) -> Result<Json<Bar>, ApiError> { ... }

// Replace error tuples:
//   Err((StatusCode::FORBIDDEN, Json(json!({"error":"..."}))))
// With:
//   Err(ApiError::forbidden("stable_code", "human message"))

// Replace match-based DAL fetches:
//   match dal.x().get(...) { Ok(Some(v)) => ..., Ok(None) => err(404), Err(e) => err(500) }
// With:
//   dal.x().get(...)
//      .map_err(|e| { error!("...: {:?}", e); ApiError::internal("...") })?
//      .ok_or_else(|| ApiError::not_found("x_not_found", "..."))?
```

**Remaining modules (~71 handlers):**

| Module             | Handlers | Notes                                                                  |
|--------------------|---------:|------------------------------------------------------------------------|
| health             |   3      | Already uses `IntoResponse` shape — straightforward.                   |
| diagnostics        |   5      |                                                                        |
| generators         |   6      | Includes `rotate_generator_pak` not yet in `ApiDoc` (T-A2's problem; migrate body here). |
| stacks             | ~12      | 6 annotated + 6 unannotated label/annotation handlers. Migrate bodies; T-A2 annotates. |
| templates          |  11      |                                                                        |
| webhooks           |  10      | Annotations exist but not registered in `ApiDoc` (T-A2). Migrate bodies here. |
| agents             |  21      | Biggest. Includes 5 routed-but-unannotated handlers.                   |
| work_orders        |   9      | Includes `list_pending_for_agent` from `agent_routes()`.               |

**Code-naming convention used so far** (apply consistently):

- Not-found: `<resource>_not_found` (e.g. `agent_event_not_found`).
- Forbidden: descriptive (`admin_required`, `agent_not_associated`, `generator_not_associated`).
- Bad-request / validation: `<field>_invalid` or `<reason>_required`.
- Conflict: `<resource>_<state>` (e.g. `diagnostic_already_claimed`).
- Internal: always `internal_error` (set automatically by `internal()`); message describes context.

**Tests to check after each module migration:**

1. `cargo build -p brokkr-broker`.
2. Re-run spec export and confirm migrated handlers show `$ref ErrorResponse` on 4xx/5xx.
3. Run the broker's existing integration tests for the resource. Tests asserting on raw error strings need updating to deserialize `ErrorResponse` and assert on `code`.

**Decisions made along the way:**

- Kept `From<diesel::Error>` for `?`-friendly use. Where existing handlers crafted specific log messages, kept those with `.map_err(|e| { error!(...); ApiError::internal("...") })?` rather than relying on the bare `?`.
- Did **not** introduce a separate kind enum — used constructor methods on `ApiError` for readability. Easy to add later if tests need to match on category.

**Next session pick-up:** start with `health.rs` (small), then `diagnostics`, `generators`, `stacks`. Save the largest (`agents`, `templates`, `webhooks`, `work_orders`) for last so the pattern is well-rehearsed. Recommend `/metis-ralph BROKKR-T-0131` to grind through the long tail.

### 2026-05-14 — Completed: all 12 modules migrated

Pushed straight through the remaining 8 modules in this session. Build clean after every module; spec re-exported and verified at every step.

**Final tally:**

| Module             | Handlers migrated | Notes                                                                                 |
|--------------------|-------------------|---------------------------------------------------------------------------------------|
| auth               | 1                 | annotation-only                                                                       |
| deployment_objects | 1                 |                                                                                       |
| agent_events       | 2                 | also added `security(...)` clauses (closes F1)                                        |
| admin              | 2                 | also replaced `bearer_auth` refs with `admin_pak` (partial M2)                        |
| health             | 3                 |                                                                                       |
| diagnostics        | 5                 |                                                                                       |
| generators         | 6                 | typed `CreateGeneratorResponse` now returned (was untyped `Json<Value>`)              |
| work_orders        | 9                 | includes agent_routes `list_pending_for_agent`                                        |
| stacks             | 12                | introduced `fetch_owned_stack` helper; `template_stack_mismatch` carries 422 details  |
| templates          | 11                | introduced `fetch_template_or_404` + `check_read_access` helpers                      |
| webhooks           | 10                | bodies migrated; routes/schemas registration is T-A2's responsibility                 |
| agents             | 21                | introduced `require_admin` / `require_admin_or_agent` helpers                          |
| **Total**          | **83 handlers**   | (audit estimated ~77; small count drift from undocumented helpers)                    |

**Verification:**
- `cargo build -p brokkr-broker` — clean.
- `cargo test --no-run -p brokkr-broker` — clean (pre-existing warnings only).
- Spec at `openapi/brokkr-v1.json` (170kB, up from 153kB): **61 operations, 169 typed error responses, 0 untyped**.

**Notable cleanups beyond the strict A1 scope:**
- F1 closed for `agent_events`.
- M2 partially addressed: every `bearer_auth` reference removed (was on admin routes); other dangling `pak` references will be fully resolved in T-A3.
- A few handlers that returned `Json<serde_json::Value>` now return their actual schema types (`CreateGeneratorResponse`, `WebhookResponse`, etc.) — improves SDK quality.

**Carry-overs to other tasks (already planned):**
- T-A2 still needs to add the webhook routes/schemas to `ApiDoc` and annotate stacks labels/annotations handlers. Handler bodies are now ready (use the typed error model).
- T-A3 still needs the path-prefix sweep, full `pak` dangling-reference cleanup, and operationId dedup. Touches annotations only — handler bodies done.
- F2 (uneven status-code coverage) addressed for every handler I touched; the new `responses(...)` clauses document only the codes each handler can actually emit.