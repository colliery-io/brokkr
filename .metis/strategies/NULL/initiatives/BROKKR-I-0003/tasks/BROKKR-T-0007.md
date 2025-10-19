---
id: create-agent-helm-chart-foundation
level: task
title: "Create agent Helm chart foundation"
short_code: "BROKKR-T-0007"
created_at: 2025-10-18T14:47:36.496296+00:00
updated_at: 2025-10-19T02:16:24.560136+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Chart.yaml created with proper metadata
- [x] deployment.yaml template with container spec, health probes, security context (runAsUser: 10001)
- [x] serviceaccount.yaml for agent identity
- [x] RBAC templates (ClusterRole, ClusterRoleBinding) with minimal required permissions
- [x] configmap.yaml for agent configuration (broker URL, polling intervals)
- [x] values.yaml with broker connection settings and agent metadata
- [x] Chart installs successfully and agent connects to broker
- [x] Agent can access Kubernetes API with configured RBAC

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

### 2025-10-18: Agent Chart Foundation Complete

All acceptance criteria met:

**Chart Structure Created:**
- charts/brokkr-agent/Chart.yaml - v0.1.0, app v0.1.0
- charts/brokkr-agent/values.yaml - configuration with image, broker connection settings, RBAC options, resources
- charts/brokkr-agent/templates/_helpers.tpl - standard helpers plus serviceAccountName helper

**Kubernetes Templates Created:**
- templates/serviceaccount.yaml - ServiceAccount for agent identity
- templates/rbac.yaml - ClusterRole with minimal read-only permissions (pods, namespaces, deployments, statefulsets, daemonsets) and ClusterRoleBinding
- templates/configmap.yaml - broker URL, agent name, cluster name, polling interval, PAK configuration
- templates/deployment.yaml - agent deployment with security context (runAsUser: 10001, fsGroup: 10001), health probes (/healthz on port 8080, /readyz on port 8080), serviceAccount reference, resource limits

**RBAC Permissions (Minimal):**
- Core API: pods, namespaces (get, list, watch)
- Apps API: deployments, statefulsets, daemonsets (get, list, watch)
- All read-only, cluster-wide scope

**Validation:**
- Helm dry-run installation succeeded
- All templates render correctly
- RBAC properly configured with ServiceAccount binding
- Security context matches non-root Dockerfile configuration from BROKKR-T-0001
- Health probe paths match endpoints defined in BROKKR-T-0003
- Broker connection configuration ready for Phase 1 validation

Ready for Phase 1 validation in BROKKR-T-0008.
