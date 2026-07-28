---
id: no-chart-render-assertions-values
level: task
title: "No chart-render assertions: values that do nothing can regress silently, as four just did"
short_code: "BROKKR-T-0313"
created_at: 2026-07-28T20:55:32.830942+00:00
updated_at: 2026-07-28T20:55:32.830942+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# No chart-render assertions: values that do nothing can regress silently, as four just did

## Objective

Add render-level assertions over both Helm charts so a value that stops taking effect fails a check rather than shipping.

BROKKR-T-0308 fixed four values that were accepted, documented, and rendered nothing — a ServiceMonitor selecting a port no Service defined, a collector endpoint pointing at a sidecar that was never rendered, a `metrics.enabled` that only gated a NetworkPolicy rule, and a `service.annotations` that appeared nowhere in the output. All four passed `helm lint`. All four survived multiple releases. Two were only found because someone was writing the README and read the templates line by line.

That is the actual defect: **nothing in CI asserts that setting a value changes the rendered output.** `helm lint` checks syntax and schema, not effect. The four fixes are individually correct but there is no reason to believe they will stay correct, and no reason to believe the remaining values are any better than the ones that were checked.

Deferred from BROKKR-T-0308's acceptance criterion 3, which asked for exactly this and which that ticket could not satisfy: verification there was by hand, and `angreal helm test` needs a live cluster.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (no live defect; this is what stops the class from recurring)

### Technical Debt Impact
- **Current Problems**: a value can silently stop taking effect, and the failure is invisible — `helm install` succeeds, the capability is absent, and the operator believes it is enabled. This class of bug is discovered only by reading templates.
- **Benefits of Fixing**: the next inert value fails a check instead of shipping. Also gives the chart READMEs' values tables something to be checked against, since they are currently maintained by hand and stamped with a chart version.
- **Risk Assessment**: without it, BROKKR-T-0308's fixes are a snapshot rather than a guarantee, and the same four could regress under a refactor with nothing to catch it.

### Technical Approach

`helm template` plus assertions is enough and needs no cluster — this should run in ordinary CI, not the k3s e2e path. For each security- or capability-relevant value: render with it set and with it unset, and assert the rendered output differs in the expected way. The high-value cases are the ones already known to have failed or to matter:

- `metrics.podMonitor.enabled` renders a PodMonitor whose `port` matches a declared container port name.
- `networkPolicy.allowMetricsScraping` toggles the metrics ingress rule.
- `service.annotations` appears on the Service.
- `telemetry.otlpEndpoint` reaches `BROKKR__TELEMETRY__OTLP_ENDPOINT`.
- Each `existingSecret` value omits its plaintext counterpart from the ConfigMap **and** adds the `secretKeyRef` — the pairing that makes the credential work is exactly what a refactor could half-break.
- `broker.pakHash` renders only when non-empty (the empty-value trap that left the public dev credential live).

A generic guard is worth considering alongside the specific ones: assert that every leaf key in `values.yaml` appears somewhere in `templates/`, with an explicit allowlist for keys consumed by subcharts or intentionally inert. That would have caught all four of BROKKR-T-0308's defects mechanically, without anyone needing to suspect them.

## Acceptance Criteria

- [ ] Render assertions cover the values listed above for both charts, runnable without a cluster.
- [ ] They run in CI on chart changes, and fail loudly rather than warning.
- [ ] A deliberately broken template (e.g. drop the `service.annotations` block) makes the suite fail — verify the check actually catches the class, rather than trusting that it would.
- [ ] Decide on the generic "every values key is referenced" guard, and either implement it with a documented allowlist or record why not.

## Status Updates

*To be added during implementation*
