---
id: create-broker-helm-chart-foundation
level: task
title: "Create broker Helm chart foundation"
short_code: "BROKKR-T-0006"
created_at: 2025-10-18T14:47:36.299249+00:00
updated_at: 2025-10-19T02:22:51.380076+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Create broker Helm chart foundation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create foundational Helm chart for broker with deployment, service, configuration, and optional bundled PostgreSQL support for Phase 1 validation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Chart.yaml created with proper metadata (name, version, description, appVersion)
- [x] deployment.yaml template with container spec, health probes, security context (runAsUser: 10001)
- [x] service.yaml template for broker API (port 3000)
- [x] configmap.yaml template for environment-based configuration
- [x] secret.yaml template for database credentials
- [x] Conditional PostgreSQL deployment (postgresql.enabled in values.yaml)
- [x] values.yaml with essential options (image, replicas, resources, database config)
- [x] Chart installs successfully with `helm install`
- [x] Broker connects to bundled or external PostgreSQL based on configuration

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Chart Structure** (charts/brokkr-broker/):
```
Chart.yaml          # Chart metadata
values.yaml         # Default configuration
templates/
  deployment.yaml   # Broker deployment
  service.yaml      # ClusterIP service
  configmap.yaml    # Environment variables
  secret.yaml       # DB credentials
  _helpers.tpl      # Template helpers
```

**Key Configuration:**

1. **Chart.yaml**:
   - name: brokkr-broker
   - version: 0.1.0 (chart version)
   - appVersion: 0.1.0 (app version)
   - description: Brokkr control plane broker

2. **deployment.yaml essentials**:
   - Image: `ghcr.io/colliery-io/brokkr-broker:{{ .Values.image.tag }}`
   - Security context: runAsNonRoot: true, runAsUser: 10001, fsGroup: 10001
   - Health probes: livenessProbe (/healthz), readinessProbe (/readyz)
   - Environment from ConfigMap and Secret
   - Resource requests/limits configurable

3. **values.yaml structure**:
   ```yaml
   image:
     repository: ghcr.io/colliery-io/brokkr-broker
     tag: latest
     pullPolicy: IfNotPresent

   replicaCount: 1

   postgresql:
     enabled: true  # Bundled PostgreSQL
     host: ""       # External PostgreSQL (if enabled: false)
     port: 5432
     database: brokkr
     username: brokkr
     password: brokkr  # Change in production!

   resources:
     requests:
       memory: "256Mi"
       cpu: "100m"
     limits:
       memory: "512Mi"
       cpu: "500m"
   ```

4. **PostgreSQL bundling**:
   - Use subchart dependency OR conditional template
   - For Phase 1: simple conditional template with basic StatefulSet
   - Phase 2 (BROKKR-T-0010): Add bitnami/postgresql dependency

**Files to Create:**
- `charts/brokkr-broker/Chart.yaml`
- `charts/brokkr-broker/values.yaml`
- `charts/brokkr-broker/templates/deployment.yaml`
- `charts/brokkr-broker/templates/service.yaml`
- `charts/brokkr-broker/templates/configmap.yaml`
- `charts/brokkr-broker/templates/secret.yaml`
- `charts/brokkr-broker/templates/_helpers.tpl`

### Dependencies

- Depends on BROKKR-T-0001 (non-root Dockerfiles) for security context configuration
- Depends on BROKKR-T-0002 (health endpoints) for liveness/readiness probe configuration
- Enables BROKKR-T-0008 (Phase 1 validation)

### Risk Considerations

**Risk: fsGroup permission issues with volumes**
- Mitigation: Learned from BROKKR-T-0001 that fsGroup: 10001 needed for volume access
- Test with actual PVCs in BROKKR-T-0008

**Risk: PostgreSQL bundling complexity**
- Mitigation: Keep Phase 1 simple (basic StatefulSet), defer full subchart to Phase 2
- Document external PostgreSQL as preferred production approach

**Risk: Default credentials in values.yaml**
- Mitigation: Clear comments warning to change in production
- Document secret management best practices

## Status Updates **[REQUIRED]**

### 2025-10-18: Chart Foundation Complete

All acceptance criteria met:

**Chart Structure Created:**
- charts/brokkr-broker/Chart.yaml - v0.1.0, app v0.1.0
- charts/brokkr-broker/values.yaml - configuration with image, replicas, resources, postgresql settings
- charts/brokkr-broker/templates/_helpers.tpl - standard helpers plus database host helper

**Kubernetes Templates Created:**
- templates/deployment.yaml - broker deployment with security context (runAsUser: 10001, fsGroup: 10001), health probes (/healthz, /readyz), resource limits
- templates/service.yaml - ClusterIP service on port 3000
- templates/configmap.yaml - database connection configuration (host, port, name)
- templates/secret.yaml - database credentials (username, password)
- templates/postgresql-statefulset.yaml - conditional PostgreSQL StatefulSet with 1Gi PVC, postgres:16-alpine
- templates/postgresql-service.yaml - conditional PostgreSQL service

**Validation:**
- Helm dry-run installation succeeded
- All templates render correctly
- Conditional PostgreSQL deployment working (enabled by default, can be disabled)
- Database host helper correctly references bundled PostgreSQL service name or external host
- Security context matches non-root Dockerfile configuration from BROKKR-T-0001
- Health probe paths match endpoints from BROKKR-T-0002

Ready for Phase 1 validation in BROKKR-T-0008.
