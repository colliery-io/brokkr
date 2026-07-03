---
id: console-tenant-scope-selector
level: task
title: "Console: tenant scope selector wired through data views"
short_code: "BROKKR-T-0273"
created_at: 2026-07-03T00:09:08.065195+00:00
updated_at: 2026-07-03T03:47:37.059675+00:00
parent: BROKKR-I-0032
blocked_by: [BROKKR-T-0269, BROKKR-T-0270, BROKKR-T-0272]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0032
---

# Console: tenant scope selector wired through data views

## Parent Initiative

[[BROKKR-I-0032]]

## Objective

Add a tenant scope selector to the console (sidebar, below the nav groups): "All" plus one entry per tenant from `GET /api/v1/paks`. Selecting a scope filters Fleet, Deployments (stacks), and Telemetry (agent-events) reactively via `?pak_id=`, persists across view navigation and page refresh.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Scope selector renders in the sidebar with "All" default; options fetched from `/api/v1/paks` (screenshot: `scope-selector.png`)
- [x] Selection is a reactive signal provided via Leptos context; Fleet, Deployments, and Telemetry re-fetch with `?pak_id=` when it changes (debug probe: scoped fetch fires immediately on select; screenshot `fleet-scoped.png` shows the narrowed fleet + recomputed KPIs)
- [x] Selection persists in `localStorage["brokkr_scope"]` and is restored on boot (probe confirmed the write); a stored id no longer present in `/paks` falls back to "All"
- [x] Styled `select` matching `var(--inset)` / `var(--border-control)` (recorded deviation: no SegmentedControl variant — sidebar width + duplicate-name safety)
- [x] Empty tenant list → selector hidden (all legacy shots scenes render without the selector)

## Implementation Notes

### Technical Approach
- `crates/brokkr-web/src/app.rs` — `Sidebar` (L74); provide `RwSignal<Option<String>>` scope context in `App` (L52).
- `crates/brokkr-web/src/api.rs` — add `paks()` fetch + optional `pak_id` param on `fleet()`, `stacks()`, `agent_events()` (append `?pak_id=` when `Some`).
- Views: `views/fleet.rs`, `views/deployments.rs`, `views/telemetry.rs` — resources keyed on the scope signal so LocalResource/create_resource re-fires on change.
- `crates/brokkr-web/src/models.rs` — `PakSummary { id, name }` DTO.

### Dependencies
BROKKR-T-0269 (`/paks`), BROKKR-T-0270 (`?pak_id=`), BROKKR-T-0272 (token boot).

## Status Updates

**2026-07-02 — implemented, verification pending**
- `app.rs`: `ScopeSignal` (`RwSignal<Option<String>>`) provided at app root; `use_scope()` accessor; restored from `localStorage["brokkr_scope"]`, persisted via Effect; `ScopeSelector` component in sidebar below nav — "All" + one option per `/paks` entry, hidden when list empty/unavailable; stale stored scope falls back to All.
- DESIGN DEVIATION (recorded): styled `<select>` for all tenant counts instead of `SegmentedControl` for ≤3 — the 220px sidebar can't fit segments for arbitrary tenant names, and `value=id` keeps duplicate names unambiguous.
- `models.rs`: `PakSummary`. `api.rs`: `paks()`, `scoped()` helper; `fleet`/`stacks`/`agent_events` take `Option<String>` scope.
- Views (fleet, overview, deployments, telemetry): LocalResource fetchers read the scope signal → auto-refetch on change.
- KNOWN GAP (pre-existing, unchanged): Fleet modal "Run diagnostic" POSTs `/api/v1/diagnostics`, which has never existed on the broker (real route `POST /deployment-objects/:id/diagnostics` needs a deployment-object id the fleet view doesn't have). Button 404s before and after this initiative. Recommend a follow-up backlog item: either a broker `POST /agents/:id/diagnostics` convenience route or fetching the agent's objects in the modal.

**2026-07-03 — VERIFIED (with one bug found & fixed)**
- BUG: gloo-net appends a trailing `&` to URLs that already carry a query string, so the hand-built `"/fleet?pak_id=<id>"` reached the network as `/fleet?pak_id=<id>&`. Harmless against the real broker (serde_urlencoded ignores the empty pair) but it dodged the Playwright query-keyed mock — caught because the `fleet-scoped` screenshot showed an unfiltered fleet. FIX: query params now attach via gloo's `.query()` builder API (`api::get_scoped`); the harness also normalizes trailing separators.
- Debug probe (Playwright request log): selecting a tenant fires `GET /fleet?pak_id=<id>` immediately (LocalResource reactivity confirmed) and on each 5s poll; `localStorage["brokkr_scope"]` written; staging tenant's agent disappears; KPIs recompute (3→2 agents).
- Screenshots regenerated: `scope-selector.png` (selector, All), `fleet-scoped.png` (team-payments narrowed) verified visually.