---
id: console-diagnostics-are-write-only
level: task
title: "Console diagnostics are write-only: the request id is discarded and no view reads GET /diagnostics/:id"
short_code: "BROKKR-T-0301"
created_at: 2026-07-27T19:10:56.285576+00:00
updated_at: 2026-07-28T15:12:12.089480+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `api::post` (or a variant) returns the deserialized response body so callers can use it.
- [ ] The diagnostic request id is retained and the console polls `GET /diagnostics/:id` until terminal.
- [ ] Results are rendered somewhere sensible (pod statuses, events, log tails), with an honest empty-state — note BROKKR-T-0299 means pod data is commonly empty until that is fixed, and BROKKR-T-0291 means collection errors currently arrive as `completed` with an `error` entry inside `events`.
- [ ] Playwright scene covering request → result display (mirrors the BROKKR-T-0274 pattern).

## Status Updates

**2026-07-28 — IMPLEMENTED** on branch `docs/tenancy-review-2026-07`. Note the implementing agent **stalled and was reported failed** while trying to *execute* the Playwright scene (trunk's file watcher rebuilds when shots are written into the crate dir, causing a reload loop). The code work was already complete at that point; I verified it independently rather than re-running the task.

Verified by inspection and build:
- `api::diagnostic(id)` fetches `GET /api/v1/diagnostics/{id}`; the create call now returns the 201 body so the request id survives.
- `models.rs` gained `DiagnosticResponse`/`DiagnosticRequestDto`/`DiagnosticResultDto` plus a parsed `DiagnosticData` view, handling the JSON-encoded-string fields (`pod_statuses`, `events`, `log_tails`).
- Polling is **bounded** (`POLL_MAX` attempts at `POLL_EVERY_MS`), settles on terminal status *or* on a read error (an error means polling cannot make progress either), and resets on every agent-row click so state cannot leak between agents.
- Collection failure is distinguished from empty success: `collection_errors()` detects the `completed` + `error`-in-`events` shape and the UI states plainly that it is proof collection failed, while an empty `pod_statuses` renders as a note ("expected when the deployment object applies no workloads"), not an error.
- BROKKR-T-0275's "Results are not shown in the console yet" caption is removed, as required.
- `trunk build` succeeds; `cargo clippy --all-targets --target wasm32-unknown-unknown` clean.

**Open:** the Playwright scene was written into `web-e2e/shots.mjs` (+82 lines) but **never executed**. It needs a run against a live broker before it can be trusted; the trunk-watcher interaction that stalled the agent is itself worth knowing about for anyone running that harness from inside the crate directory.