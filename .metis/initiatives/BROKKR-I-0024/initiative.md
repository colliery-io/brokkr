---
id: broker-api-correctness-error
level: initiative
title: "Broker API correctness: error mapping and auth scoping"
short_code: "BROKKR-I-0024"
created_at: 2026-06-11T11:01:39.433117+00:00
updated_at: 2026-06-11T15:34:10.621050+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: broker-api-correctness-error
---

# Broker API correctness: error mapping and auth scoping

## Context

The I-0021 contract suite caught `add_label` mapping a UNIQUE violation to a blanket 500 (fixed as 409 via `ApiError::from_diesel`). The pre-0.7.0 sweep audited all 129 `ApiError::internal` sites in `crates/brokkr-broker/src/api/v1/` and found **15 more** of the same class masking realistic 409/422/404s, plus auth-scoping gaps (path/body mismatches, an unscoped event-listing endpoint), 225 DAL methods that panic on DB pool exhaustion with no catch-panic layer, and a few missing input validations / a missing unique index.

## Goals & Non-Goals

**Goals:**
- Constraint violations surface as typed 4xx (409 `unique_violation`, 422 FK) everywhere, not 500 — idempotent clients depend on it.
- Path-id and body-id always agree; event listing scoped to the caller.
- DB pool exhaustion returns 500s instead of dropped connections.

**Non-Goals:**
- New endpoints or wire-format changes; SDK-side changes (BROKKR-I-0025).
- A general authz framework — only the specific gaps found.

## Detailed Design

Four tasks: the `from_diesel` sweep (T-0207), auth scoping (T-0208), pool-panic elimination + CatchPanicLayer (T-0209), input validation + `stack_annotations` unique index (T-0210). The fix pattern for T-0207 is the proven one from the add_label fix: `.map_err(|e| ApiError::from_diesel(e, "context"))` + an integration regression test per duplicate path.

## Alternatives Considered

- A blanket `impl From` swap at every `map_err` site mechanically — rejected: ~103 of the 129 sites are genuinely internal (reads, server-generated values); converting them would mislabel real 500s. Fix only the 15 audited sites.
- Global panic-to-500 middleware only (no DAL fix) — rejected as primary fix: panicking on pool exhaustion still loses request context and trips tokio worker churn; do both.

## Implementation Plan

T-0207 first (purely mechanical, immediately user-visible for idempotent SDK flows), then T-0208 (review auth changes carefully — one is a behavior change for any consumer enumerating events), T-0209, T-0210 (includes a migration; run `angreal models migrations`).