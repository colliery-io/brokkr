---
id: chart-readmes-agent-readme-claims
level: task
title: "Chart READMEs: agent README claims k8s 1.19+ vs pinned >=1.29, undisclosed cluster-wide Tekton/Shipwright install, values tables omit most of values.yaml"
short_code: "BROKKR-T-0294"
created_at: 2026-07-27T14:28:01.308585+00:00
updated_at: 2026-07-28T15:14:20.373084+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Chart READMEs: agent README claims k8s 1.19+ vs pinned >=1.29, undisclosed cluster-wide Tekton/Shipwright install, values tables omit most of values.yaml

## Objective

Fix the chart READMEs — the first install surface a Helm consumer reads (2026-07-27 review; `docs/REVIEW-2026-07-27.md`):

1. **Blocker — false compatibility claim**: `charts/brokkr-agent/README.md` says "Kubernetes 1.19+", but Chart.yaml pins `kubeVersion ">=1.29.0-0"` chart-wide — helm refuses to install below 1.29 regardless of Shipwright use. Users on 1.19–1.28 plan an install that cannot proceed.
2. **Major — undisclosed cluster-wide side effects**: a default agent install (`shipwright.enabled: true` + `install.tekton/shipwright: true`) installs Tekton Pipelines v0.68.1 and Shipwright Build v0.18.1 **cluster-wide** via pre-install hook Jobs. Never mentioned — a tenant installing "just an agent" changes cluster-global state, possibly colliding with existing Tekton installs. This deserves a loud callout for the multi-tenant audience.
3. **Major — values coverage**: broker README's table omits most of values.yaml (configReload, autoscaling, extraEnv, broker.logLevel/diagnostic*/webhookDelivery*, cors.*, metrics.*, networkPolicy.*, podDisruptionBudget, telemetry.*, apparmor, postgresql.auth/primary, postgresql.existingSecretKey, image.pullSecrets, service.annotations); agent README similarly omits shipwright.*, agent.wsUrl, agent.deploymentHealth, rbac.secretAccess, metrics, networkPolicy, telemetry, hostAliases.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Agent README requirement matches Chart.yaml kubeVersion (state why: Shipwright CRD baseline).
- [ ] Default-install side effects (Tekton/Shipwright cluster-wide, versions, opt-out values) documented prominently near the install commands.
- [ ] Values tables cover every values.yaml key or link a generated full reference; consider helm-docs to keep them honest.
- [ ] Cross-checked against docs/src install pages so the two agree (see BROKKR-T-0278, BROKKR-T-0286).

## Status Updates

**2026-07-28 — FIXED** on branch `docs/tenancy-review-2026-07`. `helm lint` passes for both charts.

**kubeVersion rationale confirmed, not assumed:** `git log` shows the `>=1.29.0-0` pin was added in the Shipwright integration commit alongside an inline `Chart.yaml` comment saying Shipwright requires ≥1.29, and `how-to/shipwright-builds.md` states the same floor independently. Documented as "the baseline the Shipwright integration requires, per Chart.yaml and the how-to" rather than asserting an upstream requirement not verifiable from the repo. Enforcement verified: `helm template --kube-version 1.28.0` fails, `1.29.0` renders. The agent README now also warns the pin is chart-wide and unconditional — `shipwright.enabled=false` does not lower it.

**Cluster-wide side effects documented with verified specifics:** Tekton Pipelines `v0.68.1` and Shipwright Build `v0.18.1`, installed by a `pre-install,pre-upgrade` hook Job bound to **`cluster-admin`**. New section placed before the install commands covering collision risk, re-run on upgrade, egress requirement, survival past uninstall, and that `rbac.clusterWide` does not help — plus an opt-out subsection with a `helm template | grep -c cluster-admin` verification. The upstream manifests were fetched to verify the object inventory quoted.

**Tables: complete, not partial.** Both values files are ~60–75 leaf keys — small enough to enumerate exhaustively, so the partial-with-pointer compromise would have bought nothing but omissions. Decisive factor: the missing keys were disproportionately the security-relevant ones (`rbac.secretAccess.readContents`, `cors.allowedOrigins: ["*"]`, `postgresql.auth.password: brokkr`, `networkPolicy.*`, `apparmor`, both security-context objects), so a scoped table would have had to include most of them anyway. Drift mitigated the way a generator would not: each table is stamped with the chart version it was written against, names `values.yaml` authoritative, and gives the `helm show values` diff command.

**Two stale claims found that the ticket did not mention, both fixed:** the broker table still advertised `tls.certManager.issuer`/`issuerKind` rows for values deleted in the TLS work — a live contradiction with the TLS section three screens above — and the agent README's uninstall section claimed it "removes all resources created by the chart", which is false given hook-installed Tekton/Shipwright survive uninstall.

**`service.annotations` is not a real broker value** — it appears only as a commented example; `templates/service.yaml` renders no annotations, so setting it is a no-op. Documented as having no effect rather than listed as supported, which would have been a fresh false claim.

**Six defects found outside this ticket's scope.** Four are the same class — chart values that silently do nothing — filed as **BROKKR-T-0308**: the agent ServiceMonitor selects a port on a Service the chart never renders; `telemetry.collector.enabled` repoints the OTLP endpoint at a sidecar neither Deployment creates; `metrics.enabled` only gates a NetworkPolicy rule despite its name; and `service.annotations` above. A possible RBAC gap in broker config-reload is filed separately as **BROKKR-T-0309**. The two documentation issues — `shipwright-builds.md` calling Tekton/Shipwright "vendored dependencies" when they are fetched at install time by a Job, and `installation.md` showing the default agent install with no mention of the cluster-wide side effect — are carried into BROKKR-T-0295's sweep.