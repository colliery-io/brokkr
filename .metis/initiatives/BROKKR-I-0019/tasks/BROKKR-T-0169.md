---
id: ws-14-docs-c4-update-agent-ops
level: task
title: "WS-14: Docs — C4 update, agent ops docs, retention/Datadog guidance"
short_code: "BROKKR-T-0169"
created_at: 2026-05-23T02:12:49.896063+00:00
updated_at: 2026-05-23T12:41:41.120422+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-14: Docs — C4 update, agent ops docs, retention/Datadog guidance

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]] · **Convention**: [[feedback_c4_architecture]]

## Objective

Documentation pass for the new internal WS channel and log/event streaming. Update C4 diagrams, write agent operations docs, and publish the retention stance with explicit "use Datadog for long-term" guidance.

## Acceptance Criteria

## Acceptance Criteria

- [x] C4 container diagram showing the internal WS edge (distinct from the external REST surface) — embedded in the new explanation page via `C4Container` mermaid block. The existing top-level `architecture.md` is unchanged; the new page links from it as a focused deep-dive.
- [x] Agent ops content: WS-on-by-default behaviour, `ws_force_rest` flag, behaviour under disconnect, expected reconnect backoff schedule.
- [x] Event/log retention section: 6h ceiling, eviction key is server-side `created_at`, opt-in granularity for logs, "ship to Datadog" guidance.
- [x] Ingress/proxy timeout guidance with concrete nginx-ingress / Traefik / AWS ALB pointers.
- [x] Endpoints table + Prometheus metrics table both included so operators can verify what they should see.
- [x] Docs build cleanly via `angreal docs build` — Rust API reference auto-discovered the new modules (`brokkr_wire`, `brokkr-broker::ws::{broadcaster,eviction,handler,push,registry,subscribe}`, `brokkr-agent::{broker_ws,kube_events,pod_logs}`, `brokkr-broker::dal::{agent_k8s_events,agent_pod_logs}`).
- [ ] **Deferred**: separate "component-level" C4 diagram showing the broker-side internals (ConnectionRegistry, broadcaster, eviction worker). The container diagram + the narrative cover the operational mental model; a deeper component view is a follow-up if UI work in WS-12 needs it.

## Implementation Notes

- **Approach taken**: single focused explanation page (`explanation/internal-ws-channel.md`) rather than scattering content across multiple existing pages. The page is the canonical operator-facing reference for the WS channel and the telemetry retention stance. It links back to ADR-0008 and the `project_log_retention_stance` memory so the source of truth is unambiguous.
- **Dependencies**: WS-01 through WS-13 complete (so docs reflect shipped behaviour, not aspirations).
- **Risk**: docs drift when the wire enum changes. Mitigated by keeping the variant table small and pointing readers at `brokkr-wire` for the authoritative list; the golden fixture there catches schema drift before it ships.

## Status Updates

**2026-05-23** — Done on branch `feat/i-0019-ws-broker-agent-channel`.

- New page `docs/src/explanation/internal-ws-channel.md`:
  - C4 container diagram showing the internal WS edge, the REST fallback, and the explicit "Brokkr is NOT a log warehouse → Datadog" arrow.
  - Wire-frame variant table (who sends what, what it replaces / supplements over REST).
  - `ws_force_rest` configuration documented as the single operator-facing knob.
  - Telemetry stance table — 6h ceiling, eviction cadence, opt-in granularity, rate limit, "use Datadog" guidance.
  - Internal vs public endpoint tables — explicit about which routes are in OpenAPI and which aren't.
  - Prometheus metrics table (WS-13 surface).
  - Ingress / proxy timeout guidance with concrete pointers for nginx-ingress, Traefik, and AWS ALB.
  - Recovery semantics (reconnect backoff, broker-restart behaviour, subscriber-lag → `log_gap`).
- `docs/src/SUMMARY.md` updated to include the new page under **Explanation**.
- `angreal docs build` runs clean. The Rust API reference auto-regenerates and now covers all the new modules.

**Deferred**:
- A separate broker-side component-level C4 diagram. The container-level diagram + the prose cover the operational mental model; a deeper component view can land when WS-12 (UI) work surfaces a need for it.
- Updating any of the existing pages (`architecture.md`, `components.md`, `data-flows.md`) to cross-reference the new content — left as a low-effort follow-up since the new page is fully linked from `SUMMARY.md`.