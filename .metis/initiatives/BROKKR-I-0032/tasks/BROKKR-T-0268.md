---
id: broker-inject-ui-pak-into-served
level: task
title: "Broker: inject UI PAK into served console HTML"
short_code: "BROKKR-T-0268"
created_at: 2026-07-03T00:08:44.088414+00:00
updated_at: 2026-07-03T02:56:40.148533+00:00
parent: BROKKR-I-0032
blocked_by: [BROKKR-T-0267]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Broker: inject UI PAK into served console HTML

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

When the broker serves the console's `index.html`, inject the ephemeral UI PAK (from BROKKR-T-0267) as `<meta name="brokkr-ui-token" content="...">` so the WASM app can authenticate with zero operator configuration.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `index.html` responses (both `/` and SPA-fallback paths) contain the meta tag with the raw UI PAK
- [x] Non-HTML assets (wasm, js, css) are served byte-identical to the embedded bundle (no rewriting)
- [x] `Cache-Control: no-store` (or equivalent) on the injected HTML so a stale token doesn't outlive a broker restart
- [x] The `embed-ui`-disabled placeholder path still compiles and serves without a token (default build compiles + unit suite green)
- [x] Injection covered by unit tests (`inject_ui_token`: injects once before `</head>`, untouched without token); full serve-path check deferred to the embed-ui e2e walkthrough (noted below)

## Implementation Notes

### Technical Approach
- `crates/brokkr-broker/src/api/assets.rs` — `serve_asset` (L66) falls back to `index.html`; inject at serve time by replacing `</head>` (or a `<!--brokkr-ui-token-->` placeholder added to `crates/brokkr-web/index.html`) with the meta tag. String replace on the embedded bytes per request is fine (index.html is small); alternatively memoize the rewritten document in a `OnceLock`.
- Token source: the raw-PAK accessor added in T-0267 (`utils::ui_pak`).

### Dependencies
BROKKR-T-0267 (UI PAK must exist).

### Risk Considerations
- The token is visible to anyone who can fetch the console URL — by design (network access is the auth boundary; the credential is read-only). Document this in the module docs and initiative non-goals.

## Status Updates

**2026-07-02 — implemented, verification pending**
- `assets.rs`: `serve_index()` injects `<meta name="brokkr-ui-token">` via pure `inject_ui_token()` helper (unit-tested: injects once before `</head>`, no-op without token); `Cache-Control: no-store` on the SPA shell; hashed assets untouched. Both index paths (direct + SPA fallback) route through `serve_index()`.
- Note: full serve-path integration test requires an `embed-ui` build with a real `dist/`; covered by unit tests + e2e walkthrough instead.

**2026-07-02 — VERIFIED**
- Unit suite green (3 injection tests pass); broker builds clean with the default feature set. Serve-path smoke against an `embed-ui` build remains bundled with the release e2e walkthrough (project convention: e2e runs on release/nightly).