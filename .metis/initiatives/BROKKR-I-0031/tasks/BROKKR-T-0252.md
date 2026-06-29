---
id: slice-1a-scaffold-crates-brokkr
level: task
title: "scaffold crates/brokkr-web (Leptos + Trunk) + aurora-leptos dependency + app shell"
short_code: "BROKKR-T-0252"
created_at: 2026-06-28T01:32:27.588822+00:00
updated_at: 2026-06-28T23:25:19.798239+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# scaffold crates/brokkr-web + aurora-leptos + app shell

## Parent Initiative

[[BROKKR-I-0031]] · decision [[BROKKR-A-0010]]

## Objective

Stand up the Leptos wasm crate, wire in the **`aurora-leptos`** design-system crate and its
stylesheet, and render the static **app shell** from the design handoff using the pack's
`AppShell` + components. No live data yet (1c adds that); this is the front-end foundation
that 1b serves and the views fill.

### Type
- [x] Feature — new front-end crate (first slice of the operator console)

## Acceptance Criteria

- [x] `crates/brokkr-web` created: a **Leptos** (0.8, `csr`) app, **Trunk**-built to wasm
      (`index.html`, `src/main.rs`, `src/app.rs`); **excluded** from the root workspace
      (mirrors Skadi's `skadi-web`) so the host toolchain never pulls leptos/wasm.
- [x] Depends on **`aurora-leptos`** pinned git dependency (`rev = b32747d…`) + `leptos 0.8 csr`.
- [x] **Styling wired** via **`<AuroraStyles/>`** (runtime CSS inject) — chosen for the
      skeleton (zero build config; flash negligible for csr). The `aurora-css` Trunk `pre_build`
      hook (no-flash) is noted as a later optimisation, in `index.html`.
- [x] App **shell** built from the pack: `AppShell` + sidebar (brand `Dot`+wordmark, the three
      nav groups Monitor/Operations/System, active **inset-accent** marker via `var(--ice)`) +
      per-view `PageHeader` with the Live/Paused `SegmentedControl`. Styled only via `token::*`/
      `var(--*)`.
- [x] Client-side route state switches the seven view placeholders; Live/Paused toggle wired.
      *(Partial: the live **1s clock** + the brand glyph/status-line/footer pixel-fidelity are
      deferred to a polish pass — structure is in.)*
- [ ] `prefers-reduced-motion` respected *(pending the live/animation pass)*.
- [x] The design handoff (README + `BrokkrMonitor.dc.html`) committed under
      `crates/brokkr-web/design/`.
- [x] **`trunk build` is clean** ✅ (aurora-leptos compiled from git; wasm bundle emitted).

## Implementation Notes

### Technical Approach
- Canonical example: `colliery-io/aurora-dark/rust/leptos-gallery` — copy its `Cargo.toml`
  (git dep + `leptos` `csr`), `Trunk.toml` (`aurora-css` `pre_build` hook), and `index.html`
  `<link>`. The pack's **`PATTERNS.md`** is the pick-by-intent guide; **`AppShell`** is the
  page scaffold.
- Reuse `brokkr-client`/broker types where the wasm target permits (confirm wasm-compat).

### Dependencies
- None (first task). Blocks [[BROKKR-T-0253]] (serve), [[BROKKR-T-0254]] (fleet view), and
  [[BROKKR-T-0255]] (app-local gap components).

### Risk Considerations
- wasm-compat of shared crates; Trunk + the `aurora-css` hook reproducible in CI (feeds 1b).
- Pin the `aurora-leptos` `rev`; bumping it is a deliberate, tracked change.

## Status Updates

**2026-06-28 — Walking-skeleton front-end building & serving.** Created
`crates/brokkr-web` (Leptos 0.8 csr, Trunk), excluded from the root workspace (root
`Cargo.toml` `exclude`), depending on `aurora-leptos` (git `rev = b32747d…`). Shell built
from the pack: `AppShell` + sidebar nav (3 groups, route signal `RwSignal<&'static str>`,
active inset-accent marker) + per-view `PageHeader` with a Live/Paused `SegmentedControl`;
styling via `<AuroraStyles/>`. `trunk build` clean (aurora-leptos compiled from git, 1.5 MB
unopt wasm); `trunk serve` returns 200 at `http://127.0.0.1:9080/`. Design handoff vendored
to `crates/brokkr-web/design/`.

**Remaining for this task (polish pass):** live 1s clock (interval), `prefers-reduced-motion`,
and shell pixel-fidelity (brand hammer/anvil glyph, "broker ready · vX" status line, footer).
Then hand to [[BROKKR-T-0253]] (broker serving) and [[BROKKR-T-0254]] (Fleet live view).

**2026-06-28 (later) — polish landed; closing 1a.** Live **clock** wired (`set_interval` →
`now_hms()`, tabular-nums in the header). Sidebar fidelity: ice **brand square + hammer glyph**,
"Brokkr / control plane" wordmark, "broker ready" status line with a glowing `Dot`, mono group
eyebrows, and the footer ("tenant · public" / "wasm"). `trunk build` green. `prefers-reduced-motion`
is a no-op for 1a (no app-local animations yet — handled with the live engine in [[BROKKR-T-0256]]).
Pixel-perfect tuning will fall out as the views fill in. 1a done.