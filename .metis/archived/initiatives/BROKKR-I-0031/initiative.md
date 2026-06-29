---
id: brokkr-operator-console
level: initiative
title: "Brokkr Operator Console"
short_code: "BROKKR-I-0031"
created_at: 2026-06-27T19:20:44.336146+00:00
updated_at: 2026-06-29T15:17:57.696304+00:00
parent: brokkr-environment-aware
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: brokkr-operator-console
---

# Brokkr Operator Console

> **Phase: design (in build).** Decisions are settled in [[BROKKR-A-0010]] and the
> initiative is decomposed (12 tasks). Slice 1a ([[BROKKR-T-0252]]) is underway — the
> `crates/brokkr-web` walking skeleton builds and serves.

## Context

Brokkr has no supported UI. The only prior UI initiative — [[BROKKR-I-0012]] "UI
Search/Filter" — was cancelled and archived (#56) on the stance *"the web UI is a
demo, not warranted."* `examples/ui-slim` remains a **consumer-side demo** (a
reference for someone who *consumes* Brokkr) and stays that way.

This initiative reverses that stance for a **different surface**: a supported
**operator console** for the people who *run* Brokkr — observing and operating the
control plane (fleet, agents, generators, registrations, stacks/deployment objects,
work orders, deployment health, diagnostics, audit). It is explicitly **not** a
manifest- / desired-state-authoring surface; authoring stays in the `brokkr` CLI,
the SDKs, and the generator path.

**Why now.** The control plane recently grew the operational surfaces that make a
console worth building — fleet legibility (`GET /api/v1/fleet`, I-0027), fleet live
push (`/api/v1/fleet/live` WS, I-0028), generator registration ([[BROKKR-I-0030]]),
audit logs, deployment health, and diagnostics. The data already exists; the console
consumes it.

**Design basis.** The colliery-io **Aurora Dark** design system
(`@colliery-io/aurora-dark`), shared with Cloacina's control-plane UIs.
**Implementation in Leptos (Rust/WASM)**, not the React/JS stack `ui-slim` uses.

## Goals & Non-Goals

**Goals**
- A supported, dark-only operator console for observing and operating one Brokkr
  control plane.
- Surface the existing read/ops APIs: fleet, agents, generators, registrations,
  stacks/deployment objects, work orders, deployment health, diagnostics, audit logs.
- Adopt the Aurora Dark **design language** for visual consistency across
  colliery-io control planes.
- Built in **Leptos (Rust)**, reusing the broker's Rust types / `brokkr-client` SDK
  where practical.

**Non-Goals**
- **Not** a manifest / desired-state authoring tool — submitting stacks and
  deployment objects stays in the CLI / SDKs / generators.
- Does **not** replace or productize `examples/ui-slim` (it remains a consumer demo).
- Not a light theme — Aurora Dark is permanently dark.
- Not a multi-tenant portal — scoped to a single broker/operator context (per-tenant
  brokers run their own console).

## Discovery Findings — the design pack (Aurora Dark)

`@colliery-io/aurora-dark` (v0.1.0, public) is a **dark-only** design system: a
**hybrid of Mantine 7 (React) components + a small set of Aurora primitives**, all
driven by **one CSS custom-property token set** (`aurora.css`). Idiom: *"dark, cold,
dense, monospace-for-identifiers."*

- **Tokens (directly portable):** surfaces (`--bg` #0e1116, `--panel`, `--border`),
  text (`--fg`, `--muted`), status accents (`--ice` #7fb2ff primary, `--teal`,
  `--violet`, `--gold`, `--ok` #4bd07f, `--bad` #f06464). Type: **IBM Plex Sans**
  (UI) / **IBM Plex Mono** (IDs, codes, timestamps).
- **Components:** generic primitives (`Panel`, `PageHeader`, `BrandMark`, `Pill`,
  `StatusBadge`, `Dot`, `Chip`, `Loading`/`Empty`/`ErrorState`) plus Cloacina-domain
  widgets (`RunCircles`, `ReactorReadiness`, `AccumulatorTable`, `GraphHealth`,
  `BuildStatusBadge`).
- Ships a `.design-sync/` workflow (per-component `.d.ts` API + `.prompt.md` usage)
  and a DesignSync tooling path.

**Update — Aurora Dark now ships a Leptos crate (supersedes the "reimplement" framing).**
`aurora-leptos` (`colliery-io/aurora-dark/rust/aurora-leptos`) is the productized Leptos
port: a full component + widget set, `token::*` + `status_color()`, the CSS bundle, a
`leptos-gallery` example, and a `PATTERNS.md` usage guide. So this is **"consume a Rust
component library,"** not "reimplement the component layer" — Brokkr depends on the crate
(Resolved Approach #1). The React `@colliery-io/aurora-dark` npm package + its Mantine
components stay unused (Leptos can't run them), and the Cloacina-domain widgets
(RunCircles, ReactorReadiness…) are out of scope — but the generic pack (Panel, PageHeader,
StatusBadge, Pill, Dot, Chip, async states, StateCounts, Meter, Table, Modal, AppShell,
Graph) covers nearly all of the Brokkr Monitor design.

## Resolved Approach (discovery decisions, 2026-06-27)

**1. Design system — consume the `aurora-leptos` crate.** Aurora Dark ships a Leptos
implementation, **`aurora-leptos`** (`colliery-io/aurora-dark` → `rust/aurora-leptos`):
the full component + widget set, `token::*` + `status_color()`, and the CSS bundle.
**Depend on it** as a pinned git dependency (`rev = b32747d…`) + `leptos 0.8`; style via
`<AuroraStyles/>` or the `aurora-css` Trunk `pre_build` hook (per `leptos-gallery`). Do
**not** reimplement primitives or hand-vendor a ds-bundle. Per the pack's `PATTERNS.md`,
the app supplies meaning (state→color/label, branding) as **data**. The few handoff pieces
not in the pack (agent slide-over, SVG sparkline, segmented health bars, toast stack) are
thin **app-local** components on the pack's tokens/primitives.

**2. Read-mostly observability plane.** Observability surface, not a control panel.
The **only write in v1** is the existing **`POST /api/v1/diagnostics`** ("run
diagnostic"). Agent activate/deactivate (in the design handoff) and the earlier
"replay last event" idea are **deferred** — neither has a broker endpoint today.
See [[BROKKR-A-0010]] for the full decision.

**3. Hosting — Leptos served by the broker.** The built Leptos WASM bundle is served
by `brokkr-broker` itself: the axum app keeps `/api/v1` for the API and adds the
static UI + SPA fallback as the outer layer (mirroring `skadi-api`'s `serve.rs` /
`assets.rs`).

## Reference Architecture

Two references, both Leptos→wasm:

- **Consuming `aurora-leptos` → `colliery-io/aurora-dark/rust/leptos-gallery`** is the
  canonical example: a Trunk Leptos app that depends on `aurora-leptos`, emits the CSS via
  the `aurora-css` `pre_build` hook (`<link>`, no flash), and renders every component. Mirror
  its `Cargo.toml` (git dep + `leptos` `csr`) and `Trunk.toml` for `crates/brokkr-web`. The
  pack's `PATTERNS.md` is the pick-by-intent guide.
- **Broker-serving → `../skadi` `crates/skadi-api/serve.rs`**: `Router::new().nest("/api/v1", api)`
  with the static wasm UI + SPA fallback added LAST so the API nest wins its routes. Brokkr's
  broker does the same to serve `brokkr-web`. (Skadi predates `aurora-leptos` and hand-vendored
  its own `ds-bundle` — we don't; we take the crate.)

## Binding Design — "Brokkr Monitor" handoff

A high-fidelity design handoff (`Brokkr wasm monitoring frontend.zip` →
`design_handoff_brokkr_monitor/`: a working HTML prototype + spec README) is the
**binding visual + interaction design**. Reproduce it faithfully (hifi); do not
re-style. It defines a fixed-sidebar shell and **seven views**:

1. **Overview** — KPI row + fleet-by-cluster + deployment health + broker throughput
   sparkline + live activity stream + work orders; three swappable grid layouts.
2. **Fleet** — per-cluster panels of clickable agent rows (status/health/labels/heartbeat),
   bound to `GET /api/v1/fleet` + `/fleet/live` WS.
3. **Deployments** — per-stack deployment-object health rollups.
4. **Telemetry** — kube events / pod logs tabs (REST-poll in v1; 6h retention caption).
5. **Work orders** — active (live progress) + history.
6. **Broker health** — Prometheus metric cards (`/metrics`, polled) + internal WS connections.
7. **Webhooks** — subscriptions + recent deliveries.

Plus an **agent-detail slide-over** (with the v1 **run-diagnostic** action) and a
**toast** system. The Aurora visual comes from the **`aurora-leptos`** crate (components +
CSS bundle); the handoff's inline token list just documents the same tokens. The handoff
(README + prototype) ships into `crates/brokkr-web/design/` when scaffolded.

## Implementation Plan (slices)

Vertical slices, each shippable. **Slice 1 is a walking skeleton** that touches every
layer to retire integration risk; later slices add one view at a time.

- **Slice 1 — Walking skeleton** (decomposed into tasks below): scaffold
  `crates/brokkr-web` (Leptos + Trunk) + the `aurora-leptos` dependency; broker serves the
  built wasm + SPA fallback; render the app shell + **one live read view** (Fleet, on
  `/api/v1/fleet`).
- **Slice 2 — Overview** (KPIs, fleet-by-cluster, deployment-health, throughput
  sparkline, live activity, work-orders widgets; three layouts).
- **Slice 3 — Fleet detail + agent slide-over** (incl. the **run-diagnostic** write via
  `POST /api/v1/diagnostics`).
- **Slice 4 — Deployments** (per-stack deployment-object health).
- **Slice 5 — Telemetry** (kube events / pod logs, REST-poll).
- **Slice 6 — Work orders**; **Slice 7 — Broker health** (`/metrics`); **Slice 8 — Webhooks**.
- **Cross-cutting**: toasts, Live/Paused engine, `prefers-reduced-motion`.

Deferred (need new broker endpoints, own decisions): agent activate/deactivate,
replay-last-event, live-WS telemetry streaming, the console's read-access auth boundary.

## Open Questions (remaining, for design)

1. **"Replay last event" mechanism** — what broker action backs it (re-emit the latest
   deployment object / re-trigger reconcile for a stack-agent?), and what auth gates the
   one write the console can make.
2. **`aurora-leptos` rev pinning & upgrades** — which `rev` to pin (currently
   `b32747d…`) and the cadence for bumping it as the design system evolves (unpublished
   git crate). Styling: `<AuroraStyles/>` vs the `aurora-css` Trunk `pre_build` hook.
3. **v1 surface set** — confirm/trim the candidate surfaces above for the first slice.
4. **Build/release integration** — Trunk build of `brokkr-web` wired into the broker
   image/`angreal` build; release cadence (lockstep with the broker version?).
5. **Auth for read access** — does viewing require an admin PAK, or a scoped read token?

## Discovery Status

Discovery is effectively complete. Consumption (depend on `aurora-leptos`), scope
(read-mostly, diagnostic-only write), and hosting (broker-served) are decided in
[[BROKKR-A-0010]], and the initiative is fully decomposed (slice 1 + foundation +
slices 2–8). No spike needed — `leptos-gallery` already demonstrates consuming the crate,
and `../skadi` the broker-serving. Remaining items are design-time, not discovery blockers:
the read-access auth boundary and the build/release wiring (the replay /
activate-deactivate / live-telemetry endpoints are deferred). **Ready to transition
discovery → design.**

## Decisions Log

- 2026-06-27 — Initiative opened (discovery). Stance shift from "UI is a demo, not
  warranted" recorded: `ui-slim` stays a consumer demo; this is a separate, supported
  operator surface. Design basis = Aurora Dark (`@colliery-io/aurora-dark`); stack =
  Leptos (Rust).
- 2026-06-27 — Gating decisions resolved with stakeholder: (1) theme from Aurora Dark
  **CSS tokens only**, build components in Leptos, ds-bundle pattern per Skadi;
  (2) read-mostly observability plane; (3) Leptos **served by the broker**. Reference
  impl = `../skadi`.
- 2026-06-27 — High-fidelity **design handoff** ("Brokkr Monitor", 7 views) received
  and adopted as binding. Framework confirmed **Leptos** (target implementation).
  **Write scope settled: diagnostic-only for v1** (`POST /api/v1/diagnostics`); the
  design's activate/deactivate and the verbal replay-last-event are deferred (no
  endpoints today). Decisions captured in [[BROKKR-A-0010]] (decided). Spike dropped —
  `../skadi` already proves the pattern; slice 1 is a keepable walking skeleton instead.
- 2026-06-28 — **Supersedes the "CSS tokens only, reimplement components" decision.**
  Aurora Dark now ships **`aurora-leptos`** — a full Leptos design-system crate (components,
  widgets, `token::*` + `status_color()`, CSS bundle, `leptos-gallery` example, `PATTERNS.md`).
  Decision: **depend on the crate** (pinned git `rev = b32747d…`, `leptos 0.8`) rather than
  hand-vendor a ds-bundle and reimplement primitives. T-0255 repurposed from "reimplement
  primitives" → "adopt `aurora-leptos` + build the few app-local gap components (slide-over,
  SVG sparkline, segmented health bars)". ADR-0010 updated.
## Status Updates

**2026-06-28 — all 12 tasks implemented + pixel-verified (Ralph run).** The operator console
is built end-to-end and self-verified via the `web-e2e` Playwright+mock harness:
- **Walking skeleton**: `crates/brokkr-web` (Leptos/WASM) served by the broker (`embed-ui` +
  Dockerfile); Aurora `AppShell` shell.
- **All 7 views** render with data: Overview (KPIs + fleet-health bar + throughput sparkline +
  activity), Fleet (+ agent slide-over + run-diagnostic write), Deployments, Telemetry, Work
  orders, Broker health, Webhooks.
- **Foundations**: data layer (`api.rs` gloo-net + Prometheus parser, `models.rs`), app-local
  gap components (Sparkline / SegmentedHealthBar / SlideOver), toast system.

**Known data gaps (need broker enhancements, out of UI scope) — backlog:**
- `/fleet` lacks `cluster_name`/`labels` → flat fleet list, no per-cluster grouping (also limits Overview).
- No "list active work orders" endpoint (only `/work-order-log` history).
- Webhook URLs redacted (`has_url`); no global deliveries feed (per-sub only).
- Telemetry kube-events + pod-logs are per-stack; no global feed.
- No `brokkr_database_queries_total` metric.
- Per-stack deployment-object health rollup needs `/stacks/:id/health` + deployment-objects.

**Deferred (per ADR-0010 / follow-ups):** UI read-access auth (interim: pasted PAK in
localStorage), `/fleet/live` WS (interim 5s poll), Live/Paused gating of polls, diagnostic-result
polling, the 3 Overview layout variants, container-build + live-broker runtime verification.

**2026-06-28 — modal detail pattern applied across all views.** Every list/card view now opens a
centered `aurora-leptos` Modal on click (shared `DetailRow` key/value helper in components.rs):
Fleet→agent (+run-diagnostic), Deployments→stack, Work orders→job, Webhooks→subscription,
Telemetry→event, Broker health→WS connection. All pixel-verified via the harness (`*-modal` scenes).

## Gap-closure follow-up (2026-06-28, user-requested)

Closing the logged data gaps. Findings on re-check: several "gaps" have existing endpoints/fields.
- **Deployment health** — `GET /stacks/:id/health` exists → wire into Deployments modal (UI only).
- **Webhook deliveries** — `GET /webhooks/:id/deliveries` exists → wire into Webhooks modal (UI only).
- **Fleet cluster grouping** — `cluster_name` is already on `Agent`; add to `FleetAgentRecord`
  (broker fleet.rs API + wire structs + builder + OpenAPI regen) then group Fleet/Overview by cluster.
- **Active work orders** — add `GET /work-orders` (DAL `.list()` exists) + OpenAPI regen; UI "Active" panel.

**2026-06-28 — all four selected data gaps closed + verified.**
- **Deployment health** (UI): Deployments stack modal fetches `GET /stacks/:id/health` → overall
  status + per-object health (agent counts). `components::sev()` for domain-status colors.
- **Webhook deliveries** (UI): Webhooks modal fetches `GET /webhooks/:id/deliveries` → recent attempts.
- **Active work orders** (UI): `GET /work-orders` already existed (I'd misread the routes); Work
  orders view gained a live **Active** panel over the History. Admin-PAK gated (degrades to a note).
- **Fleet cluster grouping** (broker + UI): added `cluster_name` (already on `Agent`) to the REST
  `FleetAgentRecord` + the `brokkr-wire` twin + `to_wire`/builders + golden fixture; OpenAPI + both
  SDKs regenerated (checks green). Fleet view now renders one panel per cluster; agent modal shows cluster.
  NOTE: the wire-struct field is a protocol change → wire/release version bump at tag time (lockstep).