---
id: c4-typescript-sdk-ergonomic-wrapper
level: task
title: "C4: TypeScript SDK ergonomic wrapper"
short_code: "BROKKR-T-0143"
created_at: 2026-05-15T12:18:06.936694+00:00
updated_at: 2026-05-15T12:44:44.637733+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0142]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# C4: TypeScript SDK ergonomic wrapper

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Add a thin ergonomic layer on top of `openapi-fetch` (from T-B4) so the TypeScript SDK presents the same single-credential, retry-helper, typed-error contract as the Rust (T-C1) and Python (T-C2) wrappers.

`openapi-fetch` already provides typed methods and lets callers inject default headers, so much of what C1 and C2 do is already "free" here. The wrapper's job is to add retry, surface a `BrokkrError` class for `.code` matching, and provide a single-arg constructor that mirrors the other languages' shapes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `BrokkrClient` in `src/client.ts` (183 LOC). Constructor `new BrokkrClient({ baseUrl, token?, requestTimeoutMs?, maxRetries?, initialBackoffMs? })`. Single `token` parameter injects `Authorization: Bearer <token>` via `createClient({ headers })`. `.api` exposes the typed `openapi-fetch` client. `.retry(op)` async helper with exponential backoff (initial × 2^attempt, capped at 10s) — same retryable status set as C1/C2 (408/429/502/503/504 + transport errors).
- [x] `BrokkrError` in `src/error.ts` (85 LOC), extends `Error`. `.code: string | undefined`, `.status: number | undefined`, `.response: ErrorResponse | undefined`, `.isRetryable()`. Three constructors: `fromResponse`, `fromTransport`, `fromOpenapiFetch` for the wrapper's tuple-style errors.
- [x] 13 wrapper-tests + 5 surface-tests = **18/18 pass**. Cover construction validation, retryable status set parity across both directions, transport error retryability, retry succeeds-first / stops-after-max / short-circuits-on-non-retryable, retry-then-success on a 503, and Authorization header injection.
- [x] `tsc --noEmit` clean. No `any` in the public surface; the wrapper uses generics + `FetchResult<T>` for typed propagation.

## Implementation Notes

### Technical Approach

1. `openapi-fetch`'s `createClient` returns `{ GET, POST, PUT, PATCH, DELETE, HEAD }` keyed by path. Wrapper holds this client + config.
2. Auth: pass `headers: { Authorization: \`Bearer ${token}\` }` to `createClient`; openapi-fetch applies them on every request.
3. Retry: generic helper `async retry<T>(op: (api) => Promise<T>): Promise<T>` with try/catch, status classification, exponential sleep, re-throw on max attempts.
4. Error classification: openapi-fetch returns `{ data, error, response }` tuples (no throw on HTTP errors). Inspect `response.status` + `error` payload (typed as `ErrorResponse` via T-B4's schema types) to build `BrokkrError`.
5. Match Rust/Python tradeoffs: opt-in retry per call, async-only, no pagination.

### Dependencies

- Hard: [[BROKKR-T-0142]].

### Risk Considerations

- `fetch` timeout portability — use `AbortController` instead of relying on runtime-specific timeout options.
- ESM-only is fine for the modern frontend target (`ui-slim`).
- Wrapper LOC budget: target ≤200, mirroring C1/C2 minimalism.

## Status Updates

### 2026-05-15 — Completed

**Files added:**

- `sdks/typescript/brokkr-client/src/error.ts` (85 LOC) — `BrokkrError` class extending `Error`, with `code`, `status`, `response` fields and `isRetryable()`. Three constructors: `fromResponse`, `fromTransport`, `fromOpenapiFetch`. Shared retryable status set `{408, 429, 502, 503, 504}` matches Rust (T-C1) and Python (T-C2) exactly.
- `sdks/typescript/brokkr-client/src/client.ts` (183 LOC) — `BrokkrClient` class. Constructor accepts `{ baseUrl, token?, requestTimeoutMs?, maxRetries?, initialBackoffMs? }`. Auth via `createClient({ headers: { Authorization: \`Bearer ${token}\` } })`. Timeout via custom `fetch` wrapping `AbortController` + signal merging (portable across Node, browsers, Bun). `retry()` is the opt-in helper.
- `sdks/typescript/brokkr-client/src/wrapper.test.ts` — 13 vitest tests.
- Updated `src/index.ts` to re-export `BrokkrClient`, `BrokkrClientOptions`, `BrokkrError`.

**Design parity across the three wrappers:**

| concern | C1 (Rust) | C2 (Python) | C4 (TypeScript) |
|---|---|---|---|
| construction | `BrokkrClient::builder(url).token(pak).build()?` | `BrokkrClient(url, token="...")` | `new BrokkrClient({ baseUrl, token })` |
| api access | `client.api()` | `client.api` | `client.api` |
| retry helper | `client.retry(\|c\| async {...})` | `await client.retry(op)` | `await client.retry((api) => …)` |
| retryable | `{408, 429, 502, 503, 504}` | identical | identical |
| backoff | `initial * 2^attempt`, cap 10s | identical | identical |
| max retries default | 3 | 3 | 3 |
| typed error | `BrokkrError::Api(ErrorResponse, status)` | `BrokkrError(message, code, status, response)` | `class BrokkrError extends Error` |
| pagination | not implemented | not implemented | not implemented |

**Verification:**

- `npm run typecheck` clean.
- `npm test` → 18/18 pass (5 surface + 13 wrapper).
- LOC budget: client.ts 183 (under the 200 guardrail); error.ts 85; index.ts 58. Total wrapper surface: 326 LOC including re-exports.

**Test approach:**

Tests use a `scriptedFetch` helper that stubs the global `fetch` and records calls (URL + headers extracted from both `Request` objects and `RequestInit`). Initial attempt with `vi.useFakeTimers()` deadlocked the Vitest worker because the `AbortController` + `setTimeout` interaction in the custom-fetch wrapper produces uncancellable promise chains under fake timers. Switched to real timers with 1ms backoffs — tests finish in ~14ms each, plenty fast.

**Implementation notes worth flagging for T-D3 (ui-slim migration):**

- `openapi-fetch`'s return shape is `{ data, error, response }` (no throws on HTTP errors). The wrapper's `retry()` unwraps this to a flat `T | throws BrokkrError` shape that React `useEffect` patterns expect. ui-slim's existing `await request(...)` style fits cleanly.
- The `token` parameter is settable only at construction. Token rotation pattern in the demo: re-instantiate the client on PAK change (already what the demo would need with any approach).
- Custom-fetch wrapper merges caller-supplied `AbortSignal` with the timeout signal, so `<button onClick={() => abortController.abort()}>` patterns still work.
- AbortController setTimeout is cleared via `.finally(() => clearTimeout(timer))` — no leaked timers even when the fetch resolves before the deadline.

**Decisions / tradeoffs:**

- **`token` instead of a header injector function.** Could have accepted `getToken: () => string` for dynamic credentials. Chose the simpler model because (a) it matches C1/C2's constructor shape, (b) consumers rotating PAKs just rebuild the client (the only consumer doing this is the demo).
- **`AbortController` over per-runtime timeout options.** Reqwest's `timeout` is non-standard in fetch. `AbortController` is universal across Node 18+, browsers, Deno, and Bun. Small cost in code (signal merging) for full portability.
- **No retry middleware library.** Same as C1/C2 — hand-rolled is ~30 LOC. Pulling in `p-retry` or similar would double our runtime dep count.

**Carry-overs:**

- T-D3 (ui-slim) can adopt the wrapper directly. Bundle math: openapi-fetch (~5KB) + wrapper (~2KB minified) ≈ 7KB added to the demo's current 200KB baseline. Well under the +50KB ceiling.