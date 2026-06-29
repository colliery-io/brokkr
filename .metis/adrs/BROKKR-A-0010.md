---
id: 0010-operator-console-leptos-wasm
level: adr
title: "Operator Console: Leptos WASM, Aurora-themed, broker-served, read-mostly"
number: 10
short_code: "BROKKR-A-0010"
created_at: 2026-06-28T01:30:34.455792+00:00
updated_at: 2026-06-28T01:30:34.455792+00:00
decision_date: 2026-06-27
decision_maker: Dylan Storey
parent:
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: true
initiative_id: brokkr-operator-console
---

# ADR-0010: Operator Console — Leptos WASM, Aurora-themed, broker-served, read-mostly

Decision for [[BROKKR-I-0031]] (Brokkr Operator Console).

## Context

Brokkr has no supported operator UI. The only prior UI initiative, [[BROKKR-I-0012]],
was cancelled as *"UI is a demo, not warranted"* — and `examples/ui-slim` remains a
consumer-side demo. I-0031 stands up a **separate, supported operator console** for the
people who *run* Brokkr: an at-a-glance live view of the fleet and control plane.

Two inputs constrain the decision:
- A **high-fidelity design handoff** ("Brokkr Operational Monitoring Front End") — a
  working HTML prototype + spec for a fixed-sidebar shell and **seven read-mostly views**
  (Overview, Fleet, Deployments, Telemetry, Work orders, Broker health, Webhooks) plus an
  agent-detail slide-over and toasts. It is bound to the **Aurora Dark** visual language.
- The **Aurora Dark** design system (`@colliery-io/aurora-dark`) is a React/Mantine + CSS
  token package; a sibling control plane (`../skadi`) already consumes it as a portable
  **CSS token bundle** and serves its wasm UI from its own daemon.

We must decide: implementation framework, how to consume Aurora Dark, where the UI is
hosted, and the write scope.

## Decision

1. **Framework — Leptos (Rust/WASM).** Build the console as a new `crates/brokkr-web`
   Leptos app, Trunk-built to wasm. Not Dioxus/Yew, not JS. (Matches Skadi, which is
   also Leptos→wasm — same framework, so its `ds-bundle` + serve patterns transfer directly.)

2. **Design system — consume the `aurora-leptos` crate.** Aurora Dark ships a Leptos
   implementation, **`aurora-leptos`** (the full component + widget set, `token::*` +
   `status_color()`, and the CSS bundle). Depend on it as a **pinned git dependency**
   (`git = "https://github.com/colliery-io/aurora-dark", rev = "b32747d…"`); style via
   `<AuroraStyles/>` (runtime inject) or the `aurora-css` Trunk **`pre_build`** hook (no
   flash) — as `leptos-gallery` dogfoods. Do **not** reimplement primitives or hand-vendor
   a `ds-bundle`, and do not touch the React `@colliery-io/aurora-dark` npm package. Per
   the pack's `PATTERNS.md`, the **app supplies meaning** (state→color/label maps, copy,
   branding) as **data**; the pack ships no app vocabulary. A handful of handoff pieces not
   in the pack (agent slide-over, SVG sparkline, segmented health bars, toast stack) are
   thin **app-local** components built on the pack's tokens/primitives.

3. **Hosting — served by the broker.** `brokkr-broker`'s axum app keeps `/api/v1` and
   adds the built wasm + SPA fallback as the outer layer (mirroring `skadi-api`'s
   `serve.rs`/`assets.rs`). No separate static deploy or desktop target.

4. **Scope — read-mostly observability.** Implement the seven-view monitor as designed.
   The **only write in v1** is the existing **`POST /api/v1/diagnostics`** ("run
   diagnostic"). Agent **activate/deactivate** (in the design pack) and **"replay last
   event"** (an earlier verbal idea) are **deferred** — neither has a broker endpoint
   today; each needs its own backend work and decision.

5. **Data — existing surfaces, poll where no stream exists.** Reads bind to existing
   APIs: `GET /api/v1/fleet` (+ `/fleet/live` WS), agents/generators/registrations,
   `/stacks/:id/{events,logs}`, agent-events, work orders, webhooks, and `/metrics`
   (Prometheus, polled). The design's assumed **live WS for telemetry events/logs** does
   not exist (telemetry is REST-poll, 6h retention) — v1 **polls**; live-streaming
   telemetry is a deferred broker enhancement. Reuse `brokkr-client` types where the wasm
   target permits.

## Alternatives Analysis

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| Framework | **Leptos** | Dioxus / Yew / JS-React | Rust end-to-end (shared types, one toolchain); Leptos is the directed choice and matches Skadi (also Leptos). JS rejected — `ui-slim` already covers a JS demo. |
| Design consumption | **Consume `aurora-leptos` crate** | Vendor CSS tokens + reimplement primitives | Aurora Dark now ships a Leptos crate — depend on it (components + tokens + CSS) instead of rebuilding the component layer by hand. |
| Design consumption | **Consume `aurora-leptos` crate** | `npm i @colliery-io/aurora-dark` / hand-roll styles | The npm package is React/Mantine (unusable from Leptos); hand-rolling loses Aurora consistency. |
| Hosting | **Broker-served wasm** | Separate static deploy / desktop | The console is operator-local; broker-serving adds zero infra and ships in lockstep. |
| Write scope | **Diagnostic only** | Activate/deactivate + replay | Those need new broker endpoints; v1 stays low-blast-radius and unblocked. |

## Rationale

Rust-end-to-end keeps one toolchain and lets the console reuse broker types/SDK.
Consuming `aurora-leptos` gives Aurora Dark consistency across colliery-io control planes
for free and skips rebuilding a component library — the same crate Cloacina's UI uses.
Broker-serving means an
operator who can reach the broker gets the console for free — no extra deployment.
Read-mostly with a single, already-existing write matches the "observability plane"
intent and keeps blast radius minimal.

## Consequences

### Positive
- One language/toolchain (Rust) across broker, agent, SDK, and console; shared types.
- Visual consistency with the Aurora Dark design language via vendored tokens.
- Zero extra infrastructure — the console ships and deploys with the broker.
- Minimal blast radius: read-only plus one pre-existing diagnostic write.

### Negative
- Depend on an **unpublished git crate** (`aurora-leptos`) — pin `rev`, track upstream.
  A few handoff pieces absent from the pack (slide-over, sparkline, segmented bars, toasts)
  are thin **app-local** components.
- A **wasm/Trunk build** is added to the broker image + CI (bigger image, new build step).
- **Telemetry live-streaming is deferred** — v1 polls REST for events/logs.
- The broker now **serves a UI** — a new static-asset + SPA-fallback surface to route and
  secure (must not shadow `/api/v1`; auth boundary for the UI to be decided).

### Neutral
- `examples/ui-slim` is untouched and remains a consumer demo.
- Future writes (activate/deactivate, replay-last-event, live telemetry WS) each need new
  broker endpoints and their own decision — out of scope here.
