---
id: create-agent-helm-chart-foundation
level: task
title: "Create agent Helm chart foundation"
short_code: "BROKKR-T-0007"
created_at: 2025-10-18T14:47:36.496296+00:00
updated_at: 2025-10-18T14:47:36.496296+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Create agent Helm chart foundation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create foundational Helm chart for agent with deployment, service account, RBAC, and broker connection configuration for Phase 1 validation.

## Acceptance Criteria **[REQUIRED]**

- [ ] Chart.yaml created with proper metadata
- [ ] deployment.yaml template with container spec, health probes, security context (runAsUser: 10001)
- [ ] serviceaccount.yaml for agent identity
- [ ] RBAC templates (ClusterRole, ClusterRoleBinding) with minimal required permissions
- [ ] configmap.yaml for agent configuration (broker URL, polling intervals)
- [ ] values.yaml with broker connection settings and agent metadata
- [ ] Chart installs successfully and agent connects to broker
- [ ] Agent can access Kubernetes API with configured RBAC

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Chart Structure** (charts/brokkr-agent/):
```
Chart.yaml          # Chart metadata
values.yaml         # Default configuration
templates/
  deployment.yaml   # Agent deployment
  serviceaccount.yaml
  rbac.yaml         # ClusterRole + ClusterRoleBinding
  configmap.yaml    # Agent config
  _helpers.tpl      # Template helpers
```

**Key Configuration:**

1. **deployment.yaml essentials**:
   - Image: `ghcr.io/colliery-io/brokkr-agent:{{ .Values.image.tag }}`
   - Security context: runAsNonRoot: true, runAsUser: 10001, fsGroup: 10001
   - Health probes: livenessProbe (/healthz), readinessProbe (/readyz) - requires BROKKR-T-0003
   - ServiceAccount: brokkr-agent
   - Environment from ConfigMap

2. **RBAC (rbac.yaml)**:
   - ClusterRole with minimal permissions:
     - pods: get, list, watch
     - namespaces: get, list
     - deployments/statefulsets/daemonsets: get, list, watch
     - (Phase 2 will expand based on reconciliation needs)
   - ClusterRoleBinding: serviceaccount → clusterrole

3. **values.yaml structure**:
   ```yaml
   image:
     repository: ghcr.io/colliery-io/brokkr-agent
     tag: latest

   broker:
     url: http://brokkr-broker:3000
     agentName: ""          # Auto-generated if empty
     clusterName: ""        # Required
     pak: ""                # From secret

   agent:
     pollingInterval: 30s

   rbac:
     create: true
     clusterWide: true      # vs namespace-scoped
   ```

**Files to Create:**
- `charts/brokkr-agent/Chart.yaml`
- `charts/brokkr-agent/values.yaml`
- `charts/brokkr-agent/templates/deployment.yaml`
- `charts/brokkr-agent/templates/serviceaccount.yaml`
- `charts/brokkr-agent/templates/rbac.yaml`
- `charts/brokkr-agent/templates/configmap.yaml`
- `charts/brokkr-agent/templates/_helpers.tpl`

### Dependencies

- Depends on BROKKR-T-0001 (non-root Dockerfiles) for security context
- Depends on BROKKR-T-0003 (agent health endpoints) for probe configuration
- Depends on BROKKR-T-0006 (broker chart) for broker URL configuration
- Enables BROKKR-T-0008 (Phase 1 validation)

### Risk Considerations

**Risk: RBAC permissions too permissive**
- Mitigation: Start minimal, expand only as needed
- Document why each permission is required

**Risk: PAK secret management**
- Mitigation: Document manual secret creation for Phase 1
- Phase 2: Add secret generation in broker chart

**Risk: Agent unable to reach broker (network policies)**
- Mitigation: Test in BROKKR-T-0008 with actual cluster
- Document network requirements

## Status Updates **[REQUIRED]**

*To be added during implementation*
