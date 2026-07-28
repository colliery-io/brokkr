---
id: chart-values-that-silently-do
level: task
title: "Chart values that silently do nothing: agent ServiceMonitor, telemetry.collector sidecar, metrics.enabled, service.annotations"
short_code: "BROKKR-T-0308"
created_at: 2026-07-28T15:14:25.420438+00:00
updated_at: 2026-07-28T20:23:00.914512+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/active"


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

## 2026-07-28 — verification and DECISIONS (Dylan)

All four confirmed against the templates. **Item 1 is worse than filed:** the agent chart has no Service *and* the container declares no named port at all — its probes hardcode `8080` as a number — so agent metrics are unreachable by Prometheus Operator through any mechanism as shipped, not merely via a broken selector.

1. **Agent metrics → PodMonitor + named container port.** Replace `servicemonitor.yaml` with a PodMonitor and name the container port. Idiomatic for a workload with no Service: the agent has nothing else calling it, so a Service would exist purely to satisfy the scraper. Values move `metrics.serviceMonitor.*` → `metrics.podMonitor.*`. Breaking in name only — the old path never worked, so nothing regresses.
2. **`telemetry.collector.*` → remove** from both charts, along with the if/else that rewrites `OTLP_ENDPOINT`. `telemetry.otlpEndpoint` stays and works. Rejected implementing a sidecar: it adds a container, a config surface, and an upstream image to track, for a capability an operator's mesh or collector deployment already provides.
3. **`metrics.enabled` → `networkPolicy.allowMetricsScraping`.** The misleading name *is* the defect; it only ever gated a NetworkPolicy rule and the endpoint is always served. Rejected adding a code flag to disable `/metrics`: an always-available metrics endpoint is normal, and disabling it mainly matters if the port is otherwise exposed — which is the NetworkPolicy's job anyway.
4. **`service.annotations` → implement** (no decision needed). Three lines, and annotations are the entire configuration surface for LoadBalancer services.

Note items 1–3 are user-visible values changes. None causes a behavioral regression, because none of the removed or renamed values did anything, but all three belong in release notes as breaking chart-interface changes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each of the four is either implemented or removed — no value survives that is accepted and does nothing.
- [ ] Where a value is removed, the chart README caveat added by BROKKR-T-0294 is removed with it rather than left describing a value that no longer exists.
- [ ] `helm template` assertions (or chart tests via `angreal helm test`) cover whichever direction is chosen, so these cannot silently regress.
- [ ] If the collector sidecar is implemented rather than removed, verify end-to-end that traces actually arrive — the current failure mode is precisely that nobody checked.

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07` (commit `183f278`). `helm lint` passes on both charts; both render against defaults, the dev values, and all three environment values files.

All four decisions implemented as recorded. Verified renders directly: the PodMonitor emits `port: health`, the agent container now declares `name: health / containerPort: 8080 / protocol: TCP`, and both probes reference the port **by name** rather than the literal — so the number lives in one place and cannot drift if `health_port` is ever made configurable.

**The broker's ServiceMonitor was verified correct and left alone.** The broker renders a Service with a port named `http` matching a declared `containerPort`, and `/metrics` is on that router, so it already worked. Converting it to a PodMonitor for symmetry would have broken something that was not broken.

Agent metrics port confirmed as 8080 from `cli/commands.rs` (`health_port.unwrap_or(8080)`), with `/metrics` registered on the same router as `/healthz` and `/readyz`, and no `BROKKR__AGENT__HEALTH_PORT` set by the chart.

**`metrics.<monitorKind>.*` wrapper deliberately kept** rather than hoisted to a top-level key: hoisting in the agent alone would break the parallel between the charts, and hoisting in both would add a third breaking rename to the broker that was never decided. The misleading part — `metrics.enabled` — is gone from both, and each block now carries a header comment stating the endpoint is always served and access control belongs to `networkPolicy`.

**A real trap documented in the agent README:** Prometheus matches PodMonitors via `podMonitorSelector`, a *different* selector from `serviceMonitorSelector`. Anyone who had set `additionalLabels` for the old ServiceMonitor path needs to check their Prometheus resource, or the PodMonitor will be ignored for a reason that looks nothing like the cause.

**Dead keys fixed beyond the agent's file ownership:** `charts/brokkr-agent/values-dev.yaml` and `charts/brokkr-broker/values-dev.yaml` both still set `metrics.enabled: true` — exactly the defect class this ticket closes. Both replaced with a comment pointing at `networkPolicy.allowMetricsScraping`.

**Still open — acceptance criterion 3.** No `helm template` assertions or chart-test coverage was added, so nothing prevents these from regressing. Verification was by hand. The chart test harness (`angreal helm test`) needs a live cluster and was out of scope here; this is the criterion that would actually keep the fix from rotting, and it deserves its own follow-up rather than being quietly dropped.

**Not done: `Chart.yaml` version bump.** Both charts remain 0.8.4 and both README values tables are stamped for that version. These are breaking interface changes that would normally warrant a bump, but the project uses lockstep versioning driven by git tags, so bumping by hand here would fight the release process. The stamps need updating together with whatever bump the next release applies.