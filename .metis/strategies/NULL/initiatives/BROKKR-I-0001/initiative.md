---
id: ephemeral-work-system-with
level: initiative
title: "Ephemeral Work System with BuildRequest Implementation"
short_code: "BROKKR-I-0001"
created_at: 2025-10-08T14:59:07.902259+00:00
updated_at: 2025-10-17T09:52:35.402007+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/ready"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: ephemeral-work-system-with
---

# Ephemeral Work System with BuildRequest Implementation

## Context **[REQUIRED]**

Brokkr currently provides environment-aware control plane functionality for distributing Kubernetes objects across clusters via agent/broker architecture. However, the platform lacks two key capabilities:

1. **Native container image building**: Users must rely on external CI/CD systems for image builds before deploying through Brokkr
2. **Generic ephemeral work management**: No system exists for one-time, non-persistent operations (builds, tests, backups, etc.)

This initiative addresses both needs by:
- Creating a generic "ephemeral work" system for managing transient operations separate from persistent deployment state
- Implementing BuildRequest as the first ephemeral work type, providing Kubernetes-native build operations via CRDs
- Leveraging existing agent/broker work distribution patterns (matching stacks/deployment objects model)
- Completing the deployment pipeline within Brokkr's architecture
- Establishing patterns for future ephemeral work types (test runs, backup operations, migrations, etc.)

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Design and implement generic ephemeral work system in broker (reusable for builds, tests, etc.)
- Implement BuildRequest CRD as first ephemeral work type with buildah operator sidecar
- Support git-based build sources from day one (not limited to ConfigMaps)
- Leverage broker work queue with retry logic (exponential backoff, stale claim detection)
- Enable work targeting via existing label/annotation patterns (matching stack targeting)
- Support registry publishing with proper authentication
- Provide comprehensive failure handling (permanent vs retryable failures, max retries)

**Non-Goals:**
- Replace external CI/CD systems entirely (complement, not replace)
- Support build tools other than buildah (initial scope)
- Provide advanced build caching beyond buildah capabilities
- Support multi-stage parallel builds (single-stage focus initially)
- Implement all possible ephemeral work types now (builds only, extensible design for future)

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

### User Requirements
- **User Characteristics**: Kubernetes operators and platform engineers familiar with CRDs and YAML manifests
- **System Functionality**: Native container building integrated with existing Brokkr deployment workflows
- **User Interfaces**: Kubernetes CRD-based interaction, status via existing Brokkr APIs

### System Requirements
- **Functional Requirements**:
  - REQ-001: Broker must provide generic ephemeral work queue system (reusable across work types)
  - REQ-002: Agent pods must support optional buildah operator sidecar container
  - REQ-003: BuildRequest CRD must support git-based build sources with authentication
  - REQ-004: System must support work targeting matching stack pattern (via broker targets table)
  - REQ-005: Built images must be publishable to container registries with secret-based auth
  - REQ-006: Work queue must implement retry logic (max retries, exponential backoff, stale claims)
  - REQ-007: System must distinguish permanent vs retryable failures

- **Non-Functional Requirements**:
  - NFR-001: Buildah operator runs as isolated sidecar (protects agent from build failures)
  - NFR-002: Builds must support both rootless and rootful modes based on security context
  - NFR-003: Work queue must handle concurrent operations from multiple agents
  - NFR-004: Registry and git credentials managed securely via Kubernetes secrets
  - NFR-005: Failed work cleaned up after TTL expiration (configurable per work item)

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

### Use Case 1: Git-Based Container Build
- **Actor**: Platform Engineer
- **Scenario**:
  1. Engineer submits BuildRequest to broker API with git repository URL and target registry
  2. Broker creates ephemeral work item and determines target agents based on labels
  3. Agent polls broker, claims the work item
  4. Agent applies BuildRequest CRD to its cluster
  5. Buildah operator sidecar watches CRD, clones git repo, executes build, pushes to registry
  6. Agent watches CRD status, reports completion to broker
  7. Work item cleaned up after TTL expires
- **Expected Outcome**: Container image built from git source and pushed to registry

### Use Case 2: Build Retry After Transient Failure
- **Actor**: Platform Engineer
- **Scenario**:
  1. Engineer submits BuildRequest to build and push image
  2. Agent claims work, buildah operator attempts build
  3. Registry is temporarily unreachable, build fails
  4. Agent marks failure as RETRYABLE, increments retry count
  5. Broker calculates next_retry_after with exponential backoff
  6. After backoff period, different agent claims and successfully completes build
- **Expected Outcome**: Build automatically retries after transient failure and succeeds

### Use Case 3: Targeted Build with Specific Resources
- **Actor**: Operations Team
- **Scenario**:
  1. Team submits BuildRequest via broker API with labels targeting GPU-enabled agents
  2. Broker populates ephemeral_work_targets table with agents matching labels
  3. Only GPU-capable agents can claim this work
  4. Build executes on appropriate infrastructure with GPU access
- **Expected Outcome**: Build executes on correctly-resourced infrastructure

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

### Overview
This initiative introduces a generic ephemeral work system with BuildRequest as the first implementation:

1. **Ephemeral Work Queue (Broker)**: Generic work queue system matching stack targeting patterns
2. **BuildRequest CRD**: Kubernetes custom resource for build specifications
3. **Buildah Operator Sidecar**: Independent CRD controller running alongside agent
4. **Agent Orchestration**: Claims work from broker, applies CRDs, monitors status
5. **Retry & Failure Handling**: Comprehensive retry logic with exponential backoff

### Component Architecture

```
┌─────────────────────────────────────────────┐
│  Brokkr Agent Pod                           │
│                                             │
│  ┌─────────────────┐    ┌────────────────┐ │
│  │ agent container │    │    buildah     │ │
│  │                 │    │    operator    │ │
│  │ - Poll broker   │    │    sidecar     │ │
│  │ - Claim work    │    │                │ │
│  │ - Apply CRDs    │    │ - Watch CRDs   │ │
│  │ - Watch status  │    │ - Clone git    │ │
│  │ - Report to     │    │ - Run buildah  │ │
│  │   broker        │    │ - Push images  │ │
│  └─────────────────┘    └────────────────┘ │
│           │                      │          │
│           └──────────────────────┘          │
│              Kubernetes API                 │
└─────────────────────────────────────────────┘
```

### Sequence Flow (Flow B - Complete Pattern)

1. **User submits work to broker**
   - POST `/api/v1/ephemeral-work` with BuildRequest spec and target labels
   - Broker creates ephemeral_work record, populates ephemeral_work_targets based on label matching

2. **Agent claims work**
   - Agent polls: GET `/api/v1/agents/{id}/ephemeral-work/pending`
   - Broker returns work where agent appears in targets table and status=PENDING
   - Agent claims via: POST `/api/v1/ephemeral-work/{id}/claim`
   - Broker atomically updates: status=CLAIMED, claimed_by=agent_id, claimed_at=NOW()

3. **Agent applies CRD to cluster**
   - Agent deserializes crd_spec from work item
   - Agent applies BuildRequest CRD to its Kubernetes cluster via k8s API

4. **Buildah operator executes build**
   - Operator sidecar watches for BuildRequest CRDs (independent controller)
   - Operator clones git repo, runs buildah build, pushes to registry
   - Operator updates BuildRequest.status directly in cluster

5. **Agent monitors and reports**
   - Agent watches BuildRequest status via k8s API
   - On completion/failure, agent reports: POST `/api/v1/ephemeral-work/{id}/complete`
   - Broker updates: status=COMPLETED/FAILED_PERMANENT/FAILED_RETRYABLE, completed_at=NOW()

6. **Retry handling (if failure)**
   - If FAILED_RETRYABLE and retry_count < max_retries:
     - Broker calculates next_retry_after = NOW() + (2^retry_count * base_delay)
     - Status returns to PENDING after backoff period
   - If FAILED_PERMANENT or retry_count >= max_retries:
     - Work stays in failed state until TTL cleanup

7. **TTL cleanup**
   - Background job soft-deletes work where completed_at + ttl_seconds < NOW()

## Detailed Design **[REQUIRED]**

### Broker Database Schema

Following the stack targeting pattern exactly, using normalized tables (no JSONB per project standards):

```sql
-- Main ephemeral work table (generic for all work types)
CREATE TABLE ephemeral_work (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,

    work_type VARCHAR(50) NOT NULL,  -- 'build', 'test', etc.
    crd_spec TEXT NOT NULL,  -- Full CRD YAML/JSON serialized

    -- Status management
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    -- PENDING, CLAIMED, COMPLETED, FAILED_PERMANENT, FAILED_RETRYABLE
    claimed_by UUID REFERENCES agents(id) NULL,
    claimed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,

    -- Retry handling (combined: max retries + stale detection + exponential backoff)
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    claim_timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    next_retry_after TIMESTAMP WITH TIME ZONE,

    -- Results
    result_message TEXT,

    -- TTL cleanup
    ttl_seconds INTEGER NOT NULL DEFAULT 3600
);

-- Work targeting (mirrors agent_targets pattern for stacks)
CREATE TABLE ephemeral_work_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_id UUID NOT NULL REFERENCES ephemeral_work(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE(work_id, agent_id)
);

-- Indexes
CREATE INDEX idx_ephemeral_work_status ON ephemeral_work(status);
CREATE INDEX idx_ephemeral_work_type ON ephemeral_work(work_type);
CREATE INDEX idx_ephemeral_work_claimed_by ON ephemeral_work(claimed_by);
CREATE INDEX idx_ephemeral_work_next_retry ON ephemeral_work(next_retry_after)
    WHERE status = 'PENDING' AND next_retry_after IS NOT NULL;
CREATE INDEX idx_ephemeral_work_ttl ON ephemeral_work(completed_at, ttl_seconds)
    WHERE deleted_at IS NULL AND completed_at IS NOT NULL;
CREATE INDEX idx_ephemeral_work_targets_work ON ephemeral_work_targets(work_id);
CREATE INDEX idx_ephemeral_work_targets_agent ON ephemeral_work_targets(agent_id);

-- Trigger
CREATE TRIGGER update_ephemeral_work_timestamp
BEFORE UPDATE ON ephemeral_work
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
```

**Key Design Notes:**
- CRD spec stored as TEXT (broker treats it as opaque data)
- Targeting uses separate join table (matches stack pattern exactly)
- Three-part retry strategy: max_retries, claim_timeout_seconds, next_retry_after
- TTL-based cleanup for completed work
- Status differentiates permanent vs retryable failures

### BuildRequest CRD Specification

```yaml
apiVersion: brokkr.io/v1
kind: BuildRequest
metadata:
  name: my-app-build
spec:
  source:
    git:
      repository: "https://github.com/org/repo"
      ref: "main"
      secretRef: "git-credentials"  # Optional: for private repos
  buildContext:
    dockerfile: "./Dockerfile"
    contextDir: "."
  image:
    name: "registry.example.com/my-app"
    tag: "latest"
    registry:
      secretRef: "registry-credentials"
  buildArgs:
    - name: "BUILD_VERSION"
      value: "1.0.0"
  resources:
    requests:
      cpu: "1"
      memory: "2Gi"
    limits:
      cpu: "2"
      memory: "4Gi"
  ttlSecondsAfterFinished: 3600  # Auto-cleanup
status:
  phase: "Pending"  # Pending, Building, Succeeded, Failed
  startTime: "2024-01-01T12:00:00Z"
  completionTime: "2024-01-01T12:05:00Z"
  message: "Build completed successfully"
  imageDigest: "sha256:abc123..."
```

### Broker API Endpoints

Reusable generic ephemeral work endpoints:

- `POST /api/v1/ephemeral-work` - Create new work item (with target labels)
- `GET /api/v1/agents/{id}/ephemeral-work/pending` - Get claimable work for agent
- `POST /api/v1/ephemeral-work/{id}/claim` - Atomically claim work
- `POST /api/v1/ephemeral-work/{id}/complete` - Report completion/failure
- `GET /api/v1/ephemeral-work/{id}` - Get work item details
- `DELETE /api/v1/ephemeral-work/{id}` - Cancel/delete work

### Agent Components

**Agent Ephemeral Work Module** (`crates/brokkr-agent/src/ephemeral_work/mod.rs`):
- Poll broker for pending work
- Claim work items
- Deserialize and apply CRDs to cluster
- Watch CRD status via k8s API
- Report completion/failure to broker

**Agent Configuration** (add to existing Settings):
```toml
[agent.ephemeral_work]
enabled = true
poll_interval_seconds = 30
```

**Buildah Operator Sidecar** (separate container in agent pod):
- Independent Rust binary: `crates/brokkr-buildah-operator`
- Watches BuildRequest CRDs in cluster
- Executes buildah commands
- Updates CRD status
- No direct communication with agent or broker

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

### Unit Testing
- **Strategy**: Test individual builder components and buildah integration logic
- **Coverage Target**: 80% coverage for builder module components
- **Tools**: Rust built-in test framework, mock buildah CLI responses

### Integration Testing
- **Strategy**: End-to-end testing with real buildah builds in test environment
- **Test Environment**: Local Kubernetes cluster (kind/minikube) with test registry
- **Data Management**: Sample Dockerfiles and test projects in test fixtures

### System Testing
- **Strategy**: Multi-agent build scenarios with concurrent builds and resource limits
- **User Acceptance**: Validate BuildRequest CRD workflows match existing Brokkr patterns
- **Performance Testing**: Concurrent build execution, resource isolation, queue management

### Test Selection
- Focus on buildah integration, CRD lifecycle, and broker/agent communication
- Registry authentication and image pushing workflows
- Error handling and build failure scenarios

## Alternatives Considered **[REQUIRED]**

### 1. Separate Builder Service
**Approach**: Deploy builder as standalone service separate from agents
**Rejected**: Would duplicate work assignment and communication patterns already implemented in agent/broker architecture

### 2. Docker-in-Docker Builds
**Approach**: Use Docker daemon for builds instead of buildah
**Rejected**: Requires privileged containers and Docker daemon, buildah provides rootless capabilities and better Kubernetes integration

### 3. Tekton/Jenkins X Integration
**Approach**: Integrate with existing cloud-native build tools
**Rejected**: Adds external dependencies and complexity, goal is to provide native Brokkr build capabilities

### 4. Build Results as Deployment Objects
**Approach**: Treat build outputs as deployment objects in existing stack system
**Rejected**: Builds are ephemeral operations, not persistent deployment state, requires separate queue management

## Implementation Plan **[REQUIRED]**

### Phase 1: Generic Ephemeral Work System (2-3 weeks)
- Create ephemeral_work and ephemeral_work_targets tables (migration 07)
- Implement broker DAL for ephemeral work operations
- Add generic ephemeral work API endpoints
- Implement work targeting logic (reuse stack targeting patterns)
- Add retry logic (max retries, stale claim detection, exponential backoff)
- Implement TTL cleanup background job

### Phase 2: Agent Ephemeral Work Integration (2 weeks)
- Create ephemeral_work module in agent
- Implement broker polling for pending work
- Add work claim logic
- Add CRD apply functionality (deserialize and apply to cluster)
- Add CRD status watching via k8s API
- Implement completion reporting to broker

### Phase 3: BuildRequest CRD & Operator Sidecar (3-4 weeks)
- Define BuildRequest CRD specification
- Create new crate: brokkr-buildah-operator
- Implement CRD controller (watch BuildRequests)
- Add git clone functionality with authentication
- Implement buildah CLI integration
- Add registry push with secret-based authentication
- Update CRD status from operator

### Phase 4: Pod Configuration & Deployment (1-2 weeks)
- Create Dockerfile for buildah-operator sidecar
- Update agent Dockerfile (if needed for git/buildah dependencies)
- Create Kubernetes manifests for agent pod with sidecar
- Add agent configuration for ephemeral work enablement
- Document sidecar deployment patterns

### Phase 5: Retry & Failure Handling (1-2 weeks)
- Implement retry decision logic in agent
- Add permanent vs retryable failure differentiation
- Implement exponential backoff calculation in broker
- Add stale claim detection and recovery
- Test failure scenarios comprehensively

### Phase 6: Testing & Documentation (2 weeks)
- Unit tests for all components
- Integration tests with real builds
- End-to-end testing with retry scenarios
- Performance testing (concurrent builds, multiple agents)
- Documentation: architecture, deployment, usage examples
- Example BuildRequest manifests

**Total Estimated Timeline**: 11-15 weeks

### Dependencies Between Phases
- Phase 2 depends on Phase 1 (needs ephemeral work system)
- Phase 3 can be developed in parallel with Phase 2 (independent operator)
- Phase 4 depends on Phases 2 & 3 (needs both components)
- Phase 5 depends on Phases 1-4 (needs full system)
- Phase 6 spans all phases (continuous testing)
