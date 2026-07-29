---
id: console-full-ui-pass-live-toggle-inert
level: task
title: "Console UI pass: the Live/Paused toggle is a visible control that does nothing"
short_code: "BROKKR-T-0319"
created_at: 2026-07-29T07:00:00+00:00
updated_at: 2026-07-29T07:00:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Console UI pass: the Live/Paused toggle is a visible control that does nothing

## Objective

The full pass over the Operator Console that Dylan called for before publication (2026-07-29), separate from the generator-minting panel in BROKKR-T-0318 so the sweep is not blocked behind that work.

Filed as a bug rather than a chore because the sweep already has one confirmed, user-facing defect to fix.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (a shipped control silently does nothing; no data loss, but it misrepresents the console's behaviour to an operator)

## 2026-07-29 — grounded against the crate, not assumed

`crates/brokkr-web/src/` is ~2,600 lines across 7 views (`overview`, `fleet`, `deployments`, `work_orders`, `webhooks`, `health`, `telemetry`) plus `app.rs`, `api.rs`, `components.rs`, `models.rs`.

### 1. The Live/Paused toggle is inert — confirmed

`app.rs:81-82` creates `let live = RwSignal::new(String::from("Live"))` with the comment *"drives the live engine in a later slice"*. It is threaded into `Main` (`:100`), declared on the component (`:248`), and bound to a `Select` with options `["Live", "Paused"]` (`:263-264`).

**Nothing ever reads it.** No view consumes a live/pause signal — the only matches in `views/` are unrelated (a `brokkr_fleet_live_subscribers` metric label, and a local `live` boolean for agent status dots). The "later slice" never landed.

So an operator can set the console to **Paused** and every view keeps polling and updating. This is the same class of defect as BROKKR-T-0308's inert chart values — a control that is accepted, rendered, and does nothing — except this one is in the operator's face rather than in a values file.

Two honest ways to close it, and the choice should be deliberate:
- **Implement it**: gate each view's refresh on the signal. Real work, since every view owns its own resource/polling.
- **Remove it**: delete the control until the live engine exists. Cheaper, and strictly better than shipping a lie.

### 2. `app.rs`'s module doc is stale

`app.rs:3` still says *"Views are placeholders; live data lands in the later slices."* That was true at slice 1a (BROKKR-I-0031); it is not now — every view fetches real data (`Resource::new`/fetch counts: overview 8, fleet 10, health 6, deployments 5, work_orders 5, telemetry 4, webhooks 3). BROKKR-I-0032 landed that. The comment misleads the next reader about how finished the crate is.

### 3. The pass itself

With those two known items as the starting point, the sweep should cover, per view: does every rendered control do what it appears to do; does every panel handle empty/error/loading states; does anything claim a capability the read-only credential cannot perform (see BROKKR-T-0318 for why the console is read-only); and does the tenant scope selector behave consistently across views.

Worth checking specifically because it has bitten twice already in this codebase: **any control whose handler is a no-op**. The Live/Paused toggle was found by grepping for a signal with no consumer; the same technique should be run across the crate rather than relying on visual inspection.

### Build note

`brokkr-web` is outside the workspace. Build and lint it with `cd crates/brokkr-web && cargo clippy --target wasm32-unknown-unknown` / `trunk build` — `cargo -p brokkr-web` from the root will not work.

## Acceptance Criteria

- [ ] The Live/Paused toggle either works or is removed — it does not ship as a control that does nothing.
- [ ] `app.rs`'s module doc no longer describes the views as placeholders.
- [ ] Every interactive control in the crate is traced to a consumer; any other no-op control is fixed or removed, and the sweep method is recorded so it can be repeated.
- [ ] Each view handles empty, loading, and error states without a blank panel.
- [ ] No view offers an action the read-only UI PAK cannot perform (excepting whatever BROKKR-T-0318 adds behind an operator-supplied admin PAK).
- [ ] `cargo clippy --target wasm32-unknown-unknown` and `trunk build` both pass.

## Status Updates

*To be added during implementation*
