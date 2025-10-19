---
id: create-comprehensive-rbac-for-agent
level: task
title: "Create comprehensive RBAC for agent"
short_code: "BROKKR-T-0012"
created_at: 2025-10-19T02:26:49.169147+00:00
updated_at: 2025-10-19T02:26:49.169147+00:00
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

# Create comprehensive RBAC for agent

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Define and document comprehensive RBAC permissions for the agent's control loop to access the Kubernetes API, including minimal required permissions for cluster observation and future reconciliation capabilities, with configurable cluster-wide vs namespace-scoped options.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria **[REQUIRED]**

- [ ] Detailed ClusterRole with specific API groups and resources documented
- [ ] Justification for each permission requirement documented
- [ ] Configurable RBAC scope (cluster-wide vs namespace-scoped)
- [ ] Role template for namespace-scoped deployments
- [ ] Support for custom additional permissions via values.yaml
- [ ] Documentation explaining why each permission is needed
- [ ] Test agent can successfully read cluster resources
- [ ] Test agent fails gracefully without required permissions

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**:
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Agent Control Loop Permissions:**
The agent is a control loop that:
1. Polls broker for desired state instructions
2. Reads Kubernetes API to gather current cluster state
3. Reports state back to broker
4. (Future) Executes reconciliation actions

**Phase 2 RBAC Strategy (Read-Only):**
Current focus: observation and reporting (no write operations yet)

**Enhanced ClusterRole Definition (templates/rbac.yaml):**
```yaml
{{- if .Values.rbac.create -}}
apiVersion: rbac.authorization.k8s.io/v1
{{- if .Values.rbac.clusterWide }}
kind: ClusterRole
{{- else }}
kind: Role
{{- end }}
metadata:
  name: {{ include "brokkr-agent.fullname" . }}
  labels:
    {{- include "brokkr-agent.labels" . | nindent 4 }}
rules:
# Core API - cluster inventory
- apiGroups: [""]
  resources:
  - pods
  - pods/log         # For log collection/analysis
  - pods/status      # For health monitoring
  - namespaces
  - nodes            # For cluster topology
  - services
  - endpoints
  - configmaps       # For configuration discovery
  - secrets          # Only metadata, not values
  - persistentvolumes
  - persistentvolumeclaims
  verbs:
  - get
  - list
  - watch

# Apps API - workload inventory
- apiGroups: ["apps"]
  resources:
  - deployments
  - deployments/status
  - statefulsets
  - statefulsets/status
  - daemonsets
  - daemonsets/status
  - replicasets
  - replicasets/status
  verbs:
  - get
  - list
  - watch

# Batch API - job inventory
- apiGroups: ["batch"]
  resources:
  - jobs
  - jobs/status
  - cronjobs
  - cronjobs/status
  verbs:
  - get
  - list
  - watch

# Networking API - network policy inventory
- apiGroups: ["networking.k8s.io"]
  resources:
  - ingresses
  - ingresses/status
  - networkpolicies
  verbs:
  - get
  - list
  - watch

# RBAC API - security posture assessment
- apiGroups: ["rbac.authorization.k8s.io"]
  resources:
  - roles
  - rolebindings
  - clusterroles
  - clusterrolebindings
  verbs:
  - get
  - list
  - watch

# Events - change tracking and debugging
- apiGroups: [""]
  resources:
  - events
  verbs:
  - get
  - list
  - watch

{{- with .Values.rbac.additionalRules }}
{{ toYaml . }}
{{- end }}
{{- end }}
```

**Configuration in values.yaml:**
```yaml
rbac:
  create: true
  clusterWide: true  # false for namespace-scoped Role
  # Additional custom rules
  additionalRules: []
    # - apiGroups: ["custom.io"]
    #   resources: ["customresources"]
    #   verbs: ["get", "list"]
```

**Permission Justification Documentation:**
Create docs/explanation/rbac-permissions.md explaining:
- Why each API group/resource is needed
- What data the agent collects from each resource
- Security implications of each permission
- Difference between cluster-wide and namespace-scoped modes

**Future Phase 3+ Write Permissions:**
(Document but don't implement in Phase 2)
- `create`, `update`, `patch`, `delete` for reconciliation
- Specific resources based on reconciliation capabilities
- More granular resource name restrictions

### Dependencies

- Depends on BROKKR-T-0007 (agent chart foundation) - completed with basic RBAC
- Expands on Phase 1 minimal permissions
- Enables future reconciliation capabilities in Phase 3+

### Risk Considerations

**Risk: Overly permissive RBAC defeating security**
- Mitigation: Follow principle of least privilege
- Document and justify each permission
- Provide namespace-scoped option for restricted environments
- Regular RBAC audit and review process

**Risk: Missing permissions causing agent failures**
- Mitigation: Comprehensive testing in different cluster configurations
- Clear error messages when permissions are missing
- Documentation for troubleshooting RBAC issues
- Validation script to check required permissions

**Risk: Cluster-wide RBAC rejected in multi-tenant environments**
- Mitigation: Support namespace-scoped mode
- Document trade-offs between cluster-wide and namespace-scoped
- Provide separate values files for different security profiles

**Risk: Future write permissions creating security concerns**
- Mitigation: Keep read and write permissions separate
- Make write permissions opt-in with explicit configuration
- Provide audit logging for all write operations
- Document security model and trust boundaries

## Status Updates **[REQUIRED]**

*To be added during implementation*
