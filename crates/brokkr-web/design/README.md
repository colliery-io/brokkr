# Handoff: Brokkr Operational Monitoring Front End

## Overview
A read-mostly operational monitoring console for **Brokkr** (Colliery's environment-aware, multi-cluster Kubernetes deployment control plane: broker + agents + stacks + deployment objects + telemetry + work orders + webhooks). The console gives an operator an at-a-glance live view of the fleet and lets them perform a small set of safe actions. It is intended to ship as a **WASM front end** (Rust — Leptos / Yew / Dioxus) talking to the broker's HTTP/`metrics`/WebSocket surfaces.

The design is a navigable shell with a fixed sidebar and seven views, an agent detail slide-over, toast notifications, and a persistent "live" engine (ticking counters, streaming events/logs, aging heartbeats, pulsing status).

## About the Design Files
The file in this bundle — **`BrokkrMonitor.dc.html`** — is a **design reference created in HTML** (a working prototype demonstrating intended look, layout, and behavior). It is **not production code to copy directly**. It is authored as a "Design Component" (a custom HTML runtime used for prototyping); the markup and a JavaScript `Component` class drive it, with all data mocked client-side.

Your task is to **recreate this design in the target WASM codebase** using that environment's established patterns (Rust reactive framework, signals/stores, real API + WebSocket clients). If no frontend environment exists yet, choose the most appropriate Rust/WASM framework for the project and implement there. Treat the mock data structures in the prototype as a guide to the **shape** of the data; bind the real views to Brokkr's API/SDK.

The Aurora Dark design system (see **Design Tokens** below) is the binding visual style. In the prototype it's loaded as CSS custom properties from `_ds/.../tokens/*.css`. In the WASM app, port those tokens to your stylesheet (a `:root { … }` block of the same custom properties) and reference them via `var(--*)` exactly as the prototype does.

## Fidelity
**High-fidelity (hifi).** Final colors, typography, spacing, radii, and interactions are all specified and should be reproduced pixel-accurately using the Aurora Dark tokens. Recreate the UI faithfully; do not re-style.

---

## Global Layout & Shell

- Root: `display:flex; height:100vh; overflow:hidden; background:var(--bg); color:var(--fg); font-family:var(--font-sans)` (IBM Plex Sans).
- **Sidebar** (fixed, `width:236px`, `flex:none`, `background:var(--sidebar)`, right border `1px var(--border-soft)`):
  - Brand block: 26×26 rounded-7px accent square holding a small hammer/anvil line glyph (stroke `#0b0d10`), wordmark "Brokkr" (16/600, `--fg-bright`), mono eyebrow "control plane" (9.5px, `.13em`, uppercase, `--faint`). Below it a live status line: pulsing 7px `--ok` dot with glow ring + "broker ready · v0.5.0" (mono 10.5px, `--muted`).
  - **Nav** (scrollable, `padding:10px 8px`), three groups with mono uppercase eyebrow labels (9.5px, `.12em`, `--fainter`):
    - **Monitor**: Overview, Fleet (badge = agent count), Deployments, Telemetry
    - **Operations**: Work orders (badge = active count), Webhooks
    - **System**: Broker health
  - Nav item: `display:flex; gap:10px; padding:8px 10px; border-radius:8px; font:13/500`. Inactive `color:var(--muted)`; **active** = `color:var(--fg-bright)`, background `color-mix(in srgb, {accent} 14%, transparent)`, and `box-shadow:inset 2px 0 0 {accent}` (left marker). Hover = faint ice wash `rgba(127,178,255,.08)`. Each item has a 9px optional kind-square marker slot (kept empty here) and an optional right-aligned mono badge.
  - Footer: `border-top:1px var(--border-fainter)`, mono 10px — "tenant · public" left, "wasm · 0.5.0" right.
- **Main** (`flex:1; overflow-y:auto; padding:20px 26px 40px`). Inner column `max-width:1500px; margin:0 auto; gap:16px`.
  - **Header row**: left = page title (22/600, `--fg-bright`) + mono sub (11px, `--faint`). Right = (Overview only) a 3-way layout segmented control, then a **Live/Paused** toggle button, then a mono clock (`HH:MM:SS`, updates each second).
  - **Live toggle**: `padding:7px 12px; border-radius:8px; font:12/500`. When live: border `color-mix(in srgb,var(--ok) 40%,transparent)`, bg `color-mix(in srgb,var(--ok) 12%,transparent)`, text `--ok`, plus a 7px pulsing dot. When paused: `--control` bg, `--border-control` border, `--muted` text, static dot.

**Scrollbars** (all scroll areas): 9px wide, thumb `#283039` radius 6, transparent track.

---

## Screens / Views

### 1. Overview
At-a-glance command view. Has **three swappable layouts** via the header segmented control (segment: `padding:5px 12px; border-radius:7px; font:11.5/500`; active = accent fill + `#0b0d10` text; inactive = transparent + `--muted`). All three render the **same six widgets**, re-arranged with CSS grid template areas:

- **command** (default): `grid-template-columns: minmax(0,1fr) ×3 + minmax(0,1.1fr)`, areas `"kpis kpis kpis kpis" / "fleet fleet stream stream" / "deploy jobs stream stream" / "flow flow stream stream"`, rows `auto 1fr 1fr auto`. (Activity stream is the tall right column.)
- **grid**: 3 equal columns, areas `"kpis kpis kpis" / "fleet deploy jobs" / "stream stream flow"`, rows `auto 1fr 1fr`.
- **stream**: `1.5fr 1fr`, areas `"stream kpis" / "stream fleet" / "stream deploy" / "flow jobs"` (activity-tail forward).
- Grid `gap:13px; min-height:calc(100vh - 130px)`.

Widgets:
- **KPI row** (`kpis`): auto-fit cards `minmax(148px,1fr)`, gap 11. Each: `--panel` bg, `1px var(--border)`, radius 10, `padding:13px 15px`. Mono uppercase label (10px) with a leading 7px status dot; value 29/600 (`cl-tnum` tabular) colored by meaning; mono 10px unit below. The six KPIs: **Active agents** `active/total` (white), **WS channel** count (teal), **Healthy** objects (ok green), **Degraded** objects (gold), **Failing** objects (bad red), **Req/min** broker http (ice).
- **Fleet by cluster** (`fleet`): titled panel. Per cluster: env color square + mono name + mono `n/total up`; below a 7px segmented bar (healthy `--ok` / degraded `--gold` / failing `--bad` / offline `--border-control`) widths = percentage of that cluster's agents.
- **Deployment health** (`deploy`): three big counts (healthy/degraded/failing, 26/600) then a per-stack list: status dot + mono stack name + mono `Nh✓ Nd⚠ Nf✕` counts.
- **Broker throughput** (`flow`): three live counts (http req/min ice, ws msg/min teal, db q/min violet, 22/600) + an SVG area sparkline (44-point history, `viewBox 0 0 240 52`, line `var(--ice)` 1.6px, fill `color-mix(in srgb,var(--ice) 12%,transparent)`).
- **Live activity** (`stream`): scrollable event feed. Each row: 6px dot colored by severity (Warning=`--gold`, Normal=`--ok`), mono reason + mono "ago", one-line message (ellipsized), mono `ns/<namespace>`. New events prepend ~every 2s when live.
- **Work orders** (`jobs`): per active order — mono short id, mono type, thin progress bar (`--ice`, or `--muted` when pending), status pill.

### 2. Fleet
- **KPI strip**: Total agents (white), Active (`--ok`), Degraded (`--gold`), Failing (`--bad`).
- One **panel per cluster** (header: env square + name + mono region + `n/total active`). Rows of agents, each clickable (opens slide-over):
  - 7px health dot with glow ring (`color-mix(in srgb,{healthColor} 16%,transparent)`); pulses if active and last beat < 8s.
  - Mono agent name (`min-width:150px`), status pill, health pill, label chips (mono 9.5px on `--control`), optional "⇄ ws" (teal) if on internal WS channel, and a right-aligned mono "ago" heartbeat colored ok/gold/faint by recency.
  - Row hover = `--panel-2` background.

### 3. Deployment health (Deployments)
- One **panel per stack** (header: mono stack name + health pill + label chips; right: mono "gen · {generator}"). Rows of deployment objects:
  - Mono object id, action pill (`apply`/`delete`; delete=`--bad`, else `--teal`), mono kind (e.g. "Deployment + Service", "DaemonSet", "StatefulSet"), then mono per-agent rollup `N✓` (ok) `N⚠` (gold) `N✕` (bad), and a mono "ago".

### 4. Telemetry
- Tab segmented control: **Kube events** / **Pod logs**. Caption (right): gold "⚠ 6h retention window · ship to Datadog for long-term".
- **Kube events**: rows — severity pill (`normal`/`warning`), mono reason, message (ellipsized), mono `ns/…`, mono "ago".
- **Pod logs**: a single `--inset` well with a mono `<pre>` (11.5px, line-height 1.7, wrap) live-tailing lines formatted `[HH:MM:SS] ns/pod/container: <line>`. New line appended ~every second when live; keep last ~90 visible.

### 5. Work orders
- **Active** section: rows — mono id, type chip, status pill, progress bar (ice; muted when pending), mono meta, mono "ago". Progress advances live; reaches `completed` at 100%.
- **History** section: rows — mono id, type chip, status pill (`completed`/`failed`), mono detail, mono "ago".

### 6. Broker health (System)
- **Metric cards** (auto-fit `minmax(160px,1fr)`): Active agents, WS connected (teal), Http req/min (ice), DB queries/min (violet), Stacks, Deploy objects. Each card: mono uppercase label, 26/600 value, mono sub = the Prometheus metric name (e.g. `brokkr_active_agents`, `brokkr_ws_connected_agents`, `brokkr_http_requests_total`, `brokkr_database_queries_total`, `brokkr_stacks_total`, `brokkr_deployment_objects_total`).
- **Internal WS connections** panel: rows — pulsing teal dot, mono agent name, mono cluster, mono `N msg/s`, mono "up {uptime}".

### 7. Webhooks
- **Subscriptions**: cards (auto-fit `minmax(260px,1fr)`) — name (12.5/600) + state pill (`enabled`/`disabled`), mono url (ellipsized), event chips (mono, ice-tinted).
- **Recent deliveries**: rows — mono id, mono event (ice), mono hook name, mono "try N", status pill (`success`/`failed`/`dead`/`pending`), mono "ago".

### Agent detail slide-over
Opens from any Fleet row (or any agent reference). Right-anchored panel `width:430px; max-width:92vw; height:100vh`, `--panel` bg, left border `1px var(--border)`, shadow `-24px 0 60px rgba(0,0,0,.5)`, over a `rgba(6,8,11,.55)` scrim (click scrim or ✕ to close).
- Header: mono agent name (16/600 `--fg-bright`) + mono agent id; ✕ close.
- 2×2 grid: Cluster, Last heartbeat (colored), Status pill, Health pill (each with mono uppercase label).
- Labels chip row.
- **Action buttons**: **Activate/Deactivate** (when active: `--bad`-tinted text on tinted bg; when inactive: solid `--ok` fill, `#0b0d10` text, 600) and **⌕ Run diagnostic** (`--control` bg, `--border-control` border).
- **Diagnostic** result block (appears after running): header + status pill. While pending, an indeterminate sweep bar (`--ice`, `brk-sweep` 1.1s) + "collecting pod statuses, events, log tails…"; on completion, a mono `<pre>` report (pod statuses / events / log tails). ~2.4s simulated latency in the prototype — wire to the real diagnostic endpoint.
- **Recent events** list: mono "ago", mono type (Apply/Heartbeat/Reconcile), result pill (`success`/`failure`).

### Toasts
Bottom-right stack (`z-index:60`, gap 9). Each: `--control` bg, `1px var(--border-control)`, **3px left border in the toast's color**, radius 9, `padding:10px 14px`, shadow `0 12px 30px rgba(0,0,0,.4)`, min-width 230. 7px leading dot + 12px message. Auto-dismiss after 3.4s. Colors: ok green / bad red / info ice. Emitted on agent activate-deactivate and on diagnostic request/completion.

---

## Interactions & Behavior

- **Navigation**: sidebar items switch the active view (single-page route state; no full reload). Active item gets the inset-left accent marker.
- **Overview layout switch**: segmented control swaps the grid template areas only — same widgets reflow. Default `command`.
- **Live engine** (1s interval, gated by the Live/Paused toggle and persisted as a prop default):
  - Clock ticks each second.
  - Agent heartbeats age each second; active agents occasionally reset to 0 (WS agents more often than poll agents) to simulate fresh beats. Dots pulse when a beat is fresh (<8s).
  - Broker metrics (req/ws/db) jitter around baselines each second; the throughput history array shifts (keeps last 44) feeding the sparkline.
  - When live: a kube event prepends ~every 2s (cap 60); a pod-log line appends every second (cap 200).
  - Work orders in `claimed` advance progress; `pending` may transition to `claimed`; at 100% → `completed`.
  - **Pausing** freezes event/log streaming (counters/clock still tick in the prototype — match or adjust to taste).
- **Pulse animation** `brk-pulse`: opacity `1 ↔ .38`, 1.6s ease-in-out, on live status dots and progress.
- **Sweep animation** `brk-sweep`: a 30%-wide bar translateX `-100% → 320%`, 1.1s, for indeterminate diagnostic progress.
- **Hover**: nav items faint ice wash; table rows lift to `--panel-2`; buttons brighten ~8%.
- **Safe actions** (the only writes): activate/deactivate an agent (optimistic state flip + toast) and run a diagnostic (async, toast on request + completion). Everything else is read-only.
- Respect `prefers-reduced-motion` — disable pulses/sweeps.

## State Management
Suggested signals/stores for the WASM app (the prototype holds these in one component `state`):
- `route` — active view (`overview|fleet|deployments|telemetry|jobs|system|webhooks`).
- `overview_layout` — `command|grid|stream`.
- `live_on` — bool (drives the live engine; default from config).
- `tele_tab` — `events|logs`.
- `selected_agent_id` — opens the slide-over; `null` = closed.
- `diag` — `{ agent_id, status: pending|completed, result }` for the active diagnostic.
- `toasts` — list of `{ id, kind, msg, color }` with timed removal.
- Live data: `agents`, `clusters`, `stacks` (+ `objects`), `events`, `logs`, `work_orders` (active+history), `webhook_subs`, `webhook_deliveries`, `metrics` (req/ws/db) + `throughput` ring buffer.

**Data fetching (replace the mock):**
- Initial load: GET fleet (agents grouped by cluster), stacks + deployment objects, work orders, webhook subscriptions/deliveries.
- Live: subscribe to the broker's **WebSocket** for events/log tails/heartbeats/work-order progress; poll `/metrics` (Prometheus) for the broker counters on an interval.
- Actions: PATCH agent active state; POST diagnostic request → poll/stream result.

## Design Tokens (Aurora Dark — port these to the WASM app's stylesheet)
Define as `:root` custom properties and reference via `var(--*)`.

**Surfaces:** `--bg:#0e1116`, `--sidebar:#12161c`, `--panel:#161a21`, `--panel-2:#13171e`, `--inset:#0a0c10`, `--control:#1b2129`.
**Borders:** `--border-control:#2a3340`, `--border:#232a34`, `--border-soft:#1d232c`, `--border-fainter:#15191f`.
**Text:** `--fg-bright:#f1f4f8`, `--fg:#e6e9ee`, `--fg-2:#c3cbd5`, `--muted:#8b95a3`, `--faint:#5b6573`, `--fainter:#4a525e`.
**Accents:** `--ice:#7fb2ff` (primary), `--teal:#5fd0c5`, `--violet:#9d8cff`, `--gold:#d8a657` (warning).
**Status:** `--ok:#4bd07f`, `--bad:#f06464`, `--skip:#cf83a4`.
**Type:** IBM Plex Sans (UI) + IBM Plex Mono (identifiers, numbers, timestamps, captions, eyebrows). Weights **400/500/600 only**. Scale: page title 22/600, section header 13/600, body 13, big metric 26–30/600 with tabular figures (`font-variant-numeric: tabular-nums`), captions/pills 10.5–11 mono, eyebrow labels mono uppercase `.1–.13em`.
**Radii:** pills 10, controls/inputs 9, cards 11, inline buttons 7, status squares 2.
**Spacing/density:** compact. Card padding ~13–17px; row padding ~9–11px; section gaps ~13–16px; chip/control gaps 5–8px. Sidebar 236px.
**Elevation:** flat — separate with borders + surface steps. Shadows only on floating surfaces (slide-over, toasts). Status dots may carry a glow ring `0 0 0 3px {color}@~16%`.
**Casing/voice:** sentence case headings; raw lowercase identifiers/statuses in mono; translate machine enums to friendly labels; no emoji.

The canonical token files are in the design system at `_ds/aurora-dark-design-system-a8591a48-9174-4266-8b9c-f4e26d291f0c/tokens/` (`colors.css`, `typography.css`, `spacing.css`, `fonts.css`, `base.css`) — copy these verbatim into the WASM app's assets.

## Iconography & Assets
- **No external image assets.** The brand glyph is a small inline SVG (hammer/anvil, 2 strokes, `stroke:#0b0d10`) inside the accent square — re-draw inline or replace with the real Brokkr mark.
- Functional glyphs used as text: `✓ ⚠ ✕ ⇄ ⌕ ✕`(close). In production prefer **Tabler Icons** (1.5 stroke line icons) per the Aurora Dark system, or keep these unicode glyphs.
- **IBM Plex Sans + Mono** webfonts come from the design system's `tokens/fonts.css` (gstatic woff2). Bundle them with the WASM app; set `font-display:swap`.
- Status color is conveyed by dots/pills mapped from raw status strings — map status → color centrally (don't hand-pick hexes per element).

## Files
- `BrokkrMonitor.dc.html` — the full prototype (markup + `Component` logic class with mock data + the live engine). All seven views, the slide-over, toasts, and the three overview layouts are in this one file. Read the `renderVals()` method for the exact per-view data shapes and the `tick()` method for the live-update logic.
- `_ds/aurora-dark-design-system-…/tokens/*.css` — Aurora Dark design tokens to port. (Bound design system; not duplicated into this folder — copy from the project's `_ds/` directory.)

## Notes
- This is a **fresh monitoring-first design**, intentionally not a port of Brokkr's existing "slim" demo UI (which is a thin API/SDK demonstrator).
- "Agent" = a Brokkr agent process (one per managed cluster) — **not** a Kubernetes node. Names are `prod-agent-01`, `staging-agent-03`, etc. The console monitors *Brokkr's own system* (broker, agents, stacks, work orders, webhooks), not raw Kubernetes infrastructure — though the diagnostic action and Telemetry view do surface pod statuses / kube events / pod-log tails that the agent reports back through the broker.
