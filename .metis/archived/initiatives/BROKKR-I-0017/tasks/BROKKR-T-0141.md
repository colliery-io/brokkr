---
id: d2-sdk-usage-documentation
level: task
title: "D2: SDK usage documentation"
short_code: "BROKKR-T-0141"
created_at: 2026-05-14T18:26:29.324966+00:00
updated_at: 2026-05-15T18:46:17.467168+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0140]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# D2: SDK usage documentation

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Write the docs that turn the SDKs into something an external integrator (or new team member) can actually pick up: getting started, auth, error handling, pagination, regeneration workflow. Sequenced last because the agent migration (D1) is the best forcing function for surfacing what's hard to learn.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Getting-started page for Rust SDK: install, construct a client, call one endpoint, handle one error. ≤ 1 page.
- [x] Getting-started page for Python SDK: same shape.
- [x] Auth section: explains the single-credential model, how PAK prefixes route internally, where to obtain PAKs.
- [x] Error-handling section: documents `ErrorResponse.code` values that consumers should expect to match on, with a stable-codes table.
- [x] Regeneration workflow: how to update the SDK after a broker API change. Names the angreal task(s) and the CI drift check.
- [x] Pages land in the existing docs site (under `docs/`) and are linked from a navigable index.
- [x] Worked-example snippet (heartbeat + fetch target state, mirroring what `brokkr-agent` does) included for the Rust SDK.

## Implementation Notes

### Technical Approach

1. Reuse the brokkr-agent migration commits from D1 as raw material for the worked example.
2. Stable error codes list: dump from the `ApiError` enum landed in T-A1.
3. Keep docs short. Anything that wants to be longer than one screen is probably a doc smell — push it into the rustdoc / Python docstrings instead.

### Dependencies

- Hard: [[BROKKR-T-0140]] (need the migration in hand for the worked example).
- Soft: docs build pipeline (`angreal docs build`) — confirmed working today.

### Risk Considerations

- Doc rot. The drift-check CI doesn't catch doc rot. Cross-reference the stable error code table against the enum in CI if cheap; otherwise live with it.

## Status Updates

- 2026-05-15: SDK docs landed under `docs/src/how-to/sdks/`:
  - `README.md` — overview, auth (PAK prefix routing table), error handling, pagination, links to lang pages.
  - `rust.md` — install/construct/call/error/retry + worked example (heartbeat + `get_target_state`) mirroring `crates/brokkr-agent/src/broker.rs`.
  - `python.md` — same shape; uses `brokkr.BrokkrClient` and the generated async surface.
  - `errors.md` — stable error-code table cross-referenced against `ApiError` usages in `crates/brokkr-broker/src/api/v1/`.
  - `regeneration.md` — names `angreal openapi export/gen-python/gen-typescript`, CI drift checks, and the cargo-build path for Rust.
- Linked from `docs/src/SUMMARY.md`. `angreal docs build` clean (only pre-existing unrelated API-docs HTML-tag warnings).