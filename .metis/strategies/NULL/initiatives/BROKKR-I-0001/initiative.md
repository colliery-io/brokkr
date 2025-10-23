---
id: ephemeral-work-system-with
level: initiative
title: "Ephemeral Work System with Shipwright Build Integration"
short_code: "BROKKR-I-0001"
created_at: 2025-10-08T14:59:07.902259+00:00
updated_at: 2025-10-22T14:45:23.097831+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/ready"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: ephemeral-work-system-with
---

# Ephemeral Work System with Shipwright Build Integration

## Context **[REQUIRED]**

Brokkr currently provides environment-aware control plane functionality for distributing Kubernetes objects across clusters via agent/broker architecture. However, the platform lacks two key capabilities:

1. **Native container image building**: Users must rely on external CI/CD systems for image builds before deploying through Brokkr
2. **Generic ephemeral work management**: No system exists for one-time, non-persistent operations (builds, tests, backups, etc.)

This initiative addresses both needs by:
- Creating a generic "ephemeral work" system for managing transient operations separate from persistent deployment state
- Integrating Shipwright Build (CNCF Sandbox project) as the first ephemeral work type for production-ready container builds
- Leveraging existing agent/broker work distribution patterns (matching stacks/deployment objects model)
- Completing the deployment pipeline within Brokkr's architecture
- Establishing patterns for future ephemeral work types (test runs, backup operations, migrations, etc.)

**Architectural Decision**: Rather than building a custom buildah operator from scratch, this initiative adopts a hybrid approach using Shipwright Build, a mature CNCF project with v1beta1 API stability. Shipwright provides production-ready build capabilities (multi-arch, vulnerability scanning, comprehensive retry logic) while reducing implementation time by 40% and eliminating long-term maintenance burden. The generic ephemeral work system remains valuable regardless, as it will support future work types beyond builds.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Design and implement generic ephemeral work system in broker (reusable for builds, tests, backups, etc.)
- Integrate Shipwright Build as first ephemeral work type for production-ready container builds
- Support git-based build sources, multi-architecture builds, and vulnerability scanning via Shipwright
- Leverage broker work queue with retry logic (exponential backoff, stale claim detection)
- Enable work targeting via existing label/annotation patterns (matching stack targeting)
- Support registry publishing with proper authentication through Shipwright
- Provide comprehensive failure handling (permanent vs retryable failures, max retries)
- Establish evaluation criteria for assessing Shipwright's fit (decision point at 8-10 weeks)

**Non-Goals:**
- Replace external CI/CD systems entirely (complement, not replace)
- Build custom buildah operator initially (Shipwright provides this, custom operator remains option if needed)
- Support non-Shipwright build tools in initial implementation
- Implement all possible ephemeral work types now (builds only, extensible design for future)
- Commit permanently to Shipwright (preserve option to build custom operator based on evaluation)

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

### User Requirements
- **User Characteristics**: Kubernetes operators and platform engineers familiar with CRDs and YAML manifests
- **System Functionality**: Native container building integrated with existing Brokkr deployment workflows
- **User Interfaces**: Kubernetes CRD-based interaction, status via existing Brokkr APIs

### System Requirements
- **Functional Requirements**:
  - REQ-001: Broker must provide generic ephemeral work queue system (reusable across work types)
  - REQ-002: Agent clusters must have Shipwright Build and Tekton installed as prerequisites
  - REQ-003: Agent must create and monitor Shipwright Build/BuildRun resources from ephemeral work items
  - REQ-004: System must support work targeting matching stack pattern (via broker targets table)
  - REQ-005: Built images must be publishable to container registries with secret-based auth via Shipwright
  - REQ-006: Work queue must implement retry logic (max retries, exponential backoff, stale claims)
  - REQ-007: System must distinguish permanent vs retryable failures
  - REQ-008: System must map Shipwright BuildRun status to broker ephemeral work status

- **Non-Functional Requirements**:
  - NFR-001: Agent remains single-container (no buildah operator sidecar needed with Shipwright)
  - NFR-002: Builds support both rootless and rootful modes via Shipwright BuildStrategy configuration
  - NFR-003: Work queue must handle concurrent operations from multiple agents
  - NFR-004: Registry and git credentials managed securely via Kubernetes secrets (Shipwright pattern)
  - NFR-005: Failed work cleaned up after TTL expiration (configurable per work item)
  - NFR-006: Shipwright and Tekton installation documented with version requirements (Tekton v0.59+, Shipwright v0.17.0+)

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

### Use Case 1: Git-Based Container Build
- **Actor**: Platform Engineer
- **Scenario**:
  1. Engineer submits build work to broker API with Shipwright Build spec (git repository URL, target registry)
  2. Broker creates ephemeral work item and determines target agents based on labels
  3. Agent polls broker, claims the work item
  4. Agent creates Shipwright Build and BuildRun resources in its cluster
  5. Shipwright + Tekton execute build: clone git repo, run buildah via ClusterBuildStrategy, push to registry
  6. Agent watches BuildRun status, reports completion to broker
  7. Work item cleaned up after TTL expires
- **Expected Outcome**: Container image built from git source and pushed to registry via production-ready Shipwright system

### Use Case 2: Build Retry After Transient Failure
- **Actor**: Platform Engineer
- **Scenario**:
  1. Engineer submits build work to build and push image
  2. Agent claims work, creates Shipwright BuildRun
  3. Registry is temporarily unreachable, Shipwright build fails
  4. Agent marks failure as RETRYABLE based on BuildRun status, increments retry count
  5. Broker calculates next_retry_after with exponential backoff
  6. After backoff period, different agent claims and successfully completes build
- **Expected Outcome**: Build automatically retries after transient failure and succeeds (leverages both Shipwright's retry logic and broker's work queue retry)

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
This initiative introduces a generic ephemeral work system with Shipwright Build integration as the first implementation:

1. **Ephemeral Work Queue (Broker)**: Generic work queue system matching stack targeting patterns
2. **Shipwright Build Integration**: Leverage CNCF Shipwright Build (v1beta1) for production-ready container builds
3. **Agent Orchestration**: Claims work from broker, creates Shipwright Build/BuildRun, monitors status
4. **Retry & Failure Handling**: Comprehensive retry logic with exponential backoff (broker-level + Shipwright-level)
5. **Hybrid Approach**: Start with Shipwright, preserve option to build custom operator if limitations discovered

### Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Kubernetes Cluster (Agent)                                 │
│                                                             │
│  ┌───────────────────────┐    ┌─────────────────────────┐  │
│  │  Brokkr Agent Pod     │    │  Shipwright + Tekton    │  │
│  │  (single container)   │    │  (installed separately) │  │
│  │                       │    │                         │  │
│  │  - Poll broker        │───→│  - Watch Build CRDs     │  │
│  │  - Claim work         │    │  - Execute BuildRuns    │  │
│  │  - Create Build/      │    │  - Clone git repos      │  │
│  │    BuildRun CRDs      │    │  - Run buildah builds   │  │
│  │  - Watch BuildRun     │←───│  - Push to registries   │  │
│  │    status             │    │  - Update status        │  │
│  │  - Report to broker   │    │                         │  │
│  └───────────────────────┘    └─────────────────────────┘  │
│              │                            │                 │
│              └────────────────────────────┘                 │
│                   Kubernetes API                            │
└─────────────────────────────────────────────────────────────┘
```

**Key Advantages:**
- Agent remains single-container (simpler deployment)
- Shipwright handles all build complexity (git clone, buildah execution, registry push)
- Tekton provides robust execution backend with retry, timeout, resource management
- Production-ready from day one (v1beta1 API stability)

### Sequence Flow (Flow B - Complete Pattern)

1. **User submits work to broker**
   - POST `/api/v1/ephemeral-work` with BuildRequest spec and target labels
   - Broker creates ephemeral_work record, populates ephemeral_work_targets based on label matching

2. **Agent claims work**
   - Agent polls: GET `/api/v1/agents/{id}/ephemeral-work/pending`
   - Broker returns work where agent appears in targets table and status=PENDING
   - Agent claims via: POST `/api/v1/ephemeral-work/{id}/claim`
   - Broker atomically updates: status=CLAIMED, claimed_by=agent_id, claimed_at=NOW()

3. **Agent creates Shipwright resources in cluster**
   - Agent deserializes crd_spec from work item (Shipwright Build spec)
   - Agent creates Shipwright Build resource (if not exists) via k8s API
   - Agent creates Shipwright BuildRun resource to trigger build execution

4. **Shipwright + Tekton execute build**
   - Shipwright controller watches BuildRun, creates Tekton TaskRun
   - Tekton executes buildah ClusterBuildStrategy: clone git, run build, push to registry
   - Shipwright updates BuildRun.status with results (image digest, completion time, errors)

5. **Agent monitors and reports**
   - Agent watches BuildRun status via k8s API
   - On completion/failure, agent maps BuildRun conditions to broker status
   - Agent reports: POST `/api/v1/ephemeral-work/{id}/complete` with image digest, logs
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
- CRD spec stored as TEXT containing Shipwright Build YAML (broker treats it as opaque data)
- work_type value: 'shipwright-build' for build operations
- Targeting uses separate join table (matches stack pattern exactly)
- Three-part retry strategy: max_retries, claim_timeout_seconds, next_retry_after
- TTL-based cleanup for completed work
- Status differentiates permanent vs retryable failures
- Agent creates both Build and BuildRun from single work item

### Shipwright Build Specification

Agent creates Shipwright Build and BuildRun resources from broker work items:

```yaml
# Shipwright Build (reusable template)
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: my-app-build
spec:
  source:
    type: Git
    git:
      url: https://github.com/org/repo
      revision: main
      cloneSecret: git-credentials  # Optional: for private repos
    contextDir: "."
  strategy:
    name: buildah
    kind: ClusterBuildStrategy
  paramValues:
    - name: dockerfile
      value: "./Dockerfile"
    - name: build-args
      values:
        - value: "BUILD_VERSION=1.0.0"
  output:
    image: registry.example.com/my-app:latest
    pushSecret: registry-credentials
    annotations:
      "org.opencontainers.image.source": "https://github.com/org/repo"
  timeout: 15m
  retention:
    ttlAfterSucceeded: 1h
    ttlAfterFailed: 24h

---
# Shipwright BuildRun (execution instance)
apiVersion: shipwright.io/v1beta1
kind: BuildRun
metadata:
  generateName: my-app-buildrun-
spec:
  build:
    name: my-app-build
  serviceAccount: builder-sa

# BuildRun.status provides:
# - conditions (Succeeded, Failed, etc.)
# - output.digest (sha256:abc123...)
# - output.size (compressed bytes)
# - sources.git.commitSha, commitAuthor, branchName
# - completionTime, startTime
# - failureDetails (pod, container, reason, message)
```

**Key Shipwright Features Used:**
- Git source with authentication (cloneSecret)
- buildah ClusterBuildStrategy (pre-installed on agent clusters)
- Build parameters for Dockerfile path and build args
- Registry push with authentication (pushSecret)
- Retention policies for automatic cleanup
- Comprehensive status with image digest and failure details

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
- Deserialize Shipwright Build specs from work items
- Create Shipwright Build and BuildRun resources in cluster
- Watch BuildRun status via k8s API
- Map BuildRun conditions to broker status (COMPLETED/FAILED_PERMANENT/FAILED_RETRYABLE)
- Report completion/failure to broker with image digest and logs

**Agent Configuration** (add to existing Settings):
```toml
[agent.ephemeral_work]
enabled = true
poll_interval_seconds = 30

[agent.ephemeral_work.shipwright]
service_account = "builder-sa"  # ServiceAccount for BuildRuns
default_timeout = "15m"
build_retention_succeeded = "1h"
build_retention_failed = "24h"
```

**Shipwright Integration Module** (`crates/brokkr-agent/src/ephemeral_work/shipwright.rs`):
- Create Build resources from work specs
- Create BuildRun resources to trigger execution
- Watch BuildRun status and conditions
- Parse BuildRun output (image digest, size, git commit info)
- Map Shipwright failure reasons to broker failure types

**No Operator Sidecar Needed:**
- Agent remains single-container
- Shipwright + Tekton (installed in cluster) handle build execution
- Simpler deployment model than custom operator sidecar

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

### Unit Testing
- **Strategy**: Test ephemeral work queue logic, Shipwright Build/BuildRun creation, status mapping
- **Coverage Target**: 80% coverage for ephemeral work module components
- **Tools**: Rust built-in test framework, mock Kubernetes API responses for Shipwright resources

### Integration Testing
- **Strategy**: End-to-end testing with real Shipwright builds in test environment
- **Test Environment**: Local Kubernetes cluster (kind) with Shipwright + Tekton installed, test registry
- **Data Management**: Sample Dockerfiles and test git repositories in test fixtures
- **Cluster Prerequisites**: Document Shipwright v0.17.0+ and Tekton v0.59+ installation steps

### System Testing
- **Strategy**: Multi-agent build scenarios with concurrent builds and resource limits
- **User Acceptance**: Validate Shipwright integration matches existing Brokkr patterns
- **Performance Testing**: Concurrent build execution, Shipwright/Tekton resource usage, queue management
- **Shipwright Feature Testing**: Multi-arch builds, vulnerability scanning, build caching

### Test Selection
- Focus on Shipwright Build/BuildRun lifecycle and broker/agent communication
- Registry authentication via Shipwright pushSecret pattern
- Error handling and Shipwright failure condition mapping
- BuildRun status watching and completion detection
- Evaluation criteria: Does Shipwright meet 80%+ of build requirements?

## Alternatives Considered **[REQUIRED]**

### 1. Shipwright Build Integration (CHOSEN)
**Approach**: Integrate CNCF Shipwright Build (v1beta1) as build execution engine
**Chosen Because**:
- Production-ready from day one with v1beta1 API stability
- Feature-rich: multi-arch builds, vulnerability scanning, comprehensive retry logic
- Reduces implementation time by 40% (5-6 weeks saved)
- Eliminates long-term maintenance burden (CNCF/Red Hat/IBM maintained)
- Battle-tested: used by IBM Cloud Code Engine, OpenShift Builds v2
- Generic ephemeral work system still provides value for future work types (tests, backups, etc.)
**Tradeoffs**: Requires Tekton dependency, operational complexity of external components

### 2. Custom BuildRequest + buildah Operator Sidecar
**Approach**: Build custom BuildRequest CRD and buildah operator sidecar from scratch
**Considered**: Complete control, no external dependencies, lighter weight
**Rejected**: 11-15 weeks implementation, maintenance burden, reinventing the wheel, feature gap (multi-arch, vuln scanning), security responsibility

### 3. Separate Builder Service
**Approach**: Deploy builder as standalone service separate from agents
**Rejected**: Would duplicate work assignment and communication patterns already implemented in agent/broker architecture

### 4. Direct Tekton Integration (no Shipwright)
**Approach**: Use Tekton Pipelines directly with buildah Tasks from Tekton Hub
**Rejected**: Lower-level abstraction requires more YAML management, Shipwright provides better build-specific abstractions and features

### 5. Kaniko Executor
**Approach**: Use Kaniko for unprivileged builds without buildah
**Rejected**: Kaniko is just an executor image, not a full build system; requires custom orchestration similar to option 2

### 6. Build Results as Deployment Objects
**Approach**: Treat build outputs as deployment objects in existing stack system
**Rejected**: Builds are ephemeral operations, not persistent deployment state, requires separate queue management

## Implementation Plan **[REQUIRED]**

### Phase 1: Generic Ephemeral Work System (2-3 weeks) - UNCHANGED
- Create ephemeral_work and ephemeral_work_targets tables (migration 07)
- Implement broker DAL for ephemeral work operations
- Add generic ephemeral work API endpoints (reusable for all work types)
- Implement work targeting logic (reuse stack targeting patterns exactly)
- Add retry logic (max retries, stale claim detection, exponential backoff)
- Implement TTL cleanup background job
- Add work type discriminator support ('shipwright-build', 'test-run', etc.)

**Deliverables:**
- Database migration 07 with ephemeral_work tables
- Broker DAL module with generic work operations
- API endpoints: POST /ephemeral-work, GET /agents/{id}/ephemeral-work/pending, POST /ephemeral-work/{id}/claim, POST /ephemeral-work/{id}/complete
- Work targeting query matching stack pattern
- Retry and TTL background jobs

### Phase 2: Agent Ephemeral Work Integration (2 weeks) - UNCHANGED
- Create ephemeral_work module in agent (`crates/brokkr-agent/src/ephemeral_work/`)
- Implement broker polling for pending work (respects poll_interval_seconds)
- Add work claim logic with atomic broker updates
- Add generic CRD apply functionality (deserialize YAML/JSON, apply to cluster)
- Add generic CRD status watching via k8s API (async watch streams)
- Implement completion reporting to broker with result data

**Deliverables:**
- Agent ephemeral_work module with broker integration
- Generic CRD application logic (work-type agnostic)
- Status watching framework for any CRD type
- Completion reporting with success/failure/retry differentiation

### Phase 3: Shipwright Build Integration (1-2 weeks) - CHANGED from 3-4 weeks
- Implement Shipwright integration module (`crates/brokkr-agent/src/ephemeral_work/shipwright.rs`)
- Add Shipwright Build creation from work specs
- Add Shipwright BuildRun creation and triggering
- Implement BuildRun status watching (watch for completion conditions)
- Map BuildRun status to broker status (Succeeded → COMPLETED, Failed → FAILED_PERMANENT/RETRYABLE)
- Parse BuildRun output (image digest, size, git commit info, failure details)
- Add Kubernetes client support for Shipwright CRDs (kube-rs with custom resources)

**Deliverables:**
- Shipwright integration module with Build/BuildRun creation
- BuildRun status mapper (conditions → broker status)
- Example Shipwright Build specs for broker API
- Agent configuration for Shipwright settings (service account, timeouts, retention)

**NO LONGER NEEDED:**
- ~~Custom BuildRequest CRD specification~~
- ~~brokkr-buildah-operator crate~~
- ~~CRD controller implementation~~
- ~~Git clone, buildah CLI, registry push logic~~

### Phase 4: Documentation & Cluster Prerequisites (1 week) - CHANGED from 1-2 weeks
- Document Shipwright + Tekton installation requirements
  - Tekton Pipelines v0.59+ installation steps
  - Shipwright Build v0.17.0+ installation steps
  - buildah ClusterBuildStrategy installation
- Document builder-sa ServiceAccount setup with RBAC
- Document git and registry secret configuration (Shipwright pattern)
- Create example Shipwright Build YAML for common scenarios
- Document agent configuration for Shipwright integration
- Update Brokkr Helm charts with Shipwright prerequisites documentation

**Deliverables:**
- Installation guide for Shipwright + Tekton on agent clusters
- ServiceAccount and RBAC manifests for builds
- Example Build/BuildRun specifications
- Troubleshooting guide for common Shipwright issues
- Agent configuration documentation

**NO LONGER NEEDED:**
- ~~Buildah-operator sidecar Dockerfile~~
- ~~Multi-container agent pod manifests~~
- ~~Sidecar deployment patterns~~

### Phase 5: Retry & Failure Handling (1 week) - REDUCED from 1-2 weeks
- Implement BuildRun failure classification (map Shipwright reasons to PERMANENT vs RETRYABLE)
  - PERMANENT: BuildStrategyNotFound, InvalidImage, InvalidDockerfile
  - RETRYABLE: ImagePushFailed, GitRemoteFailed, timeout errors
- Implement exponential backoff calculation in broker (already generic)
- Add stale claim detection and recovery (already generic)
- Test failure scenarios with Shipwright builds
- Validate broker retry logic with BuildRun failures

**Deliverables:**
- BuildRun failure reason mapper
- Retry decision logic for Shipwright failures
- Test suite for failure scenarios
- Documentation of retry behavior

### Phase 6: Testing & Evaluation (2 weeks) - CHANGED focus
- Unit tests for ephemeral work system and Shipwright integration
- Integration tests with real Shipwright builds in kind cluster
- End-to-end testing: broker → agent → Shipwright → registry
- Performance testing: concurrent builds, multiple agents, Shipwright/Tekton resource usage
- Test advanced Shipwright features: multi-arch builds, vulnerability scanning
- **Evaluation checkpoint**: Does Shipwright meet 80%+ of requirements?
  - Feature completeness assessment
  - Performance and resource usage analysis
  - Operational complexity evaluation
  - Decision point: continue with Shipwright OR plan custom buildah-operator
- Documentation: architecture decision record, deployment guide, troubleshooting

**Deliverables:**
- Comprehensive test suite
- Performance benchmarks
- Evaluation report with Shipwright assessment
- ADR documenting Shipwright decision and evaluation results
- Complete documentation for Shipwright integration

**Total Estimated Timeline**: 8-10 weeks (down from 11-15 weeks)

**Time Savings**: 5-6 weeks (40% reduction) by leveraging Shipwright

### Dependencies Between Phases
- Phase 2 depends on Phase 1 (needs ephemeral work system)
- Phase 3 depends on Phase 2 (needs agent ephemeral work integration)
- Phase 4 can be developed in parallel with Phase 3 (documentation work)
- Phase 5 depends on Phase 3 (needs Shipwright integration for failure mapping)
- Phase 6 depends on Phases 1-5 (full system evaluation)

### Evaluation Criteria (Phase 6)
At the end of Phase 6, evaluate Shipwright against these criteria:
- **Feature Coverage**: Does Shipwright support 80%+ of build requirements?
- **Performance**: Are build times and resource usage acceptable?
- **Operational Complexity**: Is Tekton + Shipwright manageable in production?
- **Limitations**: What use cases does Shipwright not support well?
- **Decision**: Continue with Shipwright OR design custom buildah-operator (new initiative)

### Post-Evaluation Options
**If Shipwright meets requirements (expected):**
- Move to next ephemeral work type (test execution, backup operations)
- Enhance Shipwright integration with advanced features
- Document Shipwright as standard Brokkr build solution

**If Shipwright has critical limitations (fallback):**
- Create new initiative: Custom BuildRequest + buildah operator
- Preserve generic ephemeral work system (reusable regardless)
- Leverage learnings from Shipwright evaluation
