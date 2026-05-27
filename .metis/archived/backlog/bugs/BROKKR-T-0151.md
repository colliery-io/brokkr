---
id: post-stack-endpoints-return-200
level: task
title: "POST stack endpoints return 200 instead of spec'd 201"
short_code: "BROKKR-T-0151"
created_at: 2026-05-21T18:48:03.625457+00:00
updated_at: 2026-05-21T20:21:48.959620+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# POST stack endpoints return 200 instead of spec'd 201

## Objective

Server returns HTTP 200 on stack-creation endpoints where the OpenAPI spec declares 201 Created. The progenitor-generated Rust client validates response status against the spec and rejects these responses as `Unexpected Response`, making the generated SDK unusable for stack creation flows without manual workarounds.

Either the server must return 201 or the spec must be changed to 200 — the server and spec must agree. 201 is the correct semantic for resource creation; prefer fixing the server.

## Affected Endpoints

- `POST /api/v1/stacks` — returns 200, spec says 201
- `POST /api/v1/stacks/{id}/annotations` — returns 200, spec says 201
- `POST /api/v1/stacks/{id}/deployment-objects` — returns 200, spec says 201

## Reproduction

1. Use a valid generator PAK
2. `POST /api/v1/stacks` with a valid stack body
3. Observe server returns `200 OK`; spec declares `201 Created`
4. Rust `brokkr-client` (progenitor) returns `Error::UnexpectedResponse`

## Backlog Item Details

- **Type**: Bug
- **Priority**: P1 — blocks Rust SDK consumers from completing core stack flow
- **Discovered**: 2026-05-21, while consuming `brokkr-client` (Rust)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] All POST resource-creation endpoints return 201 on successful creation (swept broker-wide, not just the three originally listed)
- [x] Generated Rust client accepts the responses without `UnexpectedResponse` (vendored spec at `crates/brokkr-client/spec/brokkr-v1.json` is byte-identical to canonical, declares 201)
- [x] Integration test exercises the success path through the generated SDK — done in [[qa-gap-no-contract-test-exercising]] / BROKKR-T-0154 (rust/python/ts UAT walkthroughs all assert 201 on create-stack via SDK)
- [x] OpenAPI spec regenerated and spec-drift CI passes (`angreal openapi check` clean)

## Implementation Notes

Stack-creation handlers live in the broker (likely `crates/brokkr-broker/src/api/v1/`). Change to `StatusCode::CREATED` (201), then `angreal openapi export` and ensure BROKKR-T-0134 spec-drift CI passes.

Related: [[qa-gap-no-contract-test-exercising]] — would have caught this before release.

## Status Updates

- 2026-05-21: Filed from real consumption of generated Rust client.
- 2026-05-21: Fixed. Swept all POST resource-creation handlers (`stacks.rs`, `agents.rs`, `generators.rs`, `templates.rs`) to return `(StatusCode::CREATED, Json(..))` with matching `(status = 201, ...)` utoipa attrs. `openapi/brokkr-v1.json` regen'd; Python and TypeScript SDKs regen'd; all drift checks pass; vendored Rust client spec byte-identical to canonical. Integration tests updated (17 assertions `OK` → `CREATED`); `angreal tests unit all` 283 passed; `angreal tests integration brokkr-broker` 419 passed / 0 failed. E2E to be run externally.