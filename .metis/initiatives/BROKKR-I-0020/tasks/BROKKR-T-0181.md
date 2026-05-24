---
id: document-agent-ws-url-config-docs
level: task
title: "Document agent ws_url config: docs, helm values, ADR amendment, C4 caption"
short_code: "BROKKR-T-0181"
created_at: 2026-05-24T14:40:00+00:00
updated_at: 2026-05-24T14:40:00+00:00
parent: BROKKR-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] `docs/src/explanation/internal-ws-channel.md` — add a "When to use ws_url"
      subsection covering: default behavior, when to override, the production
      patterns above, and the required URL format
      (`ws://host:port/internal/ws/agent` or `wss://`)
- [ ] Helm chart `values.yaml` — expose `agent.wsUrl` as an optional value,
      mapped to `BROKKR__AGENT__WS_URL` env in the agent Deployment template.
      Default null (don't set the env, preserving current behavior)
- [ ] ADR-0008 — one-paragraph amendment noting "WS endpoint MAY be on a
      different ingress than REST; agent gates on `ws_url` config when set,
      otherwise derives from `broker_url`". Keep the amendment scoped — don't
      reopen the broader decisions
- [ ] C4 deployment diagram (last touched by BROKKR-T-0169 / I-0019 WS-14)
      caption updated to note the split-ingress variant. Diagram itself
      doesn't need redrawing unless someone wants to add the variant explicitly
- [ ] Cross-reference from the docs page back to the ADR amendment
- [ ] `angreal docs build` still green

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

*To be added during implementation*
