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

1. **Shipwright Integration**: Rather than building a custom buildah operator from scratch, this initiative leverages Shipwright Build, a mature CNCF project with v1beta1 API stability. Shipwright provides production-ready build capabilities (multi-arch, vulnerability scanning) and ships with its own controller/operator - we don't need to build one. The agent simply creates Shipwright BuildRun resources and watches their status.

2. **Vendored Shipwright with Opt-Out**: The brokkr-agent helm chart includes Shipwright Build as a vendored dependency (subchart), enabled by default. Users who already have Shipwright installed or don't need build capabilities can disable it via `shipwright.enabled: false` in their values. This provides a batteries-included experience while maintaining flexibility for advanced deployments.

3. **Thin WorkOrder CRD**: The `brokkr.io/v1alpha1/WorkOrder` CRD is a lightweight wrapper that references existing Shipwright Build resources and adds retry policy. It does NOT embed templates - instead it references a Build by name. This avoids YAML-in-YAML complexity and lets users manage Shipwright Builds separately.

4. **Multi-Document YAML Pattern**: Users submit work as multi-doc YAML with the Shipwright Build definition first, then the WorkOrder trigger last. The agent's existing sequential apply behavior ensures the Build exists before the WorkOrder references it.

5. **Broker-Side Retry Logic**: Retry policy lives in the broker's `work_orders` table, not in a CRD controller. When a build fails, the agent reports to the broker, which handles retry scheduling. This keeps the agent simple and centralizes retry logic.

6. **Two-Table Database Design**: The database uses `work_orders` (active work routing) and `work_order_log` (permanent audit trail). This separation optimizes queue operations while maintaining complete execution history.

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

  - REQ-001: Broker must provide work order system with routing and retry management
  - REQ-002: Agent clusters must have Shipwright Build and Tekton installed as prerequisites
  - REQ-003: Agent must apply multi-doc YAML (Build + WorkOrder) and create BuildRuns
  - REQ-004: System must support work order targeting matching stack pattern (via broker targets table)
  - REQ-005: Built images must be publishable to container registries with secret-based auth via Shipwright
  - REQ-006: Broker must implement retry logic (max retries, exponential backoff, retry scheduling)
  - REQ-007: Agent must watch BuildRun status and report success/failure to broker
  - REQ-008: WorkOrder CRD must reference Shipwright Build by name (not embed templates)

- **Non-Functional Requirements**:

  - NFR-001: Agent is a thin translation layer (no controller, no reconciliation loops)
  - NFR-002: Builds support both rootless and rootful modes via Shipwright BuildStrategy configuration
  - NFR-003: Work order queue must handle concurrent operations from multiple agents (atomic claim operations)
  - NFR-004: Registry and git credentials managed securely via Kubernetes secrets (Shipwright pattern)
  - NFR-005: Completed work orders moved to log after broker receives completion report
  - NFR-006: Shipwright and Tekton installation documented with version requirements (Tekton v0.59+, Shipwright v0.17.0+)
  - NFR-007: Multi-doc YAML must be applied sequentially (Build before WorkOrder)

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

This initiative introduces a generic work system with a simplified architecture that leverages Shipwright's existing operator:

1. **Work Orders (Broker)**: Two-table design with `work_orders` for routing and `work_order_log` for audit trail
2. **WorkOrder CRD**: Thin custom resource (`brokkr.io/v1alpha1/WorkOrder`) that references Shipwright Builds and adds retry policy
3. **Agent as Translation Layer**: Agent claims work from broker, applies multi-doc YAML (Build + WorkOrder), creates BuildRun, watches status, reports back
4. **Shipwright Operator**: Handles all build execution - we don't build a controller, we use theirs
5. **Broker-Side Retries**: Retry logic centralized in broker, agent just reports success/failure

### Component Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│  Broker                                                             │
│  ┌─────────────────┐  ┌──────────────────┐  ┌───────────────────┐  │
│  │  work_orders    │  │ work_order_log   │  │ work_order_targets│  │
│  │  (active queue) │  │ (audit trail)    │  │ (agent routing)   │  │
│  └─────────────────┘  └──────────────────┘  └───────────────────┘  │
│           │                    ▲                                    │
│           │ claim              │ complete                           │
│           ▼                    │                                    │
└─────────────────────────────────────────────────────────────────────┘
                    │                    ▲
                    │ poll/claim         │ report
                    ▼                    │
┌─────────────────────────────────────────────────────────────────────┐
│  Kubernetes Cluster (Agent)                                         │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Brokkr Agent (thin translation layer)                      │   │
│  │  1. Poll broker for pending work                            │   │
│  │  2. Claim work order                                        │   │
│  │  3. Apply multi-doc YAML (Build + WorkOrder) sequentially   │   │
│  │  4. Create BuildRun from WorkOrder.spec.buildRef            │   │
│  │  5. Watch BuildRun.status until terminal                    │   │
│  │  6. Report success/failure to broker                        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│           │                              ▲                          │
│           │ apply                        │ watch status             │
│           ▼                              │                          │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐    │
│  │ Build (SW)     │  │ WorkOrder      │  │ BuildRun (SW)      │    │
│  │ (template)     │  │ (trigger)      │  │ (execution)        │    │
│  │                │  │                │  │                    │    │
│  │ - git source   │◄─│ - buildRef     │  │ - created by agent │    │
│  │ - strategy     │  │ - retryPolicy  │  │ - watched by agent │    │
│  │ - output       │  │ - serviceAcct  │  │                    │    │
│  └────────────────┘  └────────────────┘  └────────────────────┘    │
│                                                   │                 │
│                                                   ▼                 │
│                                          ┌────────────────┐         │
│                                          │ Shipwright     │         │
│                                          │ Controller     │         │
│                                          │ (handles all   │         │
│                                          │ build logic)   │         │
│                                          └────────────────┘         │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Advantages:**

- No custom controller needed - Shipwright operator handles build execution
- Agent is a thin translation layer, not a complex controller
- Broker-side retry logic keeps agent simple and stateless
- Multi-doc YAML leverages existing agent apply behavior (sequential, waits between objects)
- WorkOrder references Build by name - no YAML-in-YAML embedding

### Sequence Flow

1. **User submits work to broker**

   - POST `/api/v1/work-orders` with multi-doc YAML (Build definition + WorkOrder trigger) and target labels
   - Broker stores YAML content in `work_orders` table with status=PENDING
   - Broker populates `work_order_targets` based on label matching

2. **Agent claims work order**

   - Agent polls: GET `/api/v1/agents/{id}/work-orders/pending`
   - Broker returns work orders where agent matches targets and status=PENDING
   - Agent claims via: POST `/api/v1/work-orders/{id}/claim`
   - Broker atomically updates: status=CLAIMED, claimed_by=agent_id, claimed_at=NOW()

3. **Agent applies multi-doc YAML**

   - Agent parses multi-doc YAML into ordered list of K8s objects
   - Agent applies sequentially using existing `apply_k8s_objects()`:
     - First: Shipwright Build (the reusable template)
     - Last: WorkOrder (the trigger with retry policy)
   - Each object is applied and awaited before the next (existing behavior)

4. **Agent creates BuildRun**

   - Agent reads WorkOrder.spec.buildRef to get Build name
   - Agent creates Shipwright BuildRun referencing the Build
   - Agent begins watching BuildRun.status

5. **Shipwright executes build**

   - Shipwright controller watches BuildRun, creates Tekton TaskRun
   - Tekton executes buildah: clone git repo, run build, push to registry
   - Shipwright updates BuildRun.status with results (digest, size, git info)

6. **Agent reports completion to broker**

   - Agent watches BuildRun.status.conditions until terminal (Succeeded/Failed)
   - Agent extracts results: image digest, failure details, etc.
   - Agent reports: POST `/api/v1/work-orders/{id}/complete` with status and message
   - Broker moves record from `work_orders` to `work_order_log`

7. **Broker handles retry (on failure)**

   - If agent reports failure and retries remain:
     - Broker increments retry_count in `work_orders`
     - Broker calculates next_retry_after with exponential backoff
     - Broker resets status=PENDING after backoff period
     - Same or different agent can claim and retry
   - If max retries exceeded:
     - Broker moves to `work_order_log` with success=false

8. **Stale claim detection (broker-side)**

   - Background job detects: claimed_at + claim_timeout_seconds < NOW()
   - Broker resets `work_orders` record to status=PENDING
   - Different agent can claim and retry

## Detailed Design **\[REQUIRED\]**

### Broker Database Schema

Two-table design separating active work routing from permanent audit logging:

```sql
-- work_orders: Active work routing and retry management
CREATE TABLE work_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    work_type VARCHAR(50) NOT NULL,  -- 'build', 'test', etc.
    yaml_content TEXT NOT NULL,  -- Multi-doc YAML (Build + WorkOrder)

    -- Queue state
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',  -- PENDING, CLAIMED, RETRY_PENDING
    claimed_by UUID REFERENCES agents(id),
    claimed_at TIMESTAMP WITH TIME ZONE,
    claim_timeout_seconds INTEGER NOT NULL DEFAULT 3600,

    -- Retry management (broker-side)
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_count INTEGER NOT NULL DEFAULT 0,
    backoff_seconds INTEGER NOT NULL DEFAULT 60,
    next_retry_after TIMESTAMP WITH TIME ZONE
);

-- work_order_log: Permanent audit trail of completed work
CREATE TABLE work_order_log (
    id UUID PRIMARY KEY,  -- Matches work_orders.id
    work_type VARCHAR(50) NOT NULL,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    claimed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE NOT NULL,

    -- Execution details
    claimed_by UUID REFERENCES agents(id),
    success BOOLEAN NOT NULL,
    retries_attempted INTEGER NOT NULL DEFAULT 0,
    result_message TEXT,  -- Image digest, error details, etc.

    -- Store YAML for debugging/reconstruction
    yaml_content TEXT
);

-- Work order targeting (mirrors stack targeting pattern)
CREATE TABLE work_order_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_order_id UUID NOT NULL,  -- References work_orders.id
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE(work_order_id, agent_id)
);

-- Indexes for work_orders (optimized for queue operations)
CREATE INDEX idx_work_orders_status ON work_orders(status);
CREATE INDEX idx_work_orders_type ON work_orders(work_type);
CREATE INDEX idx_work_orders_claimed_by ON work_orders(claimed_by);
CREATE INDEX idx_work_orders_stale_claims ON work_orders(claimed_at, claim_timeout_seconds)
    WHERE status = 'CLAIMED';
CREATE INDEX idx_work_orders_retry ON work_orders(next_retry_after)
    WHERE status = 'RETRY_PENDING';

-- Indexes for work_order_log (optimized for analytics)
CREATE INDEX idx_work_order_log_type ON work_order_log(work_type);
CREATE INDEX idx_work_order_log_success ON work_order_log(success);
CREATE INDEX idx_work_order_log_completed_at ON work_order_log(completed_at);

-- Indexes for work_order_targets
CREATE INDEX idx_work_order_targets_order ON work_order_targets(work_order_id);
CREATE INDEX idx_work_order_targets_agent ON work_order_targets(agent_id);
```

**Key Design Decisions:**

- **Two-table separation**: `work_orders` for active routing, `work_order_log` for audit trail
- **Broker-side retry logic**: `max_retries`, `retry_count`, `backoff_seconds`, `next_retry_after` in broker
- **Multi-doc YAML storage**: `yaml_content` stores Build + WorkOrder as submitted
- **Three queue states**: PENDING (ready to claim), CLAIMED (in progress), RETRY_PENDING (waiting for backoff)
- **Stale claim detection**: Background job resets CLAIMED → PENDING on timeout
- **Move on completion**: Record moves from `work_orders` to `work_order_log`

### WorkOrder CRD Specification

The WorkOrder CRD is a thin wrapper that references existing Shipwright Builds and adds retry policy:

```yaml
apiVersion: brokkr.io/v1alpha1
kind: WorkOrder
metadata:
  name: build-my-app-abc123
  namespace: brokkr-agent
spec:
  workType: build  # Type discriminator for different work handlers
  retryPolicy:
    maxRetries: 3
    backoffSeconds: 60
  buildRef:
    name: my-app-build  # References existing Shipwright Build
  serviceAccount: builder-sa  # Optional: override default SA for BuildRun
status:
  phase: Pending  # Pending, Running, Succeeded, Failed
  message: ""
  buildRunName: ""  # Name of created BuildRun (set by agent)
```

**Multi-Document YAML Pattern:**

Users submit work as multi-doc YAML with Build first, WorkOrder last:

```yaml
# Document 1: Shipwright Build (applied first, ensures template exists)
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
---
# Document 2: WorkOrder (applied last, triggers the build)
apiVersion: brokkr.io/v1alpha1
kind: WorkOrder
metadata:
  name: build-my-app-abc123
spec:
  workType: build
  retryPolicy:
    maxRetries: 3
    backoffSeconds: 60
  buildRef:
    name: my-app-build
```

**Agent Behavior (no controller needed):**

1. Agent applies multi-doc YAML sequentially (existing `apply_k8s_objects()` behavior)
2. Agent reads WorkOrder.spec.buildRef to get Build name
3. Agent creates BuildRun referencing the Build
4. Agent watches BuildRun.status until terminal
5. Agent reports success/failure to broker (broker handles retries)

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

Work order management endpoints:

- `POST /api/v1/work-orders` - Create new work order (with multi-doc YAML and target labels)
- `GET /api/v1/work-orders` - List/query work orders
  - Query params: `?type=<work_type>&status=<status>&agent_id=<agent_id>` (all optional)
- `GET /api/v1/work-orders/{id}` - Get work order details
- `DELETE /api/v1/work-orders/{id}` - Cancel/delete work order

Agent-facing endpoints:

- `GET /api/v1/agents/{id}/work-orders/pending` - Get claimable work orders for agent
  - Query params: `?type=<work_type>` (optional, filter by work type)
- `POST /api/v1/work-orders/{id}/claim` - Atomically claim work order
- `POST /api/v1/work-orders/{id}/complete` - Report completion/failure
  - Body: `{ success: bool, message: string }` (e.g., image digest or error details)

Work order log endpoints:

- `GET /api/v1/work-order-log` - Query completed work orders
  - Query params: `?type=<work_type>&success=<bool>&agent_id=<agent_id>&from=<timestamp>&to=<timestamp>`
- `GET /api/v1/work-order-log/{id}` - Get completed work order details

### Agent Components

**Work Order Module** (`crates/brokkr-agent/src/work_orders/mod.rs`):

- Poll broker for pending work orders
- Claim work orders atomically
- Apply multi-doc YAML using existing `apply_k8s_objects()`
- Execute work based on workType (build handler for Shipwright)
- Report completion to broker

**Build Handler** (`crates/brokkr-agent/src/work_orders/build.rs`):

- Read WorkOrder.spec.buildRef to get Build name
- Create Shipwright BuildRun referencing the Build
- Watch BuildRun.status.conditions until terminal (Succeeded/Failed)
- Extract results: image digest, git commit, failure details
- Return success/failure to work order module

**Agent Configuration** (add to existing Settings):

```toml
[agent.work_orders]
enabled = true
poll_interval_seconds = 30
namespace = "brokkr-agent"  # Where WorkOrder CRDs and BuildRuns are created

[agent.work_orders.build]
service_account = "builder-sa"  # ServiceAccount for BuildRuns
default_timeout = "15m"
```

**Deployment Model:**

- Agent remains single-container (no sidecar, no controller)
- Agent is a thin translation layer: broker work order → K8s resources → status back to broker
- Shipwright + Tekton installed separately in cluster (prerequisites)
- Retry logic handled by broker, not agent

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

### Phase 1: Broker Work Order System (2 weeks)

- Create `work_orders`, `work_order_log`, and `work_order_targets` tables (migration 08)
- Implement broker DAL for work order operations
  - Atomic claim operations
  - Retry management (increment count, calculate backoff, reset to PENDING)
  - Stale claim detection (timeout-based)
  - Move `work_orders` → `work_order_log` on completion
- Add work order API endpoints
- Implement work order targeting logic (reuse stack targeting patterns)
- Add background jobs: stale claim detection, retry scheduling

**Deliverables:**

- Database migration with two-table design
- Broker DAL module with queue + audit log operations
- API endpoints: POST /work-orders, GET /agents/{id}/work-orders/pending, POST /work-orders/{id}/claim, POST /work-orders/{id}/complete
- Work order targeting query matching stack pattern
- Background jobs for stale claims and retry scheduling

### Phase 2: WorkOrder CRD & Agent Integration (2 weeks)

- Define WorkOrder CRD schema (`brokkr.io/v1alpha1/WorkOrder`) with kube-rs
  - Spec: workType, retryPolicy, buildRef, serviceAccount
  - Status: phase, message, buildRunName
- Create work_orders module in agent (`crates/brokkr-agent/src/work_orders/`)
  - Poll broker for pending work orders
  - Claim work orders atomically
  - Apply multi-doc YAML using existing `apply_k8s_objects()`
  - Dispatch to work type handlers
  - Report completion to broker

**Deliverables:**

- WorkOrder CRD definition with kube-rs custom resource derive
- Agent work_orders module with broker polling
- Multi-doc YAML parsing and sequential application
- Work type dispatch framework
- Completion reporting to broker

### Phase 3: Build Handler & Shipwright Integration (2 weeks)

- Implement build handler (`crates/brokkr-agent/src/work_orders/build.rs`)
  - Read WorkOrder.spec.buildRef to get Build name
  - Create Shipwright BuildRun referencing the Build
  - Watch BuildRun.status.conditions until terminal
  - Extract results: image digest, git commit, failure details
- Add Shipwright CRD types to agent (Build, BuildRun)
- Update WorkOrder.status with BuildRun name

**Deliverables:**

- Build handler for Shipwright integration
- BuildRun creation and status watching
- Result extraction (digest, git info, errors)
- Shipwright CRD type definitions

### Phase 4: Documentation & Prerequisites (1 week)

- Document Shipwright + Tekton installation requirements
  - Tekton Pipelines v0.59+ installation
  - Shipwright Build v0.17.0+ installation
  - buildah ClusterBuildStrategy installation
- Document builder-sa ServiceAccount setup with RBAC
- Document git and registry secret configuration
- Create example multi-doc YAML for common build scenarios
- Document multi-doc YAML ordering requirements (Build first, WorkOrder last)

**Deliverables:**

- Installation guide for Shipwright + Tekton
- ServiceAccount and RBAC manifests
- Example Build + WorkOrder specifications
- Troubleshooting guide

### Phase 5: Testing & Validation (1-2 weeks)

- Integration tests with real Shipwright builds in kind cluster
- Test broker retry logic
  - Verify exponential backoff calculation
  - Verify max_retries enforcement
  - Verify retry_count tracking
- Test stale claim detection and recovery
- End-to-end testing: broker → agent → Build + WorkOrder → BuildRun → registry
- Test multi-doc YAML ordering validation

**Deliverables:**

- Integration test suite with kind cluster
- Retry logic validation
- Stale claim recovery validation
- End-to-end test coverage

### Phase 6: Helm Charts & Final Polish (1 week)

- Unit tests for work order DAL and agent module
- Helm chart updates for WorkOrder CRD and RBAC
- Add Shipwright Build as vendored subchart dependency in brokkr-agent
  - Configure `shipwright.enabled: true` by default (opt-out pattern)
  - Include Tekton Pipelines as transitive dependency
  - Add values schema for Shipwright configuration passthrough
- OpenAPI documentation for work order endpoints
- Performance benchmarks: queue throughput, build latency

**Deliverables:**

- Unit test suite
- Helm chart updates with vendored Shipwright subchart
- Shipwright opt-out configuration (`shipwright.enabled: false`)
- API documentation
- Performance benchmarks

**Total Estimated Timeline**: 7-9 weeks (reduced from 9-13 due to simplified architecture)

### Dependencies Between Phases

- Phase 2 depends on Phase 1 (needs work order system in broker)
- Phase 3 depends on Phase 2 (needs WorkOrder CRD and agent framework)
- Phase 4 can be developed in parallel with Phase 2-3 (documentation)
- Phase 5 depends on Phases 1-3 (needs complete system)
- Phase 6 depends on Phases 1-5 (final polish)

**Critical Path:** Phase 1 → Phase 2 → Phase 3 → Phase 5 → Phase 6

### Key Simplifications from Original Design

- **No CRD controller**: Agent is a thin translation layer, not a reconciliation controller
- **Broker-side retries**: Retry logic centralized in broker, not distributed to CRD
- **Reference pattern**: WorkOrder references Build by name, no YAML embedding
- **Existing apply behavior**: Leverages sequential multi-doc YAML application already in agent
- **Shipwright operator**: Handles all build execution - we just create BuildRuns and watch status