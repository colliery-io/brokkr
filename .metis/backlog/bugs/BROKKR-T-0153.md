---
id: generator-pak-cannot-post-agents
level: task
title: "Generator PAK cannot POST /agents/{id}/targets (403) — deploy flow unfinishable"
short_code: "BROKKR-T-0153"
created_at: 2026-05-21T18:48:06.789344+00:00
updated_at: 2026-05-21T20:21:51.478487+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Generator PAK cannot POST /agents/{id}/targets (403) — deploy flow unfinishable

## Objective

A generator PAK can create stacks and annotate them, but `POST /api/v1/agents/{id}/targets` returns `403 Forbidden`. Adding an agent target is what routes a stack to an agent for deployment, so the deploy flow is **unfinishable using only a generator PAK**.

We need to decide and document the intended auth model:
- Option A: Generator PAKs should be allowed to target agents (broaden the permission).
- Option B: This requires admin PAK by design — document it loudly, split demo/SDK examples, and return a structured `ErrorResponse` with required scope instead of a bare 403.

## Reproduction

1. Authenticate as a generator PAK
2. Create a stack → success
3. Annotate the stack → success
4. `POST /api/v1/agents/{id}/targets` → `403 Forbidden`

## Open Questions

- Is targeting an agent a "deployment authority" act that should require admin/operator scope?
- If generator PAKs are intentionally narrow, what's the recommended workflow for generators to hand off to a targeter?
- Should the 403 carry an `ErrorResponse` explaining required scope?

## Backlog Item Details

- **Type**: Bug (or design clarification)
- **Priority**: P0 — blocks the headline deploy demo through the SDK
- **Discovered**: 2026-05-21, while consuming `brokkr-client` (Rust)

## Acceptance Criteria

## Acceptance Criteria

- [x] Auth model for `/agents/{id}/targets` is documented in the API reference (utoipa `security(("admin_pak"), ("generator_pak"))` + 404 response documented; spec regen'd)
- [x] Generator PAK is permitted when it owns the stack; structured `ErrorResponse` (`target_generator_mismatch` / `target_create_denied` / `stack_not_found`) on denial
- [x] End-to-end demo (UAT walkthrough) succeeds with a generator PAK through the generated SDK — validated in [[qa-gap-no-contract-test-exercising]] / BROKKR-T-0154 (all three SDKs complete the full generator-PAK flow; negative-path tests confirm the typed `target_generator_mismatch` error contract)
- [x] N/A — ADR not needed; we chose the broaden-permissions path (Option A), not admin-only-by-design (Option B)

## Status Updates

- 2026-05-21: Filed from real consumption of generated Rust client.
- 2026-05-21: Fixed. Introduced `agents::authorize_target_mutation(dal, auth, stack_id)`: admin always allowed; generator allowed iff it owns the targeted stack; else 403 with structured `ErrorResponse` (`target_generator_mismatch` / `target_create_denied`); 404 (`stack_not_found`) when stack missing. Applied to both `POST /agents/{id}/targets` and `DELETE /agents/{id}/targets/{stack_id}`. utoipa `security` includes `generator_pak`; 404 documented. Agent label/annotation endpoints remain strictly admin-only (per design — generators must not label/annotate agents). Spec + Python/TS SDKs regen'd; drift checks pass; integration tests green.