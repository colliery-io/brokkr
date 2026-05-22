---
id: qa-gap-no-contract-test-exercising
level: task
title: "QA gap: no contract test exercising generated SDKs against running broker"
short_code: "BROKKR-T-0154"
created_at: 2026-05-21T18:48:07.968636+00:00
updated_at: 2026-05-22T00:01:50.396927+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# QA gap: no contract test exercising generated SDKs against running broker

## Objective

We ship three generated SDKs (Rust/Python/TypeScript) but have no test that drives the **generated** SDK against a **running broker**. Three consumer-visible bugs ([[BROKKR-T-0151]], [[BROKKR-T-0152]], [[BROKKR-T-0153]]) were found the first time a human used the Rust SDK end-to-end, after the SDKs already shipped (v0.3.0 via BROKKR-T-0149). Spec-drift CI (BROKKR-T-0134) catches schema/server drift, but not "does the published SDK actually let you complete the headline flow."

Build an SDK contract / smoke-test layer that exercises the generated clients (not handcrafted reqwest) against an integration-test broker, covering at minimum the UAT walkthrough: create stack → annotate → add deployment object → label → target agent → observe deployment.

## Why this matters

- These bugs are spec/handler agreement failures and PAK-scope failures — exactly what an SDK-driven test would have surfaced.
- Strict-by-default generated clients (like progenitor) reject `Unexpected Response` codes, so any spec/server status-code drift is a hard break for Rust consumers.
- We maintain three SDKs in lockstep ([[project_release_versioning]]); doing this once per language gives us continuous protection on every release.

## Backlog Item Details

- **Type**: Tech Debt (QA gap)
- **Priority**: P1 — the meta-fix that prevents future BROKKR-T-0151/152/153-class escapes
- **Discovered**: 2026-05-21, from the same SDK-consumption session that produced the three bugs above

## Technical Debt Impact

- **Current Problems**: Spec-drift CI is necessary but insufficient. Handcrafted integration tests use the broker directly, bypassing the generated client surface. Bugs in SDK ergonomics, status codes, content-types, and auth scope reach consumers.
- **Benefits of Fixing**: Catch status-code mismatches, content-type mismatches, auth-scope regressions, and SDK ergonomic regressions before release. Tighter feedback loop for spec changes.
- **Risk Assessment**: Without this, every SDK release risks shipping a broken happy path; we discover via downstream consumers, which erodes trust.

## Acceptance Criteria

## Acceptance Criteria

- [x] An `angreal tests sdk-contract rust` target runs the generated Rust SDK (`brokkr-client` + `BrokkrClient` wrapper + raw progenitor surface) against a running broker — 3 scenarios passing
- [x] `angreal tests sdk-contract python` runs `brokkr-client-generated` against a running broker — 2 scenarios passing
- [x] `angreal tests sdk-contract typescript` runs `@colliery-io/brokkr-client` (openapi-fetch) against a running broker — 2 scenarios passing
- [x] CI wiring on PRs touching broker/spec/SDKs — added `.github/workflows/sdk_contract_tests.yml` (matrix over rust/python/typescript) and wired into `main.yml` with a `sdk_contract` path filter covering broker/spec/SDKs/contract-suite/angreal-task changes
- [x] Failure modes are legible — each suite asserts on typed `ErrorResponse.code` for the negative path (`target_generator_mismatch`), not on raw HTTP status

## Implementation Notes

- Reuse the UAT demo flow (BROKKR-T-0090) as the script — that's already the canonical end-to-end.
- Run against the same compose stack used by `angreal tests e2e`.
- Keep the test thin: this is a contract / smoke check, not a replacement for unit/integration tests.

## Status Updates

- 2026-05-21: Filed alongside the three bugs ([[BROKKR-T-0151]], [[BROKKR-T-0152]], [[BROKKR-T-0153]]) that prompted it.
- 2026-05-21: Implemented. Created `tests/sdk-contract/{rust,python,typescript}/`. Each suite drives its language's generated SDK through the full UAT walkthrough: admin creates generator+agent, generator PAK creates stack → label (`application/json` JSON-string body, exercises T-0152 fix) → annotation → deployment object → agent target (exercises T-0153 fix) → GET stack. Plus a negative-path: generator-targets-non-owned-stack expects typed 403 `ErrorResponse { code: "target_generator_mismatch" }`. New angreal task: `angreal tests sdk-contract {rust,python,typescript,all} [--skip-docker]`. Results: rust 3/3 ✓, python 2/2 ✓, typescript 2/2 ✓. All drift checks still clean.
- 2026-05-21: **Drift surfaced AND fixed in this pass** (per "immediate remediation of all found drift" directive):
  - `POST /stacks/{id}/deployment-objects` — introduced `CreateDeploymentObjectRequest { yaml_content: String, is_deletion_marker: bool }` as the wire DTO (separate from the DAL's `NewDeploymentObject`, which carries server-derived `yaml_checksum`). Handler signature changed `Json<serde_json::Value>` → `Json<CreateDeploymentObjectRequest>`; manual `payload[...]` parsing removed; utoipa `request_body` switched to the typed schema; registered in `openapi.rs`. Wire JSON shape unchanged.
  - `POST /agents` — introduced `CreateAgentResponse { agent: Agent, initial_pak: String }` and switched the handler return from `Json<Value>` to `Json<CreateAgentResponse>`; utoipa `body` updated; registered in `openapi.rs`. Wire JSON shape unchanged.
  - Spec + Python + TypeScript SDKs regen'd; all drift checks clean. SDK contract tests in all three languages tightened to assert on the new typed fields (e.g. Rust `agent_resp.agent.id` instead of `agent_raw.get("agent").get("id")`; Python `CreateAgentResponse.initial_pak`; TypeScript no longer casts `agentRes.data`).
  - Broker rebuilt via `angreal local rebuild broker` and full suite re-run end-to-end: rust 3/3, python 2/2, typescript 2/2.
- 2026-05-21: **Bearer-header investigation — closed.** Initial framing was wrong: the broker was already tolerating `Bearer <pak>` by accident. `PrefixedApiKey::from_string` splits on `_` and only requires 3 segments. The PAK shape `brokkr_BR3rVsDa_<long_token>` has 3 underscore-segments; `Bearer brokkr_BR3rVsDa_<long_token>` also has 3 (the space-containing first segment is unused). `long_token_hashed` only hashes the third segment, so both forms produce the same hash and authenticate against the same stored value. While investigating, also found and fixed a latent bug: `pak::verify_pak` / `pak::generate_pak_hash` used `.expect("Failed to parse PAK")`, which would panic the handler on any genuinely malformed Authorization header (yielding a 500). Changed both to return `Result<_, PakError>`; updated `auth_middleware` to return 401 on `PakError::Parse`. Also added a (purely ergonomic) `Bearer ` strip in `auth_middleware` so the convention is intentional rather than accidental.
- 2026-05-21: Final verification with everything in place — unit 84/0 (broker), integration 419/0 (broker), e2e 11/11 ✓, sdk-contract rust 3/0, python 2/0, typescript 2/0. All openapi drift checks clean.