---
id: c1-rust-sdk-ergonomic-wrapper
level: task
title: "C1: Rust SDK ergonomic wrapper"
short_code: "BROKKR-T-0137"
created_at: 2026-05-14T18:26:24.751621+00:00
updated_at: 2026-05-15T00:45:31.140408+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0135]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# C1: Rust SDK ergonomic wrapper

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Build the thin hand-written ergonomic layer on top of the `progenitor`-generated Rust client (from B2). The wrapper hides wire-level annoyances and presents a polished surface to brokkr-agent and external Rust consumers.

## Acceptance Criteria

## Acceptance Criteria

- [x] `BrokkrClient` in `crates/brokkr-client/src/wrapper.rs`, constructed via `BrokkrClient::builder(base_url).token(pak).build()`. Holds the generated `Client` plus retry config; exposed via `.api()`.
- [x] Auth surfaced as a single `token(pak)` builder field. PAK injected as the `Authorization` header via `reqwest::Client::default_headers`. The three spec security schemes collapse to one wrapper credential — A1 hidden, confirmed by B2 findings.
- [x] Retry/backoff: `BrokkrClient::retry(|c| async { ... })` runs the closure with exponential backoff (`initial * 2^attempt`, capped at 10s) on retryable failures. Defaults to 3 attempts and 200ms initial backoff; both configurable. Retryability classification matches progenitor's: transport errors + 408/429/502/503/504. No retries on 4xx or non-transient 5xx. Retry is **opt-in** per call — callers wrap only idempotent ops.
- [x] Typed error surface: `BrokkrError { Api(ErrorResponse, StatusCode), Transport(reqwest::Error), UnexpectedResponse{..}, InvalidRequest(String) }`. `From<progenitor_client::Error<ErrorResponse>>` for `?`-style propagation. Helpers: `code() -> Option<&str>`, `status() -> Option<StatusCode>`, `is_retryable() -> bool`. Implements `Display` + `Error`.
- [~] Pagination — no-op. v1 endpoints return full collections without cursors (audit-confirmed). Module docs flag that `Stream` adapters belong here when pagination is added. No code shipped.
- [x] Wrapper file is **283 LOC** including tests and module docs. Well under the 500 LOC guardrail.
- [x] Unit tests: 7 in `wrapper::tests` (auth-header injection valid + rejected, builder defaults, error code extraction, retryability classification, retry stops after `max_retries`, retry returns immediately on non-retryable). All pass. `cargo test -p brokkr-client` → 10/10 (7 wrapper + 3 surface). Integration test against a real broker deferred to T-D1 (agent migration) — same DB-dependency reasoning as B2/B3.

## Implementation Notes

### Technical Approach

1. Build on top of the generated low-level client. Re-export key types from the generated crate so callers don't need two imports.
2. Use a single `reqwest::Client` configured with timeouts; pass it into the generated client.
3. Retry layer: `tower::retry` or a custom `tower::Layer` against the inner client.
4. Pagination: identify which v1 endpoints actually paginate (audit may need a follow-up — list endpoints today appear to return full collections without paging tokens; if that's the case, this requirement collapses to "no-op until pagination exists" and should be noted).

### Dependencies

- Hard: [[BROKKR-T-0135]].

### Risk Considerations

- Retry policy interacting with non-idempotent POSTs: be conservative — don't retry POSTs by default unless the endpoint is explicitly safe.
- If the generated client doesn't expose hooks for injecting middleware cleanly, the wrapper may need to wrap each method by hand. Quantify before committing to a style.

## Status Updates

### 2026-05-14 — Completed

**Files added / changed:**

- `crates/brokkr-client/src/wrapper.rs` — new ergonomic module (283 LOC including tests + docs).
- `crates/brokkr-client/src/lib.rs` — `mod wrapper;` + re-exports `BrokkrClient`, `BrokkrClientBuilder`, `BrokkrError`.
- `crates/brokkr-client/Cargo.toml` — added `tokio` (default-features off, `time`) as a runtime dep for `tokio::time::sleep` in the retry loop. Dev-deps gained `test-util` + `time` features for `start_paused = true` tests.

**Design:**

- **`BrokkrClient::builder(base_url)`** returns a `BrokkrClientBuilder`. Setters: `token(pak)`, `request_timeout(d)`, `connect_timeout(d)`, `max_retries(n)`, `initial_backoff(d)`. Defaults: 30s request / 10s connect / 3 retries / 200ms initial.
- **Auth via `default_headers`** on the underlying `reqwest::Client`. The configured client is passed to the generated `Client::new_with_client(base_url, reqwest)`. Header is set once; no per-request injection logic needed. Invalid header values rejected at `build()` time with `BrokkrError::InvalidRequest`.
- **`BrokkrError`** is a thin enum over `progenitor_client::Error<ErrorResponse>`. `ResponseValue<ErrorResponse>` is destructured into `Api(ErrorResponse, StatusCode)` so callers can match on `.code()` without re-parsing JSON. `Transport`, `UnexpectedResponse`, and `InvalidRequest` cover the rest. Implements `Display` + `std::error::Error`.
- **Retry helper** uses Tokio's `sleep`; the test suite runs `start_paused = true` so backoff is virtual time (tests complete in milliseconds despite simulated exponential delays). Backoff doubles per attempt with a 10s ceiling. The first attempt always runs; `max_retries` controls only the *re*-attempts. Non-retryable errors short-circuit on the first failure.
- **Access pattern:** `client.api().list_agents().send().await` for direct access; `client.retry(|c| async { c.list_agents().send().await.map_err(BrokkrError::from).map(|r| r.into_inner()) }).await` for wrapped retries. The closure-based design keeps the wrapper free of per-operation glue while letting callers decide which ops are safe to retry (POSTs in particular need to be opted in deliberately).
- **Pagination** is not implemented (audit-confirmed: no paginated endpoints in v1). Module docs flag the placeholder for future `Stream` adapters.

**Verification:**

- `cargo build --workspace` — clean.
- `cargo test -p brokkr-client` — 10/10 pass (7 wrapper + 3 surface). Includes property-style retryable-status table test, and two virtual-time retry behavior tests.
- `cargo clippy -p brokkr-client --all-targets -- -D warnings` — clean.
- `angreal openapi check` — clean (no spec drift).

**Decisions / tradeoffs:**

- **Opt-in retry over auto-wrap.** Auto-retrying every operation transparently is tempting but unsafe for non-idempotent POSTs (`create_agent`, `create_work_order`, `complete_work_order`). The explicit `client.retry(|c| ...)` makes opt-in obvious at the call site.
- **No `tower::retry` / `reqwest-middleware`** in the prototype. Both are viable, but they pull in more deps and abstraction than a 283-LOC wrapper warrants. The closure-based retry is idiomatic and easy to swap to a middleware stack later if needed.
- **Single `tokio` runtime feature (`time`).** Avoids forcing consumers onto a particular runtime flavour. The retry helper is `async fn`; the caller's runtime drives it.
- **No `rustls-tls` opt-in yet.** Default reqwest TLS suffices for the prototype; T-D1 can flip this when the agent migrates and we want a deterministic crypto stack across platforms.

**Carry-overs:**

- **T-C2 (Python wrapper)** mirrors this design: builder-with-token, retry helper, typed error wrapping. The Python wrapper additionally needs the `Generator` name-collision workaround flagged in T-B3.
- **T-C3 (regen drift CI)** — Rust half is already covered by `cargo build` in CI (proc macro regenerates on every build). Python half is a real diff check.
- **T-D1 (brokkr-agent migration)** is now fully unblocked and should use `BrokkrClient::builder(...).token(...).build()?` plus targeted `retry(|c| ...)` wrapping for safe-to-retry calls. The integration test ("smoke test against a real broker") naturally falls out of that migration.