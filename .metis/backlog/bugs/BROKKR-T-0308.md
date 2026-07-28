---
id: chart-values-that-silently-do
level: task
title: "Chart values that silently do nothing: agent ServiceMonitor, telemetry.collector sidecar, metrics.enabled, service.annotations"
short_code: "BROKKR-T-0308"
created_at: 2026-07-28T15:14:25.420438+00:00
updated_at: 2026-07-28T15:14:25.420438+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Chart values that silently do nothing: agent ServiceMonitor, telemetry.collector sidecar, metrics.enabled, service.annotations

## Objective

Four chart values are accepted, documented, and render nothing useful. Each fails the same way — the operator sets it, `helm install` succeeds, and the capability simply is not there. Found 2026-07-28 while writing the chart READMEs (BROKKR-T-0294), which now document each as a caveat; this ticket is to make the code match the promise or remove the promise.

1. **Agent ServiceMonitor is inert.** `charts/brokkr-agent/templates/servicemonitor.yaml` selects a service port named `health`, but the agent chart renders **no Service at all**. `metrics.serviceMonitor.enabled=true` therefore produces a ServiceMonitor that scrapes nothing. Either render a Service exposing the agent's health/metrics port, or remove the ServiceMonitor template.
2. **`telemetry.collector.enabled` is a trap in both charts.** Setting it true repoints `BROKKR__TELEMETRY__OTLP_ENDPOINT` at `http://localhost:4317`, but neither Deployment renders a sidecar — so traces go to a port nothing is listening on, and telemetry silently stops working. Every `telemetry.collector.*` sub-key is inert. This is worse than the others: it takes working telemetry and breaks it.
3. **`metrics.enabled` does not gate metrics** in either chart — it only gates a NetworkPolicy rule. The `/metrics` endpoint is always served regardless. The name and the values-file comment both mislead.
4. **`service.annotations` (broker) is a no-op.** It exists only as a commented example in `values.yaml`; `templates/service.yaml` renders no `metadata.annotations`. This matters for LoadBalancer setups where annotations are the whole configuration surface.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (no data loss or security exposure, but each silently withholds a capability the operator believes they enabled; item 2 actively breaks working telemetry)

## Acceptance Criteria

- [ ] Each of the four is either implemented or removed — no value survives that is accepted and does nothing.
- [ ] Where a value is removed, the chart README caveat added by BROKKR-T-0294 is removed with it rather than left describing a value that no longer exists.
- [ ] `helm template` assertions (or chart tests via `angreal helm test`) cover whichever direction is chosen, so these cannot silently regress.
- [ ] If the collector sidecar is implemented rather than removed, verify end-to-end that traces actually arrive — the current failure mode is precisely that nobody checked.

## Status Updates

*To be added during implementation*
