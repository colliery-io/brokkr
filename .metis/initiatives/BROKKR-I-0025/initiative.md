---
id: sdk-parity-retry-validation-and
level: initiative
title: "SDK parity: retry, validation, and surface harmonization"
short_code: "BROKKR-I-0025"
created_at: 2026-06-11T11:01:39.480559+00:00
updated_at: 2026-06-11T19:45:26.687658+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: sdk-parity-retry-validation-and
---

# SDK parity: retry, validation, and surface harmonization

## Context

Pre-0.7.0 parity sweep of the three SDK wrappers (Rust `crates/brokkr-client/src/wrapper.rs`, Python `sdks/python/brokkr/brokkr/client.py`, TS `sdks/typescript/brokkr-client/src/client.ts`). The good news, verified empirically: manifest bundles are **byte-identical** across all three (same folder → same sha256), so cross-SDK `apply` idempotency holds. The bad news: Python's `retry()` silently treats HTTP 5xx as success and fabricates `BrokkrError.status`; TS auto-retries non-idempotent POSTs (double-submit risk) and validates YAML with regexes (false-accepts that leak half-created stacks, false-rejects valid comment-only docs); plus smaller surface asymmetries.

## Goals & Non-Goals

**Goals:**
- The three wrappers behave identically for the documented surface: retry semantics, error status fidelity, manifest validation, public exports.
- No SDK auto-retries a non-idempotent POST.

**Non-Goals:**
- Byte-format changes to `read_manifests` (parity already verified — do not touch the concatenation).
- New features beyond filling the Python telemetry-helper gap.

## Detailed Design

Four tasks: Python retry/status fidelity (T-0211), TS non-idempotent retry removal (T-0212), TS real-YAML validation (T-0213), cross-SDK polish (T-0214). Contract tests in tests/sdk-contract/ are the arbiter; extend them where the suites diverge.

## Alternatives Considered

- Documenting the TS regex-validation asymmetry instead of fixing — rejected: it both leaks server-side side effects (stack created, labels applied, then ingest 400) and rejects valid bundles; a dynamic-imported YAML parse preserves the browser-clean constraint.
- Generating the wrappers from a shared spec — out of scope for this release; revisit if drift recurs.

## Implementation Plan

T-0211 and T-0212 first (correctness bugs), T-0213, then T-0214. Each language change runs its unit suite + the matching `angreal tests sdk-contract <lang>` on CI.