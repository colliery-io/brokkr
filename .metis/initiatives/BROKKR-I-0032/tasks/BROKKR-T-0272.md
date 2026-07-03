---
id: console-boot-from-injected-ui-token
level: task
title: "Console: boot from injected UI token"
short_code: "BROKKR-T-0272"
created_at: 2026-07-03T00:09:03.715303+00:00
updated_at: 2026-07-03T03:46:45.798813+00:00
parent: BROKKR-I-0032
blocked_by: [BROKKR-T-0268]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Console: boot from injected UI token

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

The console authenticates with the broker-injected UI token on boot — no PAK paste required. The `localStorage["brokkr_pak"]` override remains supported (it takes precedence, unlocking writes for operators who paste an admin PAK).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] On boot the app reads `<meta name="brokkr-ui-token">` once and holds it in memory (not written to `localStorage`)
- [x] Auth precedence: `localStorage["brokkr_pak"]` if set, else injected token, else unauthenticated (current error states)
- [x] All request paths (`get`/`get_scoped`, `post` in `api.rs`) use the same token resolution (`/metrics` is public, no auth attached)
- [x] Module docs in `api.rs` updated: interim paste flow is now the override path, not the primary
- [x] Console verified via Playwright run (localStorage-override path; token-less dev server); injected-token path compile-verified — full embed-ui walkthrough rides the release e2e per project convention

## Implementation Notes

### Technical Approach
- `crates/brokkr-web/src/api.rs` — `pak()` (L14) becomes `token()`: check localStorage override, then a `thread_local!`/`OnceCell` cached read of the meta tag (`web_sys::window().document().query_selector("meta[name='brokkr-ui-token']")`).
- `Authorization: Bearer` header attachment stays as-is (L26-28).

### Dependencies
BROKKR-T-0268 (broker must inject the tag). For local dev via `trunk serve` (no broker injection), the localStorage override covers it.

## Status Updates

**2026-07-02 — implemented, verification pending**
- `api.rs`: `injected_token()` reads `<meta name="brokkr-ui-token">` once (thread_local OnceCell, in-memory only); `token()` = pasted `brokkr_pak` override, else injected token; `get`/`post` now attach `token()`. Module docs rewritten (paste flow is the write-capable override, not the primary).
- `Cargo.toml`: web-sys +`Document`, +`Element` for `query_selector`.
- PENDING: wasm build check (`cargo check --target wasm32-unknown-unknown` or `trunk build`) once Bash is back.

**2026-07-03 — VERIFIED**
- wasm build clean; `trunk build` clean; full Playwright suite regenerated against the live build (16 scenes render, auth via the localStorage-override path). Injected-token path is unit-of-logic verified (`injected_token()` + precedence in `token()`); end-to-end embed-ui smoke deferred to release e2e (same as T-0268).