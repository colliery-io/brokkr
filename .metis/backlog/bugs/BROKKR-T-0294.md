---
id: chart-readmes-agent-readme-claims
level: task
title: "Chart READMEs: agent README claims k8s 1.19+ vs pinned >=1.29, undisclosed cluster-wide Tekton/Shipwright install, values tables omit most of values.yaml"
short_code: "BROKKR-T-0294"
created_at: 2026-07-27T14:28:01.308585+00:00
updated_at: 2026-07-27T14:28:01.308585+00:00
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

- [ ] Agent README requirement matches Chart.yaml kubeVersion (state why: Shipwright CRD baseline).
- [ ] Default-install side effects (Tekton/Shipwright cluster-wide, versions, opt-out values) documented prominently near the install commands.
- [ ] Values tables cover every values.yaml key or link a generated full reference; consider helm-docs to keep them honest.
- [ ] Cross-checked against docs/src install pages so the two agree (see BROKKR-T-0278, BROKKR-T-0286).

## Status Updates

*To be added during implementation*
