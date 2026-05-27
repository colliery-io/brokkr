---
id: post-stacks-id-labels-returns-415
level: task
title: "POST /stacks/{id}/labels returns 415 Unsupported Media Type"
short_code: "BROKKR-T-0152"
created_at: 2026-05-21T18:48:05.218256+00:00
updated_at: 2026-05-21T20:21:49.964621+00:00
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

# POST /stacks/{id}/labels returns 415 Unsupported Media Type

## Objective

`POST /api/v1/stacks/{id}/labels` returns `415 Unsupported Media Type` for every Content-Type tried, including `text/plain` with a bare label string (which the OpenAPI spec appears to suggest). The endpoint is currently unreachable from the generated Rust SDK and ambiguous to all clients.

We need to (a) decide and document the canonical request format, (b) make the server accept it, and (c) regenerate clients so the call works end-to-end.

## Reproduction

1. Create a stack with a generator PAK
2. `POST /api/v1/stacks/{id}/labels` with `Content-Type: text/plain` and body `mylabel`
3. Server returns `415 Unsupported Media Type`
4. No Content-Type tried so far succeeds

## Open Questions

- Is the body a bare string, a JSON string (`"mylabel"`), or an object (`{"label": "mylabel"}`)?
- Should this be `application/json` or `text/plain`?
- Is the same shape used for `DELETE`/list label endpoints?

## Backlog Item Details

- **Type**: Bug
- **Priority**: P1 — endpoint is effectively unreachable from generated SDKs
- **Discovered**: 2026-05-21, while consuming `brokkr-client` (Rust)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Spec unambiguously declares request body schema and Content-Type (`application/json` with `type: string`)
- [x] Handler accepts the documented format and rejects others with a structured `ErrorResponse` (axum returns 415 on Content-Type mismatch; bad strings rejected by `NewStackLabel::new` as `ApiError::bad_request("invalid_label", ..)`)
- [x] Generated Rust/Python/TS clients can add a label using the SDK call (spec change picked up by all three regens)
- [x] Integration test covers it via the generated SDK — done in [[qa-gap-no-contract-test-exercising]] / BROKKR-T-0154 (all three SDKs successfully add a label via the typed `application/json` JSON-string body)

## Status Updates

- 2026-05-21: Filed from real consumption of generated Rust client.
- 2026-05-21: Fixed. Canonical request format is `application/json` with a JSON-string body (e.g. `"mylabel"`). Updated utoipa `request_body` on `stacks::add_label` and `templates::add_label` to declare `content_type = "application/json"`. `agents::add_label` is unchanged — it's admin-only and takes a `NewAgentLabel` struct (not a string body), so it was not affected. Spec + Python/TS SDKs regen'd; drift checks pass; integration tests green.