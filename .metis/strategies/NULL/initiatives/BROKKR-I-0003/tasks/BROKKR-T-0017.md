---
id: add-helm-installation-guide-to
level: task
title: "Add Helm installation guide to documentation"
short_code: "BROKKR-T-0017"
created_at: 2025-10-21T12:37:06.064872+00:00
updated_at: 2025-10-21T16:59:45.480721+00:00
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

# Add Helm installation guide to documentation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create comprehensive Helm-based installation documentation for both broker and agent components. Cover basic installation scenarios including development setup with bundled PostgreSQL and connecting agents to brokers. This is the primary installation method users will follow.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Installation guide updated in `docs/content/getting-started/installation.md`
- [x] Development setup documented (broker with bundled PostgreSQL)
- [x] Agent installation documented (connecting to broker)
- [x] Prerequisites clearly listed (Kubernetes cluster, kubectl, Helm 3.8+)
- [x] Quick start example: working broker + agent in under 10 minutes
- [x] Common configuration options documented (database, TLS, resources)
- [x] Values file examples provided for different scenarios (references to existing files)
- [x] Verification steps included (health checks, connectivity tests)
- [x] Link to Helm chart reference documentation



## Documentation Sections **[CONDITIONAL: Documentation Task]**

### Content Outline for docs/content/getting-started/installation.md

**Prerequisites Section:**
- Kubernetes cluster (v1.20+)
- kubectl CLI configured
- Helm 3.8+ installed
- Optional: PostgreSQL database (if not using bundled)

**Quick Start (Development Setup):**
```bash
# 1. Install broker with bundled PostgreSQL
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set postgresql.enabled=true \
  --wait

# 2. Get broker URL and create agent PAK
kubectl port-forward svc/brokkr-broker 3000:3000 &
BROKER_URL="http://brokkr-broker:3000"

# Create agent and get PAK (document how to do this via API/CLI)

# 3. Install agent
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=$BROKER_URL \
  --set broker.pak="<PAK_FROM_STEP_2>" \
  --wait
```

**Broker Installation Section:**
- Basic installation with bundled PostgreSQL
- Installation with external PostgreSQL
- Common configuration options (resources, replicas, TLS)
- Using values files (development.yaml, staging.yaml, production.yaml)

**Agent Installation Section:**
- Prerequisites (running broker, agent PAK)
- Basic installation command
- Configuration options (broker URL, polling intervals, RBAC mode)
- Multiple agents per cluster vs one agent per cluster

**Verification Section:**
- Check broker health: `kubectl exec deploy/brokkr-broker -- wget -qO- http://localhost:3000/healthz`
- Check agent health: `kubectl exec deploy/brokkr-agent -- wget -qO- http://localhost:8080/healthz`
- Verify agent registration in broker logs
- Test deployment of a simple resource

**Configuration Examples:**
- Development setup (bundled DB, minimal resources)
- External database configuration
- TLS/SSL setup
- Resource limits and requests
- Custom values file structure

**Next Steps:**
- Link to quick start guide for deploying first application
- Link to configuration reference
- Link to troubleshooting guide (when created)

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**File to Update:**
- `docs/content/getting-started/installation.md` - Replace existing source-based installation content

**Structure:**
1. Replace "Building from Source" section with Helm installation as primary method
2. Move source build instructions to a separate "Development" or "Advanced" section
3. Keep Docker development environment section as-is (for contributors)
4. Add clear navigation between installation methods

**Key Documentation Points:**
- Emphasize Helm as the recommended installation method
- Show OCI registry installation (from T-0016) once available
- Provide GitHub Release .tgz fallback for older Helm versions
- Use actual working examples that users can copy-paste
- Include output examples so users know what success looks like

**Values File Documentation:**
Reference the existing values files created in T-0014:
- `charts/brokkr-broker/values/development.yaml`
- `charts/brokkr-broker/values/staging.yaml`
- `charts/brokkr-broker/values/production.yaml`

### Dependencies

- Depends on T-0016 (OCI chart publishing) for final OCI URLs
- Can start with GitHub Release .tgz installation instructions
- Helm charts already exist from T-0006, T-0007 (Phase 1)
- Values files exist from T-0014 (Phase 2)

### Risk Considerations

**Risk: Installation commands may fail if charts aren't published yet**
- Mitigation: Test all commands against actual published charts
- Use placeholder version numbers, update after first release

**Risk: PostgreSQL bundling complexity may confuse users**
- Mitigation: Clearly separate "quick start" from "production setup"
- Provide decision matrix: when to use bundled vs external DB

**Risk: PAK generation not well documented**
- Mitigation: Document API endpoint or CLI command for creating agents
- Provide complete example including PAK retrieval

**Risk: Users may skip verification steps**
- Mitigation: Include verification as part of each installation section
- Make verification commands copy-pasteable

## Status Updates **[REQUIRED]**

### 2025-10-21: Documentation Complete

Successfully created comprehensive Helm installation guide:

**Documentation Sections Added:**
- Prerequisites verification (Kubernetes, kubectl, Helm 3.8+)
- Quick Start guide: broker + agent installation in 5 steps
- Detailed broker installation (bundled PostgreSQL and external database)
- Agent installation with PAK generation instructions
- Chart version management (releases and development builds)
- Upgrade and uninstall procedures
- Verification steps (health checks, connectivity tests, test deployments)
- Configuration reference tables
- Troubleshooting common issues
- Building from source (for contributors)

**Key Features:**
- OCI registry installation from ghcr.io/colliery-io/charts
- References to existing values files (development.yaml, staging.yaml, production.yaml)
- Copy-paste ready examples
- Expected output examples for verification
- Links to chart values files on GitHub

**User Guidance Improvements:**
- Helm emphasized as primary/recommended installation method
- Source builds moved to end as advanced/contributor option
- Clear separation between development and production setups
- Simplified configuration by referencing existing values files

Skipped production and multi-cluster scenarios as requested.
