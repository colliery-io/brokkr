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

## 2026-07-29 — the screenshot harness cannot be trusted as a baseline yet

Found while adding tenant scenes for BROKKR-T-0318, and it lands squarely in this ticket's scope.

`web-e2e/shots.mjs` ended every interaction in `.catch(() => {})`, so a click that did nothing still printed `shot: <name>` and wrote a plausible-looking PNG **of the default Overview view**. Most `nav:` scenes were affected. Root cause was not the clicks: `trunk serve` watched `web-e2e/`, which contains the harness's own PNG output, so each screenshot triggered a rebuild whose live-reload reset the route mid-scene (310 rebuilds in one run).

T-0318 fixed the harness itself — `Trunk.toml` watch exclusion, a `navigateTo()` that verifies the page header and retries, method-aware mocks, and a `fill` primitive. **It did not re-review the pre-existing scenes' screenshots**, which is this ticket's job:

- [ ] Re-run the harness and actually *look* at all 23 scenes. Any that were previously capturing Overview have never been visually reviewed at all.
- [ ] Only then consider committing them as golden-image baselines. A diff built on the current PNGs would enshrine whatever was wrong.
- [ ] Two scenes are worth specific suspicion because their assertions were the weakest: `fleet-scoped` (the only scene that failed even the permissive early check) and the three `fleet-diagnostic*` scenes, which depend on a `then_click` inside a modal — a nested interaction with the same silent-failure shape.

**Also note the harness's limits, so the sweep does not over-trust it.** It renders with fixtures and screenshots; it cannot see a control that renders correctly and *does nothing* — which is exactly this ticket's headline defect (the Live/Paused toggle). Pixel coverage and no-op coverage are different problems: the toggle would look perfect in every golden image. The signal-with-no-consumer grep remains the tool for that class.

## 2026-07-30 — DECISION (Dylan): remove the toggle; a real pause is per-agent and authorized

> *"the live/paused toggle probably shouldn't exist unless it takes a pak (admin or agent) to authorize the change in state on the agent itself. it also shouldn't be in a spot that makes it appear 'global'"*

This reframes the defect and makes it worse than filed. The ticket above treated "Live/Paused" as a client-side refresh control that happens to be inert. Read as an operator would read it in a deployment tool, it says **pause the agents** — and its placement in the global page header says **all of them**.

So there are two problems, and the ambiguity is the more dangerous one:

1. **It does nothing** (established above).
2. **What it appears to do is a privileged, fleet-wide operation** — with no credential, no target, and no confirmation. An operator who believes they have paused deployment, and has not, is worse off than one who never had the control.

### Pausing an agent is real, and already implemented — just not here

Checked before deciding, because "it needs a PAK to change agent state" is only actionable if that state exists:

| Piece | Where |
|---|---|
| Writable state | `PUT /api/v1/agents/{id}` sets `status` (`api/v1/agents.rs:398`), **admin-only** |
| Agent honours it | `brokkr-agent/src/cli/commands.rs:427` skips deployment-object fetches, `:545` skips work orders, when `status != "ACTIVE"` |
| Takes effect live | the agent re-fetches its own record each heartbeat (`commands.rs:409-412`), so a change lands within a poll cycle |

So the control Dylan describes is buildable today with no broker work: per-agent, in the Fleet view's agent modal, authorized by an admin PAK supplied per action — exactly the BROKKR-T-0318 pattern, which now exists to copy.

**But that is a feature, not this bug.** Removing the misleading control is the fix here; building the real one is separate and should not gate the UI sweep.

### Decision

- **Remove** the `Live/Paused` `SegmentedControl` and the `live` signal from `app.rs`. It ships as neither a working refresh toggle nor a working pause.
- **Do not** relocate or relabel it as a stopgap. A working per-agent pause belongs in the agent modal next to the diagnostic action, where it has a target and a credential prompt.
- The clock beside it stays — that one is honest.

### Acceptance Criteria (supersedes the first criterion above)

- [x] The `Live/Paused` control and its `live` signal are gone from `app.rs` and `Main` (BROKKR-T-0322).
- [x] No global header control implies a fleet-wide state change (BROKKR-T-0322).
- [x] A follow-up ticket exists for a per-agent pause — BROKKR-T-0322, built rather than deferred.
