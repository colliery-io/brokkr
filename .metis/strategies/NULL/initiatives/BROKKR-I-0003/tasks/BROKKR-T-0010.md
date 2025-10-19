---
id: implement-postgresql-bundling
level: task
title: "Implement PostgreSQL bundling option"
short_code: "BROKKR-T-0010"
created_at: 2025-10-19T02:26:48.964233+00:00
updated_at: 2025-10-19T02:26:48.964233+00:00
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

# Implement PostgreSQL bundling option

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Replace the basic PostgreSQL StatefulSet from Phase 1 with a production-ready PostgreSQL subchart dependency, adding persistence options, backup configuration, and proper lifecycle management for bundled database deployments.

## Acceptance Criteria **[REQUIRED]**

- [ ] Remove basic PostgreSQL StatefulSet templates from Phase 1
- [ ] Add bitnami/postgresql as chart dependency in Chart.yaml
- [ ] Configure conditional deployment based on postgresql.enabled value
- [ ] Add persistence configuration (PVC, storage class, size options)
- [ ] Configure connection strings for both bundled and external scenarios
- [ ] Test broker deployment with bundled PostgreSQL
- [ ] Test broker deployment with external PostgreSQL
- [ ] Verify data persistence across pod restarts

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Add Subchart Dependency:**
Update Chart.yaml:
```yaml
dependencies:
  - name: postgresql
    version: "~12.0.0"
    repository: "https://charts.bitnami.com/bitnami"
    condition: postgresql.enabled
```

**Configure PostgreSQL Values:**
In values.yaml, configure the subchart:
```yaml
postgresql:
  enabled: true  # Set to false for external DB
  auth:
    username: brokkr
    password: brokkr  # Should be overridden in production
    database: brokkr
  primary:
    persistence:
      enabled: true
      storageClass: ""  # Use default storage class
      size: 8Gi
    resources:
      requests:
        memory: "256Mi"
        cpu: "250m"
      limits:
        memory: "512Mi"
        cpu: "500m"
```

**Update Database Host Helper:**
Modify _helpers.tpl to use subchart service name:
```yaml
{{- define "brokkr-broker.databaseHost" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" .Release.Name }}
{{- else }}
{{- .Values.postgresql.externalHost }}
{{- end }}
{{- end }}
```

**Remove Phase 1 Templates:**
- Delete charts/brokkr-broker/templates/postgresql-statefulset.yaml
- Delete charts/brokkr-broker/templates/postgresql-service.yaml

**Test Scenarios:**
1. Bundled PostgreSQL with persistence
2. Bundled PostgreSQL without persistence (ephemeral)
3. External PostgreSQL (postgresql.enabled=false)

### Dependencies

- Depends on BROKKR-T-0006 (broker chart foundation) - completed
- Depends on BROKKR-T-0009 (comprehensive configuration) - needs external DB config
- Requires helm dependency update command after Chart.yaml changes

### Risk Considerations

**Risk: Data loss with default persistence settings**
- Mitigation: Enable persistence by default
- Document backup and recovery procedures
- Warn about ephemeral mode in values.yaml comments

**Risk: Storage class not available in target cluster**
- Mitigation: Use empty string for default storage class
- Document how to specify custom storage classes
- Test with multiple storage providers

**Risk: Resource limits too low for production workloads**
- Mitigation: Provide conservative defaults that work for development
- Document recommended production resource settings
- Include example production values file

**Risk: Bitnami chart version changes breaking compatibility**
- Mitigation: Pin to major version range (~12.0.0)
- Test upgrades before updating dependency version
- Document tested PostgreSQL versions

## Status Updates **[REQUIRED]**

*To be added during implementation*
