---
id: document-agent-ws-url-config-docs
level: task
title: "Document agent ws_url config: docs, helm values, ADR amendment, C4 caption"
short_code: "BROKKR-T-0181"
created_at: 2026-05-24T14:40:00+00:00
updated_at: 2026-05-24T14:40:00+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0020
---

# Document agent ws_url config: docs, helm values, ADR amendment, C4 caption

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

[[BROKKR-T-0171]] (A2 chaos test) introduced `BROKKR__AGENT__WS_URL` as an
optional agent config. Defaults preserve existing behavior (WS derived from
`broker_url`) so there's no breaking change, but the option enables a real
production topology that wasn't supported before: **WS through a different
ingress than REST**.

This is a small but real operational/architectural change. It deserves
documentation so operators discover it deliberately rather than by reading
the source. Production patterns where this matters:

- WS-aware LB with sticky sessions in front of WS, plain ALB in front of REST
  (AWS pattern: ALB's WS support is fine but its idle timeouts are aggressive
  vs. NLB)
- Different SSL termination policies (long-lived WS often wants no edge termination)
- Separate ingress controller for streaming traffic so a noisy REST burst
  can't starve WS connections

## Acceptance Criteria

## Acceptance Criteria

- [x] `docs/src/explanation/internal-ws-channel.md` — added "When to use
      ws_url" subsection (default behavior, when to override, the production
      patterns, URL format `ws://host:port/internal/ws/agent` / `wss://`)
- [x] Helm chart `values.yaml` — `agent.wsUrl` (default null), mapped to a
      conditional `BROKKR__AGENT__WS_URL` in the agent ConfigMap template.
      Verified via `helm template`: emitted only when set, absent by default
- [x] ADR-0008 — "Amendments" subsection added (2026-05-26), scoped to the
      split-ingress relaxation, referencing [[BROKKR-T-0171]]; broader
      decisions untouched
- [x] C4 container-diagram caption updated with a blockquote noting the
      default single-ingress topology vs. the `ws_url` split-ingress variant
      (diagram not redrawn, per the task's guidance)
- [x] Cross-references: docs page → ADR-0008; C4 caption → Configuration
      subsection; ADR amendment → docs page
- [x] `angreal docs build` green (exit 0)

## Implementation Notes

### Technical Approach

This is documentation + helm + ADR work; no code changes.

- The docs page section should be 5-10 lines: what the option does, when to
  set it, and the URL format. Lean on the existing config code comment as
  the source of truth.
- The helm change is one line in `values.yaml` and one conditional `env`
  entry in the agent deployment template. Use the same conditional pattern
  the chart already uses for other optional agent envs.
- The ADR amendment goes under a new "Amendments" subsection at the bottom
  of ADR-0008, dated 2026-05-24, referencing [[BROKKR-T-0171]] as the trigger.

### Dependencies

None — the config option is already shipped on the branch
([[BROKKR-T-0171]] commit).

### Risk Considerations

- Keep the ADR amendment terse. The original 9-point decision in ADR-0008
  doesn't need rework; we're just relaxing one implicit assumption.
- The C4 caption tweak risks scope creep into redrawing the diagram. If
  the existing single-ingress diagram is fine and the caption can clarify
  ("Diagram shows the default single-ingress topology; `ws_url` enables a
  split-ingress variant"), that's the win — don't redraw.

## Status Updates

### 2026-05-26 — Done (docs + helm + ADR + C4 caption)

All acceptance criteria met. Pure docs/helm/ADR work, no code change (the
`ws_url` config option already shipped on the branch with [[BROKKR-T-0171]]).

- Docs: "When to use ws_url (split WS / REST ingress)" subsection in
  `internal-ws-channel.md` — default derive-from-`broker_url`, the three
  production split-ingress patterns, URL format, and the gating behavior,
  cross-linked to ADR-0008.
- Helm: `agent.wsUrl` (default `null`) in `values.yaml`; conditional
  `BROKKR__AGENT__WS_URL` in `configmap.yaml`. `helm template` confirms it's
  emitted only when set and omitted by default (preserving current behavior).
- ADR-0008: new "Amendments" subsection (2026-05-26) scoped strictly to the
  split-ingress relaxation; the original 9-point decision is untouched.
- C4: container-diagram caption gained a blockquote distinguishing the default
  single-ingress topology from the `ws_url` split-ingress variant — diagram
  not redrawn (per the risk note about scope creep).
- `angreal docs build` exits 0. (One pre-existing plissken WARN about a
  `<string>` tag in the generated `brokkr-utils/config.md` rustdoc is
  unrelated to these edits.)