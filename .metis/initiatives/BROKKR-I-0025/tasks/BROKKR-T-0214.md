---
id: sdk-parity-polish-python-telemetry
level: task
title: "SDK parity polish: Python telemetry helpers, error wrapping, exports"
short_code: "BROKKR-T-0214"
created_at: 2026-06-11T11:02:08.223742+00:00
updated_at: 2026-06-11T11:02:08.223742+00:00
parent: sdk-parity-retry-validation-and
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0025
---

# SDK parity polish: Python telemetry helpers, error wrapping, exports

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

Close the remaining cross-SDK asymmetries from the parity sweep (low/medium severity, batched).

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
