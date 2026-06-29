---
id: map-brokkr-monitor-design-to
level: task
title: "map Brokkr Monitor design to aurora-leptos + build app-local gap components (slide-over, sparkline, segmented health bars)"
short_code: "BROKKR-T-0255"
created_at: 2026-06-28T01:44:26.547382+00:00
updated_at: 2026-06-29T00:32:35.205108+00:00
parent: brokkr-operator-console
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# design→aurora-leptos mapping + app-local gap components

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

`aurora-leptos` supplies the primitives + widgets, so we **do not reimplement** them.
This task (a) produces the **mapping** from each Brokkr Monitor design element to its
`aurora-leptos` component (per `PATTERNS.md`), and (b) builds the **few app-local
components the pack lacks** — the shared building blocks the views then compose.

### Type
- [x] Feature — design mapping + app-local component gaps

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A short mapping doc (in `crates/brokkr-web/design/` or the crate README): handoff element
      → pack component, e.g. card→`Panel`, title→`PageHeader`, status tag→`StatusBadge`/`Pill`/
      `HealthPill`, dot→`Dot`, filter→`Chip`, async states→`Loading`/`Empty`/`ErrorState`,
      KPI counts→`StateCounts`, progress/freshness→`Meter`, build state→`BuildStatusBadge`,
      tables→`Table`, shell→`AppShell`, overlay→`Modal`. Colors via `status_color` / `token::*`.
- [ ] **App-local components** built on the pack's tokens/primitives (the handoff pieces not in
      the pack):
  - [ ] **Agent slide-over** — right-anchored 430px panel + scrim (the pack's `Modal` is
        centered; build a slide-over variant or thin wrapper). Used by [[BROKKR-T-0258]].
  - [ ] **SVG area sparkline** — broker throughput (44-point ring buffer, ice line + tinted
        fill). Used by [[BROKKR-T-0257]]/[[BROKKR-T-0262]].
  - [ ] **Segmented health bar** — fleet-by-cluster (healthy/degraded/failing/offline widths).
        Used by [[BROKKR-T-0257]].
- [ ] App-local components take meaning as **data** (labels/colors/tips), matching the pack's
      convention; no hard-coded hex.
- [ ] (Toasts + the Live/Paused engine are [[BROKKR-T-0256]], not here.)

## Dependencies

- Depends on [[BROKKR-T-0252]] (crate + `aurora-leptos` wired). **Blocks** the view slices
  that use the gap components ([[BROKKR-T-0257]], [[BROKKR-T-0258]], [[BROKKR-T-0262]]); other
  views can proceed on the pack alone.

## Implementation Notes

- The Cloacina-domain widgets (RunCircles, ReactorReadiness, AccumulatorTable, GraphHealth)
  are NOT used. Confirm the pack's `Graph` is unneeded for v1 (Brokkr has no DAG view).
- Reference: `aurora-leptos/PATTERNS.md` + `INVENTORY.md`; the handoff spec for the gap pieces.

## Status Updates

*To be added during implementation*

**2026-06-28 — implemented + pixel-verified.** `src/components.rs`: `Sparkline` (SVG area
via inner_html) + `SegmentedHealthBar` (proportional ok/gold/bad/offline) — both verified in
the Overview; `SlideOver` (right-anchored panel + scrim) — verified in the Fleet agent detail.
The design→aurora-leptos mapping is realized across the views (Panel/PageHeader/StatusBadge/
Pill/Dot/Loading/Empty/ErrorState/SegmentedControl from the pack; these three app-local).
**2026-06-28 — `SlideOver` removed.** Detail views use the pack's centered `Modal` instead, so
the app-local gap components are now just `Sparkline` + `SegmentedHealthBar`.