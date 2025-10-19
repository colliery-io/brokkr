---
id: add-comprehensive-configuration-to
level: task
title: "Add comprehensive configuration to broker chart"
short_code: "BROKKR-T-0009"
created_at: 2025-10-19T02:26:48.872074+00:00
updated_at: 2025-10-19T11:51:14.002625+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Add comprehensive configuration to broker chart

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **\[CONDITIONAL: Assigned Task\]**

\[\[BROKKR-I-0003\]\]

## Objective **\[REQUIRED\]**

Extend the broker Helm chart with comprehensive configuration options for external PostgreSQL, resources, environment variables, secrets, and scaling to support production deployments.

## Acceptance Criteria

## Acceptance Criteria **\[REQUIRED\]**

- \[ \] External PostgreSQL connection configuration (host, port, database, username)
- \[ \] Resource requests and limits configurable via values.yaml
- \[ \] Environment variable management with templating support
- \[ \] Secrets management for database credentials with external secret support
- \[ \] Replica count configuration for horizontal scaling
- \[ \] Service type configuration (ClusterIP, LoadBalancer, NodePort)
- \[ \] All configuration options documented in values.yaml with inline comments
- \[ \] Chart installs successfully with both bundled and external PostgreSQL

## Implementation Notes **\[CONDITIONAL: Technical Task\]**

### Technical Approach

**External PostgreSQL Configuration**:Extend values.yaml to support external database:

```yaml
postgresql:
  enabled: true  # false for external DB
  host: ""       # external DB host when enabled=false
  port: 5432
  database: brokkr
  username: brokkr
  password: ""   # can be empty if using existingSecret
  existingSecret: ""  # name of k8s secret with credentials
  existingSecretPasswordKey: "password"
```

**Resource Configuration**:Add configurable resources with sensible defaults:

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

**Environment Variables**:Support additional env vars via values:

```yaml
extraEnv:
  - name: LOG_LEVEL
    value: "info"
  - name: CUSTOM_VAR
    value: "custom-value"
```

**Scaling Configuration:**

```yaml
replicaCount: 1
autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
```

**Service Configuration:**

```yaml
service:
  type: ClusterIP
  port: 3000
  annotations: {}
```

**Files to Modify:**

- charts/brokkr-broker/values.yaml - add all new configuration options
- charts/brokkr-broker/templates/deployment.yaml - use resource values, extraEnv
- charts/brokkr-broker/templates/secret.yaml - support existingSecret
- charts/brokkr-broker/templates/service.yaml - support service type configuration
- charts/brokkr-broker/templates/hpa.yaml - new file for autoscaling (if enabled)

### Dependencies

- Depends on BROKKR-T-0006 (broker chart foundation) - completed
- Enables BROKKR-T-0010 (PostgreSQL bundling) - needs external DB config
- Enables BROKKR-T-0014 (values files for scenarios) - needs base configuration

### Risk Considerations

**Risk: Configuration complexity overwhelming users**

- Mitigation: Provide sensible defaults that work out-of-box
- Document common configuration patterns
- Use inline comments in values.yaml

**Risk: Secret management exposing credentials**

- Mitigation: Support external secret providers (existingSecret)
- Document best practices for credential management
- Never set default passwords in production examples

## Status Updates **\[REQUIRED\]**

*To be added during implementation*
