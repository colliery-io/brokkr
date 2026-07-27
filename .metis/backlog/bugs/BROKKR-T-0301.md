---
id: console-diagnostics-are-write-only
level: task
title: "Console diagnostics are write-only: the request id is discarded and no view reads GET /diagnostics/:id"
short_code: "BROKKR-T-0301"
created_at: 2026-07-27T19:10:56.285576+00:00
updated_at: 2026-07-27T19:10:56.285576+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Console diagnostics are write-only: the request id is discarded and no view reads GET /diagnostics/:id

## Objective

Give the console somewhere to read a diagnostic result. Even once the route mismatch of BROKKR-T-0275 is fixed, the feature is a dead end: `api::post` discards the response body (`crates/brokkr-web/src/api.rs:170-189`), so the created request id is thrown away, and no console view calls `GET /diagnostics/:id`. The operator gets a toast and can never see what was collected.

The broker already returns the created `DiagnosticRequest` at 201, and `GET /diagnostics/:id` is a plain read that the read-only UI PAK can perform — so this needs no new broker surface and no allowlist change.

Found 2026-07-27 while scoping BROKKR-T-0275; filed separately so that ticket can close on "the button works" without implying results are visible.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (incomplete feature)

### Priority
- [x] P2 - Medium (depends on BROKKR-T-0275 landing first to be meaningful)

## Acceptance Criteria

- [ ] `api::post` (or a variant) returns the deserialized response body so callers can use it.
- [ ] The diagnostic request id is retained and the console polls `GET /diagnostics/:id` until terminal.
- [ ] Results are rendered somewhere sensible (pod statuses, events, log tails), with an honest empty-state — note BROKKR-T-0299 means pod data is commonly empty until that is fixed, and BROKKR-T-0291 means collection errors currently arrive as `completed` with an `error` entry inside `events`.
- [ ] Playwright scene covering request → result display (mirrors the BROKKR-T-0274 pattern).

## Status Updates

*To be added during implementation*
