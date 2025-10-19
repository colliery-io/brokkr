---
id: add-security-contexts-to-all-pod
level: task
title: "Add security contexts to all pod specs"
short_code: "BROKKR-T-0013"
created_at: 2025-10-19T02:26:49.406890+00:00
updated_at: 2025-10-19T02:26:49.406890+00:00
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

# Add security contexts to all pod specs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Enhance Helm chart security by adding comprehensive pod and container security contexts to all templates, including non-root enforcement, read-only filesystems, capability dropping, and seccomp profiles to meet security best practices and restricted PSS standards.

## Acceptance Criteria **[REQUIRED]**

- [ ] Pod-level security contexts added to all deployment templates
- [ ] Container-level security contexts with minimal capabilities
- [ ] Read-only root filesystem where possible
- [ ] All Linux capabilities dropped, only required ones added back
- [ ] Seccomp profile set to RuntimeDefault
- [ ] AppArmor annotations where applicable
- [ ] Security contexts configurable via values.yaml
- [ ] Charts deploy successfully in restricted Kubernetes clusters (PSS restricted)
- [ ] Documentation explaining each security setting

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Pod Security Context (Pod Level):**
Apply to both broker and agent deployments:
```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
  seccompProfile:
    type: RuntimeDefault
```

**Container Security Context (Container Level):**
```yaml
securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 10001
  capabilities:
    drop:
      - ALL
    add: []  # Only add specific capabilities if absolutely required
```

**Broker Deployment Template Updates:**
```yaml
spec:
  template:
    metadata:
      annotations:
        container.apparmor.security.beta.kubernetes.io/broker: runtime/default
    spec:
      securityContext:
        {{- toYaml .Values.podSecurityContext | nindent 8 }}
      containers:
      - name: broker
        securityContext:
          {{- toYaml .Values.containerSecurityContext | nindent 10 }}
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: cache
          mountPath: /var/cache
      volumes:
      - name: tmp
        emptyDir: {}
      - name: cache
        emptyDir: {}
```

**Agent Deployment Template Updates:**
Same pattern as broker.

**PostgreSQL Security Context:**
Update bundled PostgreSQL subchart values:
```yaml
postgresql:
  primary:
    podSecurityContext:
      runAsNonRoot: true
      runAsUser: 1001
      fsGroup: 1001
      seccompProfile:
        type: RuntimeDefault
    containerSecurityContext:
      allowPrivilegeEscalation: false
      runAsNonRoot: true
      runAsUser: 1001
      capabilities:
        drop:
          - ALL
```

**Configuration in values.yaml:**
```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
  seccompProfile:
    type: RuntimeDefault

containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 10001
  capabilities:
    drop:
      - ALL
```

**Handling Read-Only Root Filesystem:**
Applications need writable directories:
- Mount emptyDir volumes for /tmp, /var/cache, etc.
- Update application to write only to these volumes
- Document required writable paths

**Files to Modify:**
- charts/brokkr-broker/values.yaml - add security context configuration
- charts/brokkr-broker/templates/deployment.yaml - apply security contexts, add tmp volumes
- charts/brokkr-agent/values.yaml - add security context configuration
- charts/brokkr-agent/templates/deployment.yaml - apply security contexts, add tmp volumes
- charts/brokkr-broker/templates/postgresql-*.yaml - remove (replaced by subchart in T-0010)

### Dependencies

- Depends on BROKKR-T-0001 (non-root Dockerfiles) - completed
- Depends on BROKKR-T-0006 (broker chart) - completed
- Depends on BROKKR-T-0007 (agent chart) - completed
- Depends on BROKKR-T-0010 (PostgreSQL subchart) - needs coordinated security context

### Risk Considerations

**Risk: Read-only filesystem breaking application functionality**
- Mitigation: Identify all writable paths needed by applications
- Mount emptyDir volumes for required writable locations
- Test thoroughly with read-only filesystem enabled
- Document any paths that require write access

**Risk: Capability restrictions preventing required operations**
- Mitigation: Start with all capabilities dropped
- Add back only specific capabilities if needed (document why)
- Test all functionality with restricted capabilities
- Provide escape hatch via values.yaml for special cases

**Risk: PSS restricted mode rejection in some clusters**
- Mitigation: Make security contexts configurable
- Test in actual restricted clusters
- Document minimum required security settings
- Provide different security profiles for different environments

**Risk: Breaking compatibility with existing deployments**
- Mitigation: Phase in security contexts with deprecation warnings
- Provide migration guide for existing deployments
- Test upgrade path from Phase 1 charts
- Document any behavioral changes

## Status Updates **[REQUIRED]**

*To be added during implementation*
