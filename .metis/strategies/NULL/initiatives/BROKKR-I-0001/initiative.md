---
id: work-system-with
level: initiative
title: "Work System with Shipwright Build Integration"
short_code: "BROKKR-I-0001"
created_at: 2025-10-08T14:59:07.902259+00:00
updated_at: 2025-10-22T14:45:23.097831+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: work-system-with
---

# Work System with Shipwright Build Integration

## Context **\[REQUIRED\]**

Brokkr currently provides environment-aware control plane functionality for distributing Kubernetes objects across clusters via agent/broker architecture. We wish to extend our current capabilities with: 

1. **Native container image building**: Users must rely on external CI/CD systems for image builds before deploying through Brokkr
2. **Generic work management**: No system exists for one-time, non-persistent operations (builds, tests, backups, etc.)

This initiative addresses both needs by:

- Creating a generic "work" system for managing transient operations separate from persistent deployment state
- Integrating Shipwright Build (CNCF Sandbox project) as the first work type for production-ready container builds
- Leveraging existing agent/broker work distribution patterns (matching stacks/deployment objects model)
- Completing the deployment pipeline within Brokkr's architecture
- Establishing patterns for future work types (test runs, backup operations, migrations, etc.)

**Architectural Decisions**:

1. **Shipwright Integration**: Rather than building a custom buildah operator from scratch, this initiative adopts a hybrid approach using Shipwright Build, a mature CNCF project with v1beta1 API stability. Shipwright provides production-ready build capabilities (multi-arch, vulnerability scanning, comprehensive retry logic) while reducing implementation time by 40% and eliminating long-term maintenance burden.

2. **CRD-Controller Pattern**: The system uses a Kubernetes-native controller pattern with a Brokkr Work CRD (`brokkr.io/v1alpha1/Work`) that wraps user-provided CRD templates. The Work CRD controller handles retry logic via reconciliation loops, making the system resilient to agent failures and aligned with Kubernetes best practices. The broker becomes a lightweight message queue + audit log rather than a state machine.

3. **Two-Table Database Design**: The database is split into `work_queue` (transient message queue for active work) and `work_execution_log` (permanent audit trail). This separation optimizes queue operations while maintaining complete execution history for monitoring and analytics.

The generic work system remains valuable regardless of build implementation, as it will support future work types beyond builds (tests, backups, migrations, etc.).

## Goals & Non-Goals **\[REQUIRED\]**

**Goals:**

- Design and implement generic work system in broker (reusable for builds, tests, backups, etc.)
- Integrate Shipwright Build as first work type for production-ready container builds
- Support git-based build sources, multi-architecture builds,
- Leverage broker work queue with retry logic (exponential backoff, stale claim detection)
- Enable work targeting via existing label/annotation patterns (matching stack targeting)
- Support registry publishing with proper authentication through Shipwright
- Provide comprehensive failure handling (permanent vs retryable failures, max retries)

**Non-Goals:**

- Replace external CI/CD systems entirely (complement, not replace)
- Build custom buildah operator initially (Shipwright provides this, custom operator remains option if needed)
- Support non-Shipwright build tools in initial implementation
- Implement all possible work types now (builds only, extensible design for future)
- Any functionality outside of image build + push

## Requirements **\[CONDITIONAL: Requirements-Heavy Initiative\]**

### User Requirements

- **User Characteristics**: Kubernetes operators and platform engineers familiar with CRDs and YAML manifests
- **System Functionality**: Native container building integrated with existing Brokkr deployment workflows
- **User Interfaces**: Kubernetes CRD-based interaction, status via existing Brokkr APIs

### System Requirements

- **Functional Requirements**:

  - REQ-001: Broker must provide generic work queue system (reusable across work types)
  - REQ-002: Agent clusters must have Shipwright Build and Tekton installed as prerequisites
  - REQ-003: Agent must create and monitor Work CRD instances that wrap user-provided templates
  - REQ-004: System must support work targeting matching stack pattern (via broker targets table)
  - REQ-005: Built images must be publishable to container registries with secret-based auth via Shipwright
  - REQ-006: Work CRD controller must implement retry logic via reconciliation loops (max retries, exponential backoff)
  - REQ-007: System must distinguish permanent vs retryable failures
  - REQ-008: Work CRD controller must watch wrapped template status and map to Work CRD status

- **Non-Functional Requirements**:

  - NFR-001: Agent implements Work CRD controller pattern with reconciliation loops
  - NFR-002: Builds support both rootless and rootful modes via Shipwright BuildStrategy configuration
  - NFR-003: Work queue must handle concurrent operations from multiple agents (atomic claim operations)
  - NFR-004: Registry and git credentials managed securely via Kubernetes secrets (Shipwright pattern)
  - NFR-005: Completed work moved to execution log after broker receives completion report
  - NFR-006: Shipwright and Tekton installation documented with version requirements (Tekton v0.59+, Shipwright v0.17.0+)

## Use Cases **\[CONDITIONAL: User-Facing Initiative\]**

### Use Case 1: Git-Based Container Build

- **Actor**: Platform Engineer
- **Scenario**:
  1. Engineer submits build work to broker API with Shipwright Build spec (git repository URL, target registry)
  2. Broker creates work item and determines target agents based on labels
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
  2. Broker populates work_targets table with agents matching labels
  3. Only GPU-capable agents can claim this work
  4. Build executes on appropriate infrastructure with GPU access
- **Expected Outcome**: Build executes on correctly-resourced infrastructure

## Architecture **\[CONDITIONAL: Technically Complex Initiative\]**

### Overview

This initiative introduces a generic work system using a Kubernetes-native CRD-controller pattern:

1. **Work Queue (Broker)**: Lightweight message queue + audit log for routing work to agents
2. **Work CRD**: Custom resource (`brokkr.io/v1alpha1/Work`) that wraps user-provided templates (Shipwright Build, etc.)
3. **Work CRD Controller (Agent)**: Kubernetes controller with reconciliation loops handling retry logic and template execution
4. **Shipwright Integration**: First work type using Shipwright Build (v1beta1) for production-ready container builds
5. **Two-Table Database**: Separation of transient queue (`work_queue`) and permanent audit trail (`work_execution_log`)

### Component Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Kubernetes Cluster (Agent)                                      │
│                                                                  │
│  ┌────────────────────────┐         ┌──────────────────────┐    │
│  │  Brokkr Agent Pod      │         │  Work CRD Controller │    │
│  │  (single container)    │         │  (in agent process)  │    │
│  │                        │         │                      │    │
│  │  - Poll broker queue   │─────┐   │  - Reconcile Work    │    │
│  │  - Claim work          │     │   │    CRDs              │    │
│  │  - Create Work CRD     │─────┼──→│  - Apply template    │    │
│  │  - Report completion   │     │   │    (e.g., Build)     │    │
│  │                        │     │   │  - Watch template    │    │
│  └────────────────────────┘     │   │    status            │    │
│              │                  │   │  - Retry logic       │    │
│              │                  │   │  - Update Work CRD   │    │
│              ▼                  │   │    status            │    │
│    ┌──────────────────────┐    │   └──────────────────────┘    │
│    │  Work CRD            │◄───┘             │                  │
│    │  brokkr.io/v1alpha1  │                  ▼                  │
│    │                      │       ┌──────────────────────┐      │
│    │  spec:               │       │  Template CRD        │      │
│    │    workType: build   │       │  (e.g., Shipwright   │      │
│    │    maxRetries: 3     │       │   Build/BuildRun)    │      │
│    │    template: {...}   │◄──────│                      │      │
│    │  status:             │       │  - Tekton executes   │      │
│    │    phase: Running    │       │  - Git clone         │      │
│    │    retries: 1        │       │  - Buildah build     │      │
│    └──────────────────────┘       │  - Registry push     │      │
│                                   └──────────────────────┘      │
│                                                                  │
│                      Kubernetes API                             │
└──────────────────────────────────────────────────────────────────┘
```

**Key Advantages:**

- Kubernetes-native controller pattern with reconciliation loops
- Work CRD handles retry logic, broker becomes lightweight queue
- Resilient to agent restarts (Work CRDs persist in cluster)
- Aligned with Kubernetes best practices (controller-runtime pattern)
- Broker simplified to message routing + audit logging

### Sequence Flow (Work CRD Pattern)

1. **User submits work to broker**

   - POST `/api/v1/work` with template spec (e.g., Shipwright Build YAML) and target labels
   - Broker wraps template in Work CRD spec with metadata (workType, maxRetries, claimTimeout)
   - Broker inserts into `work_queue` table with status=PENDING
   - Broker populates targeting based on label matching

2. **Agent claims work from queue**

   - Agent polls: GET `/api/v1/agents/{id}/work/pending`
   - Broker returns work where agent matches targets and status=PENDING
   - Agent claims via: POST `/api/v1/work/{id}/claim`
   - Broker atomically updates `work_queue`: status=CLAIMED, claimed_by=agent_id, claimed_at=NOW()

3. **Agent creates Work CRD in cluster**

   - Agent creates Work CRD (`brokkr.io/v1alpha1/Work`) in cluster
   - Work CRD spec contains: workType, maxRetries, template (embedded user YAML)
   - Work CRD controller begins reconciliation

4. **Work CRD controller executes template**

   - Controller extracts template from Work CRD spec
   - Controller applies template to cluster (e.g., Shipwright Build + BuildRun)
   - Controller watches template status (e.g., BuildRun.status.conditions)
   - If template execution fails and retries remain:
     - Controller increments Work.status.retries
     - Controller calculates exponential backoff delay
     - Controller re-applies template after delay

5. **Template executes (Shipwright example)**

   - Shipwright controller watches BuildRun, creates Tekton TaskRun
   - Tekton executes buildah: clone git repo, run build, push to registry
   - Shipwright updates BuildRun.status with results

6. **Work CRD controller updates Work status**

   - Controller watches template completion (e.g., BuildRun Succeeded=True/False)
   - Controller updates Work.status.phase (Succeeded/Failed)
   - Controller updates Work.status.message with results (e.g., image digest)
   - Controller updates Work.status.retries with final count

7. **Agent reports completion to broker**

   - Agent watches Work CRD status
   - When Work.status.phase is terminal (Succeeded/Failed):
     - Agent reports: POST `/api/v1/work/{id}/complete` with status, retries, message
   - Broker moves record from `work_queue` to `work_execution_log`
   - `work_execution_log` captures: success boolean, retries_attempted, result_message

8. **Stale claim detection (broker-side)**

   - If agent crashes before creating Work CRD:
     - Broker detects: claimed_at + claim_timeout_seconds < NOW()
     - Broker resets `work_queue` record to status=PENDING
     - Different agent can claim and retry
   - If agent created Work CRD but crashed:
     - Work CRD controller continues execution (CRD persists in cluster)
     - Agent can resume watching Work CRD on restart

## Detailed Design **\[REQUIRED\]**

### Broker Database Schema

Two-table design separating transient queue operations from permanent audit logging:

```sql
-- work_queue: Transient message queue for active work routing
CREATE TABLE work_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    work_type VARCHAR(50) NOT NULL,  -- 'build', 'test', etc.
    crd_spec TEXT NOT NULL,  -- Work CRD YAML (contains template + metadata)

    -- Queue state (PENDING or CLAIMED only)
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    claimed_by UUID REFERENCES agents(id),
    claimed_at TIMESTAMP WITH TIME ZONE,
    claim_timeout_seconds INTEGER NOT NULL DEFAULT 3600
);

-- work_execution_log: Permanent audit trail of completed work
CREATE TABLE work_execution_log (
    id UUID PRIMARY KEY,  -- Matches work_queue.id
    work_type VARCHAR(50) NOT NULL,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    claimed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE NOT NULL,

    -- Execution details
    claimed_by UUID REFERENCES agents(id),
    success BOOLEAN NOT NULL,
    retries_attempted INTEGER,  -- Reported by agent from Work CRD status
    result_message TEXT,  -- Image digest, error details, etc.

    -- Optional: store CRD spec for debugging (can be omitted to save space)
    crd_spec TEXT
);

-- Work targeting (mirrors stack targeting pattern)
CREATE TABLE work_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_id UUID NOT NULL,  -- References work_queue.id OR work_execution_log.id
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE(work_id, agent_id)
);

-- Indexes for work_queue (optimized for fast queue operations)
CREATE INDEX idx_work_queue_status ON work_queue(status);
CREATE INDEX idx_work_queue_type ON work_queue(work_type);
CREATE INDEX idx_work_queue_claimed_by ON work_queue(claimed_by);
CREATE INDEX idx_work_queue_stale_claims ON work_queue(claimed_at, claim_timeout_seconds)
    WHERE status = 'CLAIMED';

-- Indexes for work_execution_log (optimized for analytics queries)
CREATE INDEX idx_work_execution_log_type ON work_execution_log(work_type);
CREATE INDEX idx_work_execution_log_success ON work_execution_log(success);
CREATE INDEX idx_work_execution_log_completed_at ON work_execution_log(completed_at);
CREATE INDEX idx_work_execution_log_claimed_by ON work_execution_log(claimed_by);

-- Indexes for work_targets
CREATE INDEX idx_work_targets_work ON work_targets(work_id);
CREATE INDEX idx_work_targets_agent ON work_targets(agent_id);
```

**Key Design Decisions:**

- **Two-table separation**: `work_queue` for routing, `work_execution_log` for audit trail
- **Simplified queue states**: Only PENDING and CLAIMED (no retry states in broker)
- **CRD spec contains Work CRD YAML**: Includes template + metadata (maxRetries, workType)
- **Retry logic in Work CRD**: Controller handles retries, not broker database
- **Agent reports retries**: `retries_attempted` populated from Work.status.retries
- **Stale claim detection**: Broker detects timeout, resets to PENDING
- **Move on completion**: Record moves from work_queue to work_execution_log
- **Targeting persists**: work_targets remains for both active and completed work

### Work CRD Specification

The Brokkr Work CRD wraps user-provided templates and manages execution with retry logic:

```yaml
apiVersion: brokkr.io/v1alpha1
kind: Work
metadata:
  name: build-my-app-abc123
  namespace: brokkr-agent
spec:
  workType: build  # Type discriminator for different work handlers
  maxRetries: 3
  claimTimeout: 3600  # Seconds before broker can reclaim
  template:  # User-provided template (embedded YAML)
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
      strategy:
        name: buildah
        kind: ClusterBuildStrategy
      output:
        image: registry.example.com/my-app:latest
        pushSecret: registry-credentials

status:
  phase: Running  # Pending, Running, Succeeded, Failed
  retries: 1
  message: "Image built successfully: sha256:abc123..."
  startTime: "2025-11-02T10:00:00Z"
  completionTime: "2025-11-02T10:05:23Z"
  conditions:
    - type: TemplateApplied
      status: "True"
      lastTransitionTime: "2025-11-02T10:00:05Z"
    - type: TemplateSucceeded
      status: "True"
      reason: BuildCompleted
      message: "BuildRun succeeded"
      lastTransitionTime: "2025-11-02T10:05:23Z"
```

**Work CRD Controller Behavior:**

1. **Reconciliation**: Controller watches Work CRDs, extracts template, applies to cluster
2. **Status Watching**: Controller watches template status (e.g., BuildRun.status.conditions)
3. **Retry Logic**: On failure, if `status.retries < spec.maxRetries`:
   - Increment `status.retries`
   - Calculate exponential backoff: `delay = 2^retries * base_delay`
   - Requeue reconciliation after delay
   - Re-apply template
4. **Completion**: Update `status.phase` (Succeeded/Failed), extract results from template
5. **Agent Integration**: Agent watches Work.status.phase, reports to broker when terminal

### Shipwright Build Specification

Work CRD templates for build work type contain Shipwright Build specifications:

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

Reusable generic work endpoints:

- `POST /api/v1/work` - Create new work item (with target labels)
- `GET /api/v1/agents/{id}/work/pending` - Get claimable work for agent
  - Query params: `?type=<work_type>` (optional, filter by work type: 'build', 'test', etc.)
- `POST /api/v1/work/{id}/claim` - Atomically claim work
- `POST /api/v1/work/{id}/complete` - Report completion/failure
- `GET /api/v1/work/{id}` - Get work item details
- `GET /api/v1/work` - List/query work items
  - Query params: `?type=<work_type>&status=<status>&agent_id=<agent_id>` (all optional filters)
- `DELETE /api/v1/work/{id}` - Cancel/delete work

### Agent Components

**Agent Work Queue Module** (`crates/brokkr-agent/src/work_queue/mod.rs`):

- Poll broker for pending work from work_queue
- Claim work items atomically
- Create Work CRDs in cluster from claimed work
- Watch Work CRD status for completion
- Report completion to broker (moves work_queue → work_execution_log)

**Work CRD Controller** (`crates/brokkr-agent/src/work_crd/controller.rs`):

- Reconciliation loop watching Work CRDs
- Extract template from Work.spec.template
- Apply template to cluster using existing `apply_k8s_objects()`
- Watch template status (e.g., BuildRun.status.conditions)
- Implement retry logic with exponential backoff
- Update Work.status.phase, Work.status.retries, Work.status.message
- Handle different work types via type discriminator

**Work Type Handlers** (`crates/brokkr-agent/src/work_crd/handlers/`):

- `build.rs`: Shipwright Build/BuildRun handler
  - Parse BuildRun status conditions
  - Extract image digest, size, git commit from BuildRun.status
  - Map Shipwright failure reasons to retry decisions
  - Classify permanent vs retryable failures
- `mod.rs`: Handler registry for work type routing

**Agent Configuration** (add to existing Settings):

```toml
[agent.work_queue]
enabled = true
poll_interval_seconds = 30

[agent.work_crd]
namespace = "brokkr-agent"  # Where Work CRDs are created
reconcile_interval_seconds = 10
retry_base_delay_seconds = 60  # Base for exponential backoff

[agent.work_crd.build]
service_account = "builder-sa"  # ServiceAccount for BuildRuns
default_timeout = "15m"
build_retention_succeeded = "1h"
build_retention_failed = "24h"
```

**Deployment Model:**

- Agent remains single-container
- Work CRD controller runs within agent process (no sidecar)
- Shipwright + Tekton installed separately in cluster
- Work CRDs managed by agent's controller implementation

## Testing Strategy **\[CONDITIONAL: Separate Testing Initiative\]**

### Unit Testing

- **Strategy**: Test work queue logic, Shipwright Build/BuildRun creation, status mapping
- **Coverage Target**: 80% coverage for work queue module components
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

## Alternatives Considered **\[REQUIRED\]**

### 1. Shipwright Build Integration (CHOSEN)

**Approach**: Integrate CNCF Shipwright Build (v1beta1) as build execution engine **Chosen Because**:

- Production-ready from day one with v1beta1 API stability
- Feature-rich: multi-arch builds, vulnerability scanning, comprehensive retry logic
- Reduces implementation time by 40% (5-6 weeks saved)
- Eliminates long-term maintenance burden (CNCF/Red Hat/IBM maintained)
- Battle-tested: used by IBM Cloud Code Engine, OpenShift Builds v2
- Generic work system still provides value for future work types (tests, backups, etc.) **Tradeoffs**: Requires Tekton dependency, operational complexity of external components

### 2. Custom BuildRequest + buildah Operator Sidecar

**Approach**: Build custom BuildRequest CRD and buildah operator sidecar from scratch **Rejected Because**:

- 11-15 weeks implementation time vs 8-10 weeks with Shipwright
- Ongoing maintenance burden for custom operator
- Reinventing capabilities that Shipwright already provides
- Feature gap: would need to implement multi-arch, vulnerability scanning, build caching
- Security responsibility for build isolation and secret handling
- Shipwright provides production-ready solution with CNCF backing

### 3. Separate Builder Service

**Approach**: Deploy builder as standalone service separate from agents **Rejected**: Would duplicate work assignment and communication patterns already implemented in agent/broker architecture

### 4. Direct Tekton Integration (no Shipwright)

**Approach**: Use Tekton Pipelines directly with buildah Tasks from Tekton Hub **Rejected**: Lower-level abstraction requires more YAML management, Shipwright provides better build-specific abstractions and features

### 5. Kaniko Executor

**Approach**: Use Kaniko for unprivileged builds without buildah **Rejected**: Kaniko is just an executor image, not a full build system; requires custom orchestration similar to option 2

### 6. Build Results as Deployment Objects

**Approach**: Treat build outputs as deployment objects in existing stack system **Rejected**: Builds are transient operations, not persistent deployment state, requires separate queue management

## Implementation Plan **\[REQUIRED\]**

### Phase 1: Work Queue System (2-3 weeks)

- Create work_queue, work_execution_log, and work_targets tables (migration 07)
- Implement broker DAL for work queue operations
  - Atomic claim operations
  - Stale claim detection (timeout-based)
  - Move work_queue → work_execution_log on completion
- Add generic work queue API endpoints (reusable for all work types)
- Implement work targeting logic (reuse stack targeting patterns)
- Add stale claim detection background job

**Deliverables:**

- Database migration 07 with two-table design
- Broker DAL module with queue + audit log operations
- API endpoints: POST /work, GET /agents/{id}/work/pending, POST /work/{id}/claim, POST /work/{id}/complete
- Work targeting query matching stack pattern
- Stale claim detection background job

**Architecture Changes:**

- Removed retry state tracking from broker (retry_count, next_retry_after, FAILED_RETRYABLE status)
- Simplified to PENDING/CLAIMED queue states
- Added execution log table for permanent audit trail

### Phase 2: Work CRD Definition & Agent Queue Integration (2-3 weeks)

- Define Work CRD schema (`brokkr.io/v1alpha1/Work`) with kube-rs custom resource
  - Spec: workType, maxRetries, claimTimeout, template
  - Status: phase, retries, message, startTime, completionTime, conditions
- Create work_queue module in agent (`crates/brokkr-agent/src/work_queue/`)
  - Poll broker for pending work
  - Claim work atomically
  - Create Work CRDs in cluster
  - Watch Work CRD status for completion
  - Report completion to broker

**Deliverables:**

- Work CRD definition with kube-rs custom resource derive
- Agent work_queue module with broker polling
- Work CRD creation from broker work items
- Work CRD status watching
- Completion reporting (moves work_queue → work_execution_log)

**Architecture Changes:**

- Work CRD becomes central execution primitive
- Agent creates CRDs, controller executes them
- Broker becomes message queue, CRD handles execution

### Phase 3: Work CRD Controller & Build Handler (2-3 weeks)

- Implement Work CRD controller (`crates/brokkr-agent/src/work_crd/controller.rs`)
  - Reconciliation loop using kube-rs runtime
  - Extract template from Work.spec.template
  - Apply template using existing `apply_k8s_objects()`
  - Watch template status
  - Retry logic with exponential backoff
  - Update Work.status.phase, retries, message
- Implement build handler (`crates/brokkr-agent/src/work_crd/handlers/build.rs`)
  - Watch BuildRun status conditions
  - Extract image digest, size, git commit from BuildRun.status
  - Map Shipwright failure reasons to retry decisions
  - Classify permanent vs retryable failures
- Kubernetes client support for Shipwright CRDs (kube-rs custom resources)

**Deliverables:**

- Work CRD controller with reconciliation loop
- Build handler for Shipwright Build/BuildRun templates
- Retry logic in controller (exponential backoff)
- Template status watching framework
- Work.status updates with execution results

**Architecture Changes:**

- Controller handles retry logic, not broker
- Template application uses existing agent functionality
- Work CRD persists across agent restarts

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

### Phase 5: Integration Testing & Failure Scenarios (1-2 weeks)

- Integration tests with real Shipwright builds in kind cluster
- Test Work CRD controller retry logic
  - Verify exponential backoff calculation
  - Verify maxRetries enforcement
  - Verify Work.status.retries tracking
- Test BuildRun failure classification
  - PERMANENT: BuildStrategyNotFound, InvalidImage, InvalidDockerfile
  - RETRYABLE: ImagePushFailed, GitRemoteFailed, timeout errors
- Test stale claim detection and recovery
- Test agent restart scenarios (Work CRD persistence)
- End-to-end testing: broker → agent → Work CRD → BuildRun → registry

**Deliverables:**

- Integration test suite with kind cluster
- Failure scenario test coverage
- Work CRD controller retry validation
- Stale claim recovery validation
- Documentation of retry behavior and failure classification

### Phase 6: Performance Testing & Final Documentation (1-2 weeks)

- Unit tests for work queue system and Work CRD controller
- Performance testing: concurrent builds, multiple agents, Work CRD overhead
- Test advanced Shipwright features: multi-arch builds, vulnerability scanning
- Performance benchmarks: Work CRD reconciliation latency, queue throughput
- Helm chart updates (agent deployment with Work CRD RBAC)
- Complete documentation: deployment guide, troubleshooting, API examples

**Deliverables:**

- Unit test suite (work queue DAL, Work CRD controller, build handler)
- Performance benchmarks and resource usage analysis
- Helm chart with Work CRD and Shipwright support
- Complete documentation (installation, configuration, troubleshooting)
- API documentation with Work CRD examples

**Total Estimated Timeline**: 9-13 weeks (increased from 8-10 due to CRD controller implementation)

### Dependencies Between Phases

- Phase 2 depends on Phase 1 (needs work queue system in broker)
- Phase 3 depends on Phase 2 (needs Work CRD definition and agent queue integration)
- Phase 4 can be developed in parallel with Phase 3 (documentation work)
- Phase 5 depends on Phases 1-3 (needs complete system for integration testing)
- Phase 6 depends on Phases 1-5 (performance testing and final documentation)

**Critical Path:** Phase 1 → Phase 2 → Phase 3 → Phase 5 → Phase 6