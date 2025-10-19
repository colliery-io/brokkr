---
id: create-values-files-for-deployment
level: task
title: "Create values files for deployment scenarios"
short_code: "BROKKR-T-0014"
created_at: 2025-10-19T02:26:49.681872+00:00
updated_at: 2025-10-19T02:26:49.681872+00:00
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

# Create values files for deployment scenarios

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create pre-configured values files for common deployment scenarios (production, development, staging) with appropriate settings for resources, security, persistence, and observability to simplify deployment for different environments.

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

- [ ] values/production.yaml created for broker and agent with production-ready settings
- [ ] values/development.yaml created for broker and agent with dev-friendly settings
- [ ] values/staging.yaml created for broker and agent as middle ground
- [ ] All files thoroughly commented explaining each setting
- [ ] Documentation explaining when to use each values file
- [ ] Each scenario tested with successful deployment
- [ ] README in values/ directory explaining the files

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

**File Structure:**
```
charts/brokkr-broker/values/
  production.yaml
  development.yaml
  staging.yaml
  README.md

charts/brokkr-agent/values/
  production.yaml
  development.yaml
  staging.yaml
  README.md
```

**Broker Production Values (values/production.yaml):**
```yaml
# Production configuration for Brokkr Broker
# Use with: helm install -f values/production.yaml

replicaCount: 3  # High availability

image:
  tag: "v1.0.0"  # Pin to specific version, not latest
  pullPolicy: IfNotPresent

resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"

postgresql:
  enabled: false  # Use managed database in production
  host: "postgres.example.com"
  port: 5432
  database: "brokkr_prod"
  username: "brokkr"
  existingSecret: "brokkr-db-credentials"  # Never use inline passwords
  existingSecretPasswordKey: "password"

tls:
  enabled: true
  certManager:
    enabled: true
    issuer: "letsencrypt-prod"

ingress:
  enabled: true
  className: "nginx"
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
  hosts:
    - host: brokkr.example.com
      paths:
        - path: /
          pathType: Prefix

podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  fsGroup: 10001
  seccompProfile:
    type: RuntimeDefault

containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  capabilities:
    drop:
      - ALL
```

**Broker Development Values (values/development.yaml):**
```yaml
# Development configuration for Brokkr Broker
# Use with: helm install -f values/development.yaml

replicaCount: 1  # Single instance for dev

image:
  tag: "latest"  # Use latest for dev
  pullPolicy: Always

resources:
  requests:
    memory: "128Mi"
    cpu: "50m"
  limits:
    memory: "256Mi"
    cpu: "200m"

postgresql:
  enabled: true  # Bundled PostgreSQL for convenience
  auth:
    password: "devpassword"  # Simple password for dev
  primary:
    persistence:
      enabled: false  # Ephemeral storage for dev

tls:
  enabled: false  # No TLS in dev

ingress:
  enabled: false  # Use port-forward in dev

# Relaxed security for development ease
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  fsGroup: 10001

containerSecurityContext:
  allowPrivilegeEscalation: false
  runAsNonRoot: true
```

**Broker Staging Values (values/staging.yaml):**
```yaml
# Staging configuration for Brokkr Broker
# Use with: helm install -f values/staging.yaml

replicaCount: 2  # Some redundancy

image:
  tag: "v1.0.0-rc1"  # Release candidates
  pullPolicy: IfNotPresent

resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "512Mi"
    cpu: "500m"

postgresql:
  enabled: false  # External DB but not managed
  host: "postgres-staging"
  database: "brokkr_staging"
  existingSecret: "brokkr-db-staging"

tls:
  enabled: true
  existingSecret: "brokkr-tls-staging"  # Self-signed or internal CA

ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: brokkr-staging.internal
      paths:
        - path: /
          pathType: Prefix

# Full security like production
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  fsGroup: 10001
  seccompProfile:
    type: RuntimeDefault

containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  capabilities:
    drop:
      - ALL
```

**Agent Values Files:**
Similar pattern but focused on:
- Broker connection URLs (different per environment)
- Resource limits (production vs dev)
- RBAC scope (cluster-wide for prod, namespace for dev/staging)

**README.md Content:**
```markdown
# Brokkr Values Files

Pre-configured values for common deployment scenarios.

## Usage

Install with a specific values file:

\`\`\`bash
helm install brokkr-broker . -f values/production.yaml
\`\`\`

## Available Scenarios

### Production (values/production.yaml)
- High availability (3 replicas)
- External managed database
- TLS with cert-manager
- Ingress enabled
- Strict security contexts
- Production resource limits

### Development (values/development.yaml)
- Single replica
- Bundled PostgreSQL (ephemeral)
- No TLS
- Minimal resources
- Relaxed security for debugging

### Staging (values/staging.yaml)
- 2 replicas
- External database
- TLS with internal certificates
- Production-like security
- Moderate resources

## Customization

These files are starting points. Create your own by:
1. Copy a file matching your needs
2. Customize specific values
3. Use with -f flag during install
\`\`\`
```

### Dependencies

- Depends on BROKKR-T-0009 (comprehensive configuration) - needs all config options
- Depends on BROKKR-T-0010 (PostgreSQL bundling) - production vs dev DB
- Depends on BROKKR-T-0011 (TLS support) - production TLS config
- Depends on BROKKR-T-0013 (security contexts) - security settings

### Risk Considerations

**Risk: Users using development values in production**
- Mitigation: Clear naming and warnings in files
- Add "DEVELOPMENT ONLY" comments prominently
- Document security implications

**Risk: Values files becoming outdated**
- Mitigation: Include in CI/CD testing
- Automated validation against base values.yaml
- Document maintenance process

**Risk: Too many options overwhelming users**
- Mitigation: Start with 3 core scenarios
- Provide clear decision tree in documentation
- Include inline comments explaining each choice

## Status Updates **[REQUIRED]**

*To be added during implementation*
