---
id: rust-sdk-cli-cannot-reach-an-https
level: task
title: "Rust SDK/CLI cannot reach an https broker — reqwest built with no TLS backend"
short_code: "BROKKR-T-0220"
created_at: 2026-06-11T12:16:45.335757+00:00
updated_at: 2026-06-11T16:25:06.362055+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# Rust SDK/CLI cannot reach an https broker — reqwest built with no TLS backend

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

`crates/brokkr-client/Cargo.toml:25` declares `reqwest = { version = "0.13", default-features = false, features = ["json", "stream"] }` — no TLS feature. Verified with `cargo tree -p brokkr-cli -e features`: the only reqwest features in the whole tree are `json`/`query`/`stream`; `rustls-tls`/`native-tls`/`default-tls`/`__tls` are all absent, and the wrapper builds a plain `reqwest::Client::builder().build()` (wrapper.rs:218) with no TLS connector. Result: the Rust SDK (`BrokkrClient`) and the `brokkr` CLI can only talk HTTP — an `https://` broker URL fails at runtime with a transport error. Every doc example uses `https://broker.example.com/api/v1` (how-to/sdks/rust.md, how-to/cli-apply.md, reference/cli.md), so the documented usage does not work. Shipped in 0.6.0.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: anyone using the Rust SDK or `brokkr` CLI against a TLS-terminated broker (the documented default). HTTP-only/in-cluster users are unaffected, which is likely why it slipped through (contract tests run against `http://localhost:3000`).
- **Reproduction**: `brokkr apply -f ./manifests --stack x --broker-url https://<broker> --pak <pak>` → transport error; same for any `BrokkrClient` call with an `https://` base URL.
- **Expected vs Actual**: Expected TLS handshake + request; actual: no TLS backend compiled, request fails.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `brokkr-client`'s reqwest enables a TLS backend — use **`rustls-tls`** (pure Rust; no `openssl-sys`), NOT `native-tls`/`default-tls`.
- [ ] An `https://` request from `BrokkrClient` and from `brokkr apply` succeeds against a TLS broker (add coverage: a contract/integration case hitting an https endpoint, or at minimum a test asserting the client builds an https-capable connector).
- [ ] `cargo tree -p brokkr-cli` shows `rustls`/`ring` (or `aws-lc`) and still **no `openssl-sys`** — the T-0199 macOS cross-compile must stay clean (this is why rustls, not native-tls).
- [ ] No lockstep/version churn beyond the dep change; SDK how-to unchanged (examples already use https and will now work).

## Implementation Notes

### Technical Approach
Add `"rustls-tls"` to the reqwest features in `crates/brokkr-client/Cargo.toml`. Confirm progenitor-client (which also depends on reqwest) doesn't separately need a TLS feature — features unify, so enabling it on the brokkr-client dep should suffice for the shared reqwest. Consider whether to also pull system root certs (`rustls-tls-native-roots`) vs webpki roots; webpki-roots is simpler and avoids a platform dep, native-roots respects corporate CAs — pick and document.

### Dependencies
Interacts with [[BROKKR-T-0199]]: the macОS x86_64 leg cross-compiles cleanly *because* the tree has no `openssl-sys` today. The fix MUST keep that true (rustls), or the release CLI build breaks again.

### Risk Considerations
Low — additive feature. Verify the contract suites (http://localhost) still pass and binary size is acceptable.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0025-sdk-parity). Root cause: brokkr-client's reqwest had `default-features = false` + only json/stream — NO TLS backend, so any `https://` broker URL failed at runtime (the Rust SDK and `brokkr` CLI could only do http). Fix: added the `rustls` feature. In reqwest 0.13 the feature is named `rustls` (not `rustls-tls`); it uses aws-lc-rs + the OS trust store via rustls-platform-verifier — openssl-free, so `cargo tree -p brokkr-cli` shows 0 openssl-sys (the T-0199 cross-compile stays openssl-clean; the task AC explicitly allows aws-lc). Had to bump rustls 0.23.20→0.23.40 (`cargo update -p rustls --precise`) to unify with kube's `^0.23` since rustls-platform-verifier needs ≥0.23.27 — agent + broker still build clean (kube 0.95 unaffected). Verified: workspace builds (cli/agent/broker), brokkr-client 14 unit tests pass, clippy clean, 0 openssl-sys in cli tree. The fix is the TLS backend itself (reqwest+rustls does https; without it, https fails) — a live TLS-broker integration fixture is out of scope for the existing http://localhost contract harness. WATCH-POINT: aws-lc-sys is a C lib; its cross-compile for the x86_64-apple-darwin CLI leg (T-0199, macos-14 runner) is supported but only exercised at the next release tag — verify that release build.
