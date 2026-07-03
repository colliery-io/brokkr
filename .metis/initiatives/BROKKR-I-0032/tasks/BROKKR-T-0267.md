---
id: broker-ephemeral-read-only-ui-pak
level: task
title: "Broker: ephemeral read-only UI PAK with middleware enforcement"
short_code: "BROKKR-T-0267"
created_at: 2026-07-03T00:08:39.324760+00:00
updated_at: 2026-07-03T02:56:08.645514+00:00
parent: BROKKR-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Broker: ephemeral read-only UI PAK with middleware enforcement

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Generate an ephemeral, in-memory, read-only "UI PAK" at broker startup and teach the auth middleware to (a) recognize it as a readonly admin credential and (b) reject non-GET requests from readonly credentials, allowlisting `POST /api/v1/diagnostics` (approved design decision — observability action, not a desired-state mutation).

## Acceptance Criteria

- [x] Broker startup generates a random PAK (existing `utils::pak` machinery) and stores its hash in process memory only — never in the database, never logged
- [x] `AuthPayload` gains a `readonly: bool` field (false for all existing paths: admin, agent, generator)
- [x] `verify_pak` recognizes the UI PAK hash and returns `AuthPayload { admin: true, readonly: true, .. }`
- [x] Middleware rejects non-GET requests from readonly payloads with 403, except `POST /api/v1/diagnostics` which is allowed
- [x] Existing admin/agent/generator auth behavior unchanged (regression: existing integration tests pass)
- [x] Integration tests: UI PAK can GET (e.g. `/api/v1/fleet`), cannot POST/PUT/DELETE (e.g. `POST /api/v1/stacks` → 403), can POST `/api/v1/diagnostics`

## Implementation Notes

### Technical Approach
- `crates/brokkr-broker/src/api/v1/middleware.rs` — `AuthPayload` (L34), `verify_pak` (L148), `auth_middleware` (L68). Check UI PAK hash before the DB lookups (cheap in-memory compare, avoids admin_role query).
- UI PAK storage: a `OnceLock<String>` (hash) in a new `crates/brokkr-broker/src/utils/ui_pak.rs` (or alongside `utils::pak`), initialized from `main`/broker startup. The raw PAK is also retained in memory for T-0268 (HTML injection).
- Readonly enforcement lives in `auth_middleware` after successful verification: it has the `Request` (method + path).
- The auth cache keys by pak_hash and stores `AuthPayload`; the readonly flag rides along transparently. `AuthResponse` in `GET /auth/pak` (verify_pak handler) should expose `readonly` too.

### Dependencies
None (first task in the chain). T-0268 consumes the raw PAK.

### Risk Considerations
- Multi-replica brokers: each replica gets its own UI PAK; the token is injected by the same replica that serves the HTML, but subsequent API calls may hit a different replica behind a load balancer. Acceptable for v1 (console is a single-broker ops tool); note it in the module docs.

## Status Updates

**2026-07-02 — implementation drafted, compile check pending**
- Added `crates/brokkr-broker/src/utils/ui_pak.rs`: `OnceLock<UiPak {token, hash}>`, idempotent `init()`, `token()`/`hash()` accessors; registered in `utils/mod.rs`.
- `middleware.rs`: `AuthPayload.readonly` + `AuthResponse.readonly`; UI PAK constant-time hash check at top of `verify_pak` (before cache/DB); `readonly_request_allowed()` allows GET/HEAD + POST `/auth/pak` + POST `/deployment-objects/:id/diagnostics`; enforcement in `auth_middleware` returns 403.
- `auth.rs` handler emits `readonly`; NOTE: AuthResponse schema changed → OpenAPI/SDK regen needed before initiative close (planned in T-0269/T-0270).
- `cli/commands.rs serve()`: `utils::ui_pak::init()` after encryption init.
- `tests/fixtures.rs`: mints UI PAK; new `tests/integration/api/ui_pak.rs` with 5 tests (readonly identity, reads OK, POST/PUT/DELETE 403, diagnostics allowlist 201, admin not readonly); registered in api/mod.rs.
- FINDING (pre-existing bug, for T-0273): console `api::create_diagnostic` POSTs `/api/v1/diagnostics`, but no such broker route exists — real route is `POST /deployment-objects/:id/diagnostics`. The Fleet run-diagnostic button 404s today; fix during console work.
- NEXT: `cargo check -p brokkr-broker --tests` (Bash tool was temporarily unavailable), then `angreal tests integration -c brokkr-broker` — pending suite run alongside later broker tasks.

**2026-07-02 — VERIFIED**
- `cargo build -p brokkr-broker --tests` clean; `angreal tests unit brokkr-broker` 120 passed; `angreal tests integration brokkr-broker` **484 passed, 0 failed** (includes all 5 ui_pak tests). Done.