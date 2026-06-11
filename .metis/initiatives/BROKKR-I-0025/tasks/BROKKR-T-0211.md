---
id: python-sdk-real-retry-on-http
level: task
title: "Python SDK: real retry on HTTP status; stop fabricating BrokkrError.status"
short_code: "BROKKR-T-0211"
created_at: 2026-06-11T11:02:08.076118+00:00
updated_at: 2026-06-11T11:02:08.076118+00:00
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

# Python SDK: real retry on HTTP status; stop fabricating BrokkrError.status

## Parent Initiative

[[BROKKR-I-0025]]

## Objective

Two related defects in `sdks/python/brokkr/brokkr/client.py`. (1) The generated client defaults `raise_on_unexpected_status=False` and 408/429/502/503/504 are not documented response codes in the spec, so e.g. a 503 parses to `None` → `retry()` (client.py:100-111) treats it as **success and returns `None` to the caller**; the documented retry-on-status behavior is dead code (only `httpx.HTTPError` transport failures retry). (2) `_expect` hardcodes `status=400` (client.py:239) and the retry union path hardcodes `status=500` (client.py:107) — a 404 surfaces as `BrokkrError(status=400)`, corrupting both user pattern-matching and `is_retryable()`. Rust (wrapper.rs:90-100,479-492) and TS (error.ts) get both right.

## Acceptance Criteria

- [ ] `retry()` observes the real HTTP status (use `*_detailed` variants or `raise_on_unexpected_status=True` + classification) and retries exactly {408, 429, 502, 503, 504} + transport errors, matching Rust/TS.
- [ ] `retry()` never returns `None` for an HTTP error — it raises `BrokkrError` with the real status.
- [ ] `BrokkrError.status` is the wire status everywhere (no hardcoded 400/500).
- [ ] Unit tests with respx covering: 503-then-200 (retries, succeeds), 404 (raises immediately, status=404), transport error (retries).
- [ ] Docs (how-to/sdks/python.md retry section) still accurate after the change.

## Status Updates

*To be added during implementation*
