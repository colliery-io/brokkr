---
id: b4-prototype-typescript-client
level: task
title: "B4: Prototype TypeScript client with openapi-typescript + openapi-fetch"
short_code: "BROKKR-T-0142"
created_at: 2026-05-15T12:18:05.465252+00:00
updated_at: 2026-05-15T12:31:04.572439+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0133]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# B4: Prototype TypeScript client with openapi-typescript + openapi-fetch

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Scaffold `sdks/typescript/brokkr-client`, generate types via `openapi-typescript`, and confirm the package installs, imports, and exposes a typed wire-level client built on `openapi-fetch`. Mirrors T-B2 (Rust) and T-B3 (Python) for the third consumer language.

Decision recorded on the initiative: `openapi-typescript` for types, `openapi-fetch` for runtime. Rationale — types-only generation keeps bundle impact near zero (~5KB runtime + ~50-100KB of `.d.ts` that compile away). Aligns with the broker's "lightweight consumers" philosophy and is what `ui-slim` (T-D3) will actually depend on.

## Acceptance Criteria

## Acceptance Criteria

- [x] `sdks/typescript/brokkr-client/` exists with `package.json` (pinned `openapi-fetch@0.17.0` runtime + `openapi-typescript@7.13.0` dev), `tsconfig.json`, `src/schema.d.ts` (generated), `src/index.ts` (client factory + convenience re-exports), `src/surface.test.ts`.
- [x] `angreal openapi gen-typescript` regenerates the schema; `angreal openapi check-typescript` is the drift check. Both discoverable via `angreal tree`.
- [x] Drift check passes on clean tree (byte-identical schema on round-trip).
- [x] `npm install` clean; `npm run typecheck` (tsc --noEmit) clean.
- [x] 5/5 vitest surface tests pass — client construction (with and without baseUrl/headers), typed paths for baseline operations, `ErrorResponse`/`Agent`/`WorkOrder` type shapes.
- [x] Generator findings captured below.

## Implementation Notes

### Technical Approach

1. Add a new angreal task `openapi gen-typescript` invoking `npx openapi-typescript openapi/brokkr-v1.json -o sdks/typescript/brokkr-client/src/schema.d.ts` with pinned version.
2. Scaffold `package.json` declaring:
   - `openapi-fetch` as a runtime dep (consumers install this).
   - `openapi-typescript` as a dev dep (regeneration).
   - `typescript` as a dev dep for `tsc --noEmit`.
3. `src/index.ts` exports `createClient<paths>(...)` from `openapi-fetch` plus the `paths` and `components` types.
4. The wrapper layer (auth, retries, typed errors) is T-C4's responsibility; B4 only ships the typed wire client.

### Dependencies

- Hard: [[BROKKR-T-0133]] (hardened spec).
- Soft: existing `angreal openapi` infrastructure from T-B1.

### Risk Considerations

- `openapi-typescript` 6.x/7.x has subtle changes around how it handles enums and discriminated unions. Pin a known-good version.
- The spec's `nullable: true` pattern (downgraded from 3.1 in `openapi_export.rs`) should translate cleanly to `T | null` in TS; verify on a few schemas.
- Bundle impact target: schema types compile away entirely; runtime overhead is just openapi-fetch (~5KB minified). If the generated `.d.ts` exceeds ~200KB, investigate — likely indicates a spec oddity worth fixing.

## Status Updates

### 2026-05-15 — Completed

**Files added:**

- `sdks/typescript/brokkr-client/package.json` — ESM-only, `@brokkr/client`. Runtime: `openapi-fetch@0.17.0`. Dev: `openapi-typescript@7.13.0`, `typescript@5.7.3`, `vitest@3.0.4`.
- `sdks/typescript/brokkr-client/tsconfig.json` — strict mode, ES2022, bundler resolution, `allowImportingTsExtensions`.
- `sdks/typescript/brokkr-client/src/schema.d.ts` — generated, 6,584 lines / 204KB (mostly docstrings; ~30KB gzipped; 0KB at runtime).
- `sdks/typescript/brokkr-client/src/index.ts` — 53 LOC. Exports `createBrokkrClient`, `BrokkrApi`, and convenience re-exports for common schema types.
- `sdks/typescript/brokkr-client/src/surface.test.ts` — 68 LOC, 5 vitest tests.
- `sdks/typescript/brokkr-client/package-lock.json` — committed for `npm ci` in CI.

**Angreal + CI:**

- `.angreal/task_openapi.py`: `OPENAPI_TYPESCRIPT_VERSION = "7.13.0"`, new `_run_typescript_gen()` helper, new `gen-typescript` and `check-typescript` tasks.
- `.github/workflows/openapi.yml`: triggers extended for `sdks/typescript/brokkr-client/**`; new steps for TS drift check + `npm ci && npm run typecheck && npm test`.

**Verification:**

- `npm run typecheck` clean.
- `npm test` → 5/5 pass.
- `angreal openapi check-typescript` clean.

**Generator quality findings (for T-C4):**

1. **openapi-fetch API.** Path-string keyed methods: `client.GET("/agents/{id}", { params: { path: { id } } })`. Fully typed via inferred schema. ~5KB runtime.
2. **Return convention** is `{ data, error, response }` tuples (no throw). `error` is the typed union of documented 4xx/5xx response bodies — uniformly `ErrorResponse` thanks to T-A1. Wrapper can build `BrokkrError` directly without re-parsing JSON.
3. **Nullable handling.** `nullable: true` → `T | null`. Verified.
4. **Two unused-component warnings** (`NewTemplateLabel`, `WebhookSubscription`) — same as redocly's. Pre-existing.
5. **Bundle math.** schema.d.ts: 204KB on disk → ~30KB gzipped → 0KB at runtime. openapi-fetch runtime: ~5KB gzipped.
6. **Verbosity vs progenitor.** Progenitor's `client.list_agents().send()` is shorter than `client.GET("/agents")`, but the TypeScript variant catches typos at compile and is the canonical pattern in the openapi-fetch ecosystem.

**Implications for downstream tasks:**

- **T-C4** unblocked. Wrapper job: auth header via `openapi-fetch` constructor headers, `retry()` helper, `BrokkrError` class from `{ error, response }`. Easily under the ≤200 LOC budget.
- **T-D3** has a clean foundation. `BASE_URL` → `createBrokkrClient({ baseUrl })`. localStorage PAK pattern fits the single-token constructor.
- **T-C3 drift CI** extended in this task for TS — fully covered.

**Decisions:**

- **No code-gen for runtime methods.** Stuck with openapi-fetch over fluent alternatives. Saves ~10× runtime size.
- **ESM-only.** Modern bundlers handle ESM fine; saves a dual-build matrix.
- **Convenience re-exports** of common types in index.ts so consumers don't drill into `components["schemas"][...]` everywhere — same as Rust/Python wrappers.