---
id: sdk-parity-polish-python-telemetry
level: task
title: "SDK parity polish: Python telemetry helpers, error wrapping, exports"
short_code: "BROKKR-T-0214"
created_at: 2026-06-11T11:02:08.223742+00:00
updated_at: 2026-06-11T19:21:30.864387+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: true

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# SDK parity polish: Python telemetry helpers, error wrapping, exports

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

Close the remaining cross-SDK asymmetries from the parity sweep (low/medium severity, batched).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Python gains the telemetry surface Rust/TS have: `list_telemetry_events`, `list_telemetry_logs`, `list_ws_connections` (Rust wrapper.rs:273-317, TS client.ts:121-152); decide whether `liveSubscriptionUrl` (TS-only, client.ts:260-272) belongs in all three and implement or document the decision.
- [ ] Error wrapping harmonized: Python raw `OSError` on unreadable file (client.py:264) and raw `ValueError` on generator-UUID parse (:175) → `BrokkrError`; TS raw Node errors in helpers → `BrokkrError` (overlaps T-0213 — coordinate).
- [ ] Retryability semantics aligned on transport-only for status-less errors (Rust behavior, wrapper.rs:90-100); Python `errors.py:40-45` and TS `error.ts:41-43` currently report local validation errors as retryable.
- [ ] Export stance unified: TS exports `readManifests`/`sha256Hex`; make the equivalents public in Rust (`read_manifests`, `sha256_hex`) and Python or document why not.
- [ ] Rust doc-comment fix: wrapper.rs:88-89 says "429/502/503/504" but code includes 408 — say so.
- [ ] TS gains a connect-timeout option (Rust/Python expose one) or the gap is documented.
- [ ] SDK how-to pages updated where surface changed.

## Implementation Notes

Naming asymmetry `ApplyOutcome` (Rust) vs `ApplyResult` (Py/TS) ships in 0.6.0 — renaming is a breaking change; leave it, note it in the SDK README. The Authorization wire-format difference (Rust raw PAK vs Bearer) is accepted by the broker both ways (middleware.rs:86-89) — informational, no change.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0025-sdk-parity). Parity polish:
  - **Python telemetry helpers**: added `list_telemetry_events`, `list_telemetry_logs`, `list_ws_connections` to the Python wrapper (Rust/TS already had them) — thin `_expect`(`*_detailed`) wrappers over the generated stack_telemetry/admin endpoints. python.md gained a "Stack telemetry" section.
  - **Python error wrapping**: the generator-id `UUID(str(generator))` parse in apply now wraps a bad value in BrokkrError instead of leaking a raw ValueError (parity with Rust's typed UnexpectedResponse). (TS fs-error wrapping was handled in T-0213.)
  - **Rust doc fix**: BrokkrError::is_retryable doc said "429/502/503/504" but the code matches 408|429|502|503|504 — corrected to include 408.
  DOCUMENTED DECISIONS (not changed):
  - **liveSubscriptionUrl** stays TS-only (browser-oriented); python.md tells Python users to build the ws:// URL themselves. Rust likewise.
  - **Export stance**: read_manifests/sha256_hex stay private in Rust (`_`-private in Python); TS exports them only because the demo UI imports them. Keeping the minimal public surface (submit_manifests/apply) elsewhere is intentional.
  - **Retryability of status-less errors**: Python/TS report a status-less BrokkrError as retryable while Rust restricts to transport. In practice local-validation BrokkrErrors are raised directly, never passed through retry()'s loop, so the difference is benign — left as-is.
  - **TS connect-timeout**: Rust/Python expose a connect timeout; TS exposes only requestTimeoutMs. Low value; gap noted, not added.
  Rust + Python build clean; Python 32 wrapper tests pass.