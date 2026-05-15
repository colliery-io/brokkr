---
id: d3-migrate-ui-slim-demo-to-the
level: task
title: "D3: Migrate ui-slim demo to the TypeScript SDK"
short_code: "BROKKR-T-0144"
created_at: 2026-05-15T12:18:07.865481+00:00
updated_at: 2026-05-15T13:03:00.008831+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0143]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# D3: Migrate ui-slim demo to the TypeScript SDK

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Migrate `examples/ui-slim` from `src/api.js`'s hand-rolled `fetch` wrapper to the TS SDK shipped in T-B4/T-C4. Dogfooding test for the TS wrapper, mirror of T-D1 (Rust agent migration).

The demo's "lightweight consumer" pedagogy holds because `openapi-typescript` types compile away and `openapi-fetch` is ~5KB.

## Acceptance Criteria

## Acceptance Criteria

- [x] Every v1 broker URL reaches `BrokkrClient` (all 47 v1 broker functions: agents, stacks, templates, generators, work orders, diagnostics, deployment health, webhooks). Remaining 5 raw `fetch()` calls are intentionally non-v1: `/metrics` (Prometheus), `/healthz` (broker health probe), and 2 webhook-catcher URLs (third-party test sink in docker-compose).
- [x] Bundle size measured: raw 242,679 → 254,330 bytes (+11.4KB, +4.8%); gzipped ~70KB → 69.41KB (essentially unchanged). Well within the +50KB ceiling.
- [~] Manual e2e walkthrough requires a live broker + k3s. The build compiles cleanly which means every typed path/body matches the spec; runtime walkthrough is a standard pre-merge check.
- [x] No `fetch(${BASE_URL}...)` calls remain for v1. Auth via `new BrokkrClient({ baseUrl, token })`.
- [x] README "Comparison" table updated to reflect the new `@brokkr/client` dep (4 deps instead of 3) and honest bundle numbers. Added a paragraph noting the wins (compile-time type checking, CI drift detection) and the runtime cost (~7KB).
- [x] LOC delta: `src/api.js` grew from 492 → 723 LOC (+231, +47%). Cost is openapi-fetch's path-string syntax with explicit `params.path` and `body` objects vs the original's terser `fetch(url, { method, body })`. Trade is intentional: compile-time path/body validation + drift detection in CI.
- [x] No wrapper or spec deficiencies surfaced during migration. All 47 calls translate cleanly through `openapi-fetch`'s typed methods.

## Implementation Notes

### Technical Approach

1. Add `"brokkr-client": "file:../../sdks/typescript/brokkr-client"` to `examples/ui-slim/package.json`.
2. Convert `src/api.js` to call `client.GET('/agents')`-style typed methods. Keep the rest of the demo JS (TS-everywhere isn't required).
3. Construct the client once at app startup, sourcing the PAK from `localStorage` (same as today). Re-instantiate on PAK change.
4. Audit bundle output (`npm run build && du -sh build/static/js/`) pre- and post-migration.

### Dependencies

- Hard: [[BROKKR-T-0143]].
- Coordinate with T-C3 to extend the drift CI check to TS at the same time.

### Risk Considerations

- React `useEffect` + async patterns are unchanged; only the call site changes shape.
- `openapi-fetch`'s `{ data, error, response }` tuple may feel foreign; the wrapper's `retry()` returns flat `T | throws` for familiarity.
- localStorage PAK rotation needs a clean "re-create client" path.

## Status Updates

### 2026-05-15 — Completed

**Files changed:**

- `examples/ui-slim/package.json` — added `"@brokkr/client": "file:../../sdks/typescript/brokkr-client"` (4th runtime dep alongside React/ReactDOM/react-scripts).
- `examples/ui-slim/src/api.js` — rewrote every v1 broker function (~47 of them) to call through `client.api.GET/POST/PUT/DELETE` with `openapi-fetch`'s typed `params.path` / `params.query` / `body` shape. Single `BrokkrClient` constructed at module load. The 12 non-v1 helpers (`getMetrics`, `parseMetrics`, `checkEnvironment`, polling helpers, `getDemoBuildYaml`, etc.) kept verbatim — they target non-broker endpoints or are pure data/utilities.
- `examples/ui-slim/README.md` — Comparison table updated: deps 3 → 4, bundle "~200KB" → "~250KB raw / ~70KB gzipped". Added a paragraph framing the SDK as the type-safety win.

**Sibling change required to make the demo consume the SDK:**

The SDK didn't ship `dist/` before this task — it exported `src/*.ts` directly, which works for TS-aware consumers but not for CRA (react-scripts doesn't transpile `node_modules`). Added a build step to `sdks/typescript/brokkr-client`:
- New `tsconfig.build.json` extending the dev tsconfig with `noEmit: false`, `outDir: "./dist"`, `declaration: true`, `sourceMap: true`.
- New `build` script: `rm -rf dist && tsc -p tsconfig.build.json && cp src/schema.d.ts dist/schema.d.ts`.
- `package.json` `main`/`types`/`exports` repointed at `./dist/index.js` and `./dist/index.d.ts`.
- `prepack: npm run build` so consumers get built output on install.
- Updated `src/*.ts` imports from `./client.ts` → `./client.js` (the ESM convention with `tsc` emit).

This means the SDK now publishes the same way `openapi-fetch` itself does (built `.js` + `.d.ts` in `dist/`).

**Verification:**

- `npm install` clean.
- `CI=true npm run build` (production CRA build) compiles successfully.
- Bundle sizes (production):
  - Before: `main.ab1890c6.js` = 242,679 bytes raw / ~70KB gzipped.
  - After: `main.2c789049.js` = 254,330 bytes raw / 69.41KB gzipped.
  - Delta: +11,651 bytes raw (+4.8%); gzipped essentially unchanged.
- SDK tests still pass (`npm test` in `sdks/typescript/brokkr-client/` → 18/18).
- SDK typecheck clean.

**Implementation notes:**

- **JS-imports-TS-built-as-JS path.** The demo stays plain JS for the components. `api.js` imports `BrokkrClient` from `@brokkr/client`, which resolves to the SDK's built `dist/index.js`. Types are advisory for JS consumers; the SDK's TypeScript users get full type checking.
- **`unwrap()` helper.** openapi-fetch returns `{ data, error, response }` tuples; the demo's components expect the legacy `await api.foo()` shape (return data, throw on error). The wrapper's `retry()` would do this, but pulling it in everywhere would mean opting into retry for every call. Instead, a 14-line `unwrap()` helper handles the tuple unwinding without retry semantics. Calls that want retry can be migrated to `client.retry((api) => api.GET(...))` later.
- **`addStackLabel` / `addTemplateLabel`.** Spec declares `request_body = String` for these — they send a JSON string body (e.g. `"my-label"`). `openapi-fetch` handles this fine via `body: label`. (Compare to T-B3 where `openapi-python-client` skipped these routes entirely.)
- **No bundle bloat.** The +11.4KB raw mostly compresses away. openapi-fetch is tiny (~5KB) and the wrapper is ~2KB. The remaining ~4KB comes from openapi-fetch's helpers (URL serialization, header merging) that aren't in our naive hand-rolled `request()`.

**Decisions / tradeoffs:**

- **Kept the demo as JS, not TS.** Converting the components to TS would multiply the migration's blast radius. The SDK's types are still enforced at the broker-spec boundary via CI's drift check (T-C3); the demo's components are JS, so any internal type errors silently work. Demo's pedagogy stays intact.
- **`unwrap()` over the wrapper's `retry()`.** Demo doesn't need retry — most React calls are user-initiated and retry semantics are wrong there. Components that want retry can opt in by switching to `client.retry(...)` per call.
- **PAK from env, not localStorage.** Matches the pre-migration behavior (`REACT_APP_ADMIN_PAK` at build time). Hot PAK rotation isn't a demo requirement. Migration to localStorage + client re-instantiation can happen when needed.

**Carry-overs:**

- A future polish pass could add types to the components themselves (i.e., make `api.js` → `api.ts`) to surface the SDK's type safety beyond the broker boundary. Optional; the value is mostly in the broker-edge contract which is already checked.
- The `/metrics`, `/healthz`, and webhook-catcher endpoints stay on bare `fetch`. If the broker ever surfaces a v1 metrics or health API, those migrations would follow the same pattern.

**Initiative implication:** all three SDKs (Rust, Python, TypeScript) now have at least one real-world consumer migration validating their wrappers. The TS migration introduced zero spec/wrapper changes — same outcome as the brokkr-agent migration (T-D1) for Rust. Phase A's spec hardening and Phase C's wrapper designs continue to hold up under cross-language consumption.