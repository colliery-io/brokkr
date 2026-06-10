---
id: namespace-scoped-rbac-chart-option
level: task
title: "Namespace-scoped RBAC chart option is non-functional — fix or remove"
short_code: "BROKKR-T-0192"
created_at: 2026-06-10T03:04:18.981640+00:00
updated_at: 2026-06-10T04:58:27.318645+00:00
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

# Namespace-scoped RBAC chart option is non-functional — fix or remove

## Objective

CORRECTED 2026-06-09 (verified against code; the original claim from `charts/brokkr-agent/RBAC.md:273` — "agent fails during startup" — is STALE/WRONG): namespace-scoped agents DO run. Startup only calls `apiserver_version()` (no RBAC needed, `k8s/api.rs:910`). What actually happens under a namespaced Role: the pod-log and kube-event watchers use cluster-wide `Api::all` and crash-loop with a warn every 5s (`pod_logs.rs:62-74`, `kube_events.rs:151`) so telemetry never flows; pruning lists per-resource and warn-skips 403s (`k8s/api.rs:344`) so cross-namespace cleanup silently degrades; deployment health lists cluster-wide and reports `unknown` on error; applying cluster-scoped resources (Namespace, CRD) in a stack fails. In-namespace deploys work.

Work: scope the watchers/queries to the agent.s namespace when running namespace-scoped (e.g. downward-API `POD_NAMESPACE`), make pruning honor the scope, and fix the stale claim in `charts/brokkr-agent/RBAC.md`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Namespace-scoped mode works end to end, OR the chart value is removed/fails validation with a clear message
- [ ] `charts/brokkr-agent/RBAC.md` and the book (`installation.md`, `security-model.md`, `network-flows.md` warnings added 2026-06-09) updated to match

## Status Updates

- 2026-06-09: Surfaced as the completeness review's sole blocker during the /docs-diataxis run.
- 2026-06-10 (closure pass): namespace-scoped operation IMPLEMENTED — new `agent.watch_namespace` setting (BROKKR__AGENT__WATCH_NAMESPACE); pod-log tailer, kube-event tailer, and health discovery scope to it when set; agent chart sets it via downward API when rbac.clusterWide=false (helm template verified both modes). Remaining by design: pruning warn-skips unlistable resource types; cluster-scoped manifests fail. RBAC.md + book pages updated.