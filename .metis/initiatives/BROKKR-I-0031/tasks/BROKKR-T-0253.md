---
id: slice-1b-broker-serves-the-brokkr
level: task
title: "Slice 1b: broker serves the brokkr-web wasm bundle + SPA fallback; Trunk build wired into image/angreal"
short_code: "BROKKR-T-0253"
created_at: 2026-06-28T01:32:27.739206+00:00
updated_at: 2026-06-28T23:23:48.457035+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Slice 1b: broker serves brokkr-web wasm + SPA fallback; build wiring

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Make `brokkr-broker` serve the built `brokkr-web` wasm bundle so the console is reachable
at the broker with **zero extra infrastructure**, and wire the Trunk build into the
broker image + `angreal` build so it ships in lockstep.

### Type
- [x] Feature — broker-served UI (hosting half of the walking skeleton)

## Acceptance Criteria

- [x] Broker axum app serves the wasm + **SPA fallback** on the **outer** router via
      `api::assets::attach(app)`, added AFTER the `/api/v1` nest + route groups so the API
      wins; `/api/*` and `/internal/*` 404 honestly (`is_api_path`). Mirrors `skadi-api`.
- [x] Static assets **embedded in the binary** via `rust-embed` (`#[folder="../brokkr-web/dist"]`)
      behind an **`embed-ui` feature** (default OFF → placeholder, so the host workspace/tests
      build with no `dist/`). `cargo check` green both with and without the feature.
- [x] Auth unchanged — it's scoped to the `/api/v1` nest (`v1/mod.rs`), so the outer static
      fallback serves the shell without a PAK. The UI's API-call **read-access auth boundary**
      remains a deferred decision (ADR-0010).
- [ ] **Live verification** — index.html + assets load against a running broker (needs the
      broker built `--features embed-ui` + a DB; do with the integration stack). *(pending)*
- [ ] **Build wiring** — `trunk build` of `brokkr-web` + `--features embed-ui` into
      `docker/Dockerfile.broker` and an `angreal` task; CI builds the wasm. *(pending)*
- [ ] `index.html` + hashed wasm/css/font assets load and the shell (from 1a) renders in a
      browser against a running broker.
- [ ] **Build wiring**: Trunk build of `crates/brokkr-web` integrated into the broker
      container build (`docker/Dockerfile.broker`) and an `angreal` build task; CI builds
      the wasm. Image-size impact noted.
- [ ] SPA deep-link refresh works (fallback serves `index.html` for non-API, non-asset
      paths).

## Implementation Notes

### Technical Approach
- Reference: `crates/skadi-api/src/serve.rs` (`Router::new().nest("/api/v1", api)` then
  static UI + SPA fallback last) and `assets.rs`. Brokkr's broker router does the same.
- Prefer **embedding** the bundle in the broker binary (single artifact, matches the
  "ships with the broker" intent) unless image-build constraints favour `ServeDir`.

### Dependencies
- Depends on [[BROKKR-T-0252]] (the `brokkr-web` crate + build output to serve).

### Risk Considerations
- Route precedence: the API nest MUST win every route it owns; the fallback only catches
  the rest. Add a test asserting `/api/v1/*` is never shadowed by the SPA fallback.
- Trunk in the Docker/CI build (toolchain availability, build time, image size).

## Status Updates

**2026-06-28 — Serving wired & compiling.** Added `crates/brokkr-broker/src/api/assets.rs`
(generic `attach<S>` + `is_api_path` + cfg'd `serve_asset`), declared `pub mod assets;`, and
attached the fallback on the outer router in `configure_api_routes` (after `CatchPanicLayer`).
`Cargo.toml`: optional `rust-embed`/`mime_guess` + `embed-ui` feature. `cargo check -p
brokkr-broker` green **and** `--features embed-ui` green (embeds `crates/brokkr-web/dist`).
Auth confirmed scoped to `/api/v1`, so the console shell serves PAK-free.

**Remaining:** build wiring (Dockerfile + angreal trunk step) and a live run of the broker
`--features embed-ui` to curl `/` (console) + `/api/v1/...` (API) + `/api/v1/nope` (404).
Best done with the integration stack; add the route-precedence test there.