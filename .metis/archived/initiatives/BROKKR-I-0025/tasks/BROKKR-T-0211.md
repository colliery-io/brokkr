---
id: python-sdk-real-retry-on-http
level: task
title: "Python SDK: real retry on HTTP status; stop fabricating BrokkrError.status"
short_code: "BROKKR-T-0211"
created_at: 2026-06-11T11:02:08.076118+00:00
updated_at: 2026-06-11T19:21:30.673490+00:00
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

# Python SDK: real retry on HTTP status; stop fabricating BrokkrError.status

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

Two related defects in `sdks/python/brokkr/brokkr/client.py`. (1) The generated client defaults `raise_on_unexpected_status=False` and 408/429/502/503/504 are not documented response codes in the spec, so e.g. a 503 parses to `None` → `retry()` (client.py:100-111) treats it as **success and returns `None` to the caller**; the documented retry-on-status behavior is dead code (only `httpx.HTTPError` transport failures retry). (2) `_expect` hardcodes `status=400` (client.py:239) and the retry union path hardcodes `status=500` (client.py:107) — a 404 surfaces as `BrokkrError(status=400)`, corrupting both user pattern-matching and `is_retryable()`. Rust (wrapper.rs:90-100,479-492) and TS (error.ts) get both right.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `retry()` observes the real HTTP status (use `*_detailed` variants or `raise_on_unexpected_status=True` + classification) and retries exactly {408, 429, 502, 503, 504} + transport errors, matching Rust/TS.
- [ ] `retry()` never returns `None` for an HTTP error — it raises `BrokkrError` with the real status.
- [ ] `BrokkrError.status` is the wire status everywhere (no hardcoded 400/500).
- [ ] Unit tests with respx covering: 503-then-200 (retries, succeeds), 404 (raises immediately, status=404), transport error (retries).
- [ ] Docs (how-to/sdks/python.md retry section) still accurate after the change.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0025-sdk-parity). Root of both bugs: the wrapper unwrapped non-detailed `.asyncio()` results, which drop the HTTP status. Fixed by routing through `*_detailed` Responses (carry `.status_code`), matching the add-label loop that already did this:
  - **retry()**: `op` must now return a `*_detailed` Response. retry reads the real `status_code`: <400 returns `.parsed`; otherwise raises BrokkrError with the REAL status (ErrorResponse body → from_response, else a status-only error). Fixes (a) the `None`-returned-as-success bug for undocumented statuses (e.g. 503), and (b) retry classification now uses the true status. The old code returned None on a 503 and treated any ErrorResponse as status=500.
  - **_expect()** (apply's internal unwrap): same change — reads status from the detailed Response, no more hardcoded `status=400`. The 6 apply/submit_manifests call sites switched from `.asyncio()` to `.asyncio_detailed()`.
  Tests (test_wrapper.py): success returns parsed; **503-then-200 retries and succeeds** (was the None bug); **404 raises with status=404** (was fabricated); transport retry + backoff unchanged — 32 pass. Docs: python.md retry examples switched to `.asyncio_detailed` with a note on why. The apply/contract flow runs on CI's python contract suite.