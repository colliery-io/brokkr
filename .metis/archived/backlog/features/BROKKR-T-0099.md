---
id: deployment-health-monitoring
level: task
title: "Deployment Health Monitoring"
short_code: "BROKKR-T-0099"
created_at: 2025-12-14T21:09:43.602084+00:00
updated_at: 2025-12-29T01:17:02.089641+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Deployment Health Monitoring

Enable operators to observe and diagnose the health of deployments across remote agents through a two-tier system: continuous lightweight health status reporting and on-demand detailed diagnostics.

## Objective

Implement a deployment health monitoring system that:
1. **Continuously reports lightweight health status** - Agents periodically check deployed resources and report summary health (healthy/degraded/failing) to the broker, giving operators visibility into problems
2. **Provides on-demand diagnostics** - When operators see a degraded deployment, they can request detailed diagnostics including pod status, K8s events, and log tails

This enables catching common issues like malformed YAML, ImagePullBackOff, CrashLoopBackOff, and other K8s errors without requiring direct cluster access.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Operators get visibility into deployment health across all clusters and can quickly diagnose failures without direct cluster access
- **Business Value**: Reduces time-to-resolution for deployment issues, catches problems proactively before users report them
- **Effort Estimate**: M (Medium) - simpler than continuous log aggregation; extends existing agent/broker patterns

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Tier 1: Health Status Reporting
- [ ] Agent periodically checks health of deployed resources (pods, deployments)
- [ ] Agent reports per-deployment-object health status to broker with each heartbeat
- [ ] Health status includes: overall status (healthy/degraded/failing), pod counts, detected conditions
- [ ] Broker stores current health status per deployment object
- [ ] API exposes health status for querying by stack, agent, or deployment object
- [ ] Common K8s issues are detected: ImagePullBackOff, CrashLoopBackOff, Pending, CreateContainerConfigError, OOMKilled

### Tier 2: On-Demand Diagnostics
- [ ] Operator can request diagnostics for a specific deployment object
- [ ] Broker queues diagnostic request for agent
- [ ] Agent picks up request on next poll, gathers diagnostic data
- [ ] Diagnostic bundle includes: pod status, container status, recent K8s events, log tail
- [ ] Agent posts diagnostic result back to broker
- [ ] API exposes diagnostic results with short retention (1 hour)
- [ ] Works with existing agent-to-broker egress model (no new network requirements)

---

## Architecture Overview

### Two-Tier Health Monitoring

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           TIER 1: HEALTH STATUS                             │
│                         (Continuous, Lightweight)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Remote Cluster                              Central Broker                 │
│  ┌─────────────────────┐                    ┌─────────────────────┐        │
│  │ Agent               │   Heartbeat +      │ Broker              │        │
│  │ ┌─────────────────┐ │   Health Status    │ ┌─────────────────┐ │        │
│  │ │ Health Checker  │─┼───────────────────>│ │ Status Store    │ │        │
│  │ │ - Pod status    │ │   (every poll)     │ │ (per deploy obj)│ │        │
│  │ │ - Conditions    │ │                    │ └─────────────────┘ │        │
│  │ └─────────────────┘ │                    │         │           │        │
│  │         │           │                    │         ▼           │        │
│  │         ▼           │                    │ ┌─────────────────┐ │        │
│  │ ┌─────────────────┐ │                    │ │ API             │ │        │
│  │ │ K8s API         │ │                    │ │ GET /health     │◄├────┐   │
│  │ │ (pod list)      │ │                    │ └─────────────────┘ │    │   │
│  │ └─────────────────┘ │                    └─────────────────────┘    │   │
│  └─────────────────────┘                                               │   │
│                                                                        │   │
│                                              Operator sees: "degraded" ┘   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         TIER 2: ON-DEMAND DIAGNOSTICS                       │
│                            (Triggered, Detailed)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Operator requests diagnostics for degraded deployment                      │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────────────┐                    ┌─────────────────────┐        │
│  │ Broker              │                    │ Remote Cluster      │        │
│  │ ┌─────────────────┐ │   Agent polls,     │ ┌─────────────────┐ │        │
│  │ │ Diagnostic      │ │   sees request     │ │ Agent           │ │        │
│  │ │ Request Queue   │─┼───────────────────>│ │ - Gather pods   │ │        │
│  │ └─────────────────┘ │                    │ │ - Get events    │ │        │
│  │         ▲           │                    │ │ - Tail logs     │ │        │
│  │         │           │   Diagnostic       │ └────────┬────────┘ │        │
│  │ ┌───────┴─────────┐ │   result posted    │          │          │        │
│  │ │ Diagnostic      │◄┼────────────────────┼──────────┘          │        │
│  │ │ Results Store   │ │                    └─────────────────────┘        │
│  │ │ (1hr retention) │ │                                                   │
│  │ └─────────────────┘ │                                                   │
│  └─────────────────────┘                                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Flow Summary

1. **Continuous**: Agent checks pod health → reports status with heartbeat → broker stores → operator sees dashboard
2. **On-demand**: Operator sees "degraded" → requests diagnostics → agent gathers details → posts result → operator investigates

---

## Database Schema

### Tier 1: Health Status

Health status is tracked per agent+deployment_object combination in a dedicated table. This preserves the immutability of deployment_objects while properly modeling that different agents may have different health for the same deployment.

```sql
CREATE TABLE deployment_health (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    deployment_object_id UUID NOT NULL REFERENCES deployment_objects(id) ON DELETE CASCADE,
    
    -- Health status
    status VARCHAR(20) NOT NULL,              -- healthy, degraded, failing, unknown
    summary JSONB,                             -- structured health data
    
    -- Timing
    checked_at TIMESTAMPTZ NOT NULL,          -- when agent checked health
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One health record per agent+deployment combination
    CONSTRAINT unique_agent_deployment_health UNIQUE (agent_id, deployment_object_id)
);

CREATE INDEX idx_deployment_health_agent ON deployment_health(agent_id);
CREATE INDEX idx_deployment_health_deployment ON deployment_health(deployment_object_id);
CREATE INDEX idx_deployment_health_status ON deployment_health(status);

-- Auto-update timestamp
CREATE TRIGGER update_deployment_health_timestamp
BEFORE UPDATE ON deployment_health
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
```

**Health Summary JSON Structure**:
```json
{
  "pods_ready": 2,
  "pods_total": 3,
  "conditions": ["ImagePullBackOff", "CrashLoopBackOff"],
  "resources": [
    {
      "kind": "Deployment",
      "name": "my-app",
      "namespace": "production",
      "ready": false,
      "message": "1/3 replicas available"
    }
  ]
}
```

### Tier 2: Diagnostic Requests & Results

```sql
CREATE TABLE diagnostic_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    deployment_object_id UUID NOT NULL REFERENCES deployment_objects(id) ON DELETE CASCADE,
    
    -- Request state
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, claimed, completed, failed
    requested_by VARCHAR(255),                       -- operator who requested
    
    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    claimed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL                  -- auto-cleanup after 1 hour
);

CREATE TABLE diagnostic_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES diagnostic_requests(id) ON DELETE CASCADE,
    
    -- Diagnostic data (structured JSON)
    pod_statuses JSONB NOT NULL,      -- pod phase, conditions, container statuses
    events JSONB NOT NULL,            -- recent K8s events
    log_tails JSONB,                  -- optional log snippets per container
    
    -- Metadata
    collected_at TIMESTAMPTZ NOT NULL,  -- when agent gathered the data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_diagnostic_requests_agent_pending 
    ON diagnostic_requests(agent_id) WHERE status = 'pending';
CREATE INDEX idx_diagnostic_requests_expires 
    ON diagnostic_requests(expires_at);
CREATE INDEX idx_diagnostic_results_request 
    ON diagnostic_results(request_id);
```

### Design Decisions

1. **Health status on deployment_objects** - No join needed for common queries; status is part of the deployment object
2. **JSONB for structured data** - Flexible schema for varying K8s resource types and conditions
3. **Short-lived diagnostic requests** - 1 hour expiry, hard delete via cleanup task
4. **Separate results table** - Keeps request metadata separate from potentially large result payloads
5. **No soft delete** - Diagnostics are ephemeral troubleshooting data

---

## Broker API

### Tier 1: Health Status Endpoints

#### PATCH `/api/v1/agents/{agent_id}/health-status`

Agent reports health status for its deployment objects (called with heartbeat).

**Authentication**: Agent PAK (Bearer token)

**Request Body**:
```json
{
  "deployment_objects": [
    {
      "id": "uuid",
      "status": "degraded",
      "summary": {
        "pods_ready": 2,
        "pods_total": 3,
        "conditions": ["ImagePullBackOff"],
        "resources": [
          {
            "kind": "Deployment",
            "name": "my-app",
            "namespace": "production",
            "ready": false,
            "message": "2/3 replicas available"
          }
        ]
      },
      "checked_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

**Response**: `200 OK`

#### GET `/api/v1/deployment-objects/{id}/health`

Get health status for a specific deployment object.

**Response**: `200 OK`
```json
{
  "deployment_object_id": "uuid",
  "status": "degraded",
  "summary": { ... },
  "checked_at": "2025-01-15T10:30:00Z"
}
```

#### GET `/api/v1/stacks/{id}/health`

Get health status for all deployment objects in a stack.

**Response**: `200 OK`
```json
{
  "stack_id": "uuid",
  "overall_status": "degraded",
  "deployment_objects": [
    { "id": "uuid", "status": "healthy", ... },
    { "id": "uuid", "status": "degraded", ... }
  ]
}
```

### Tier 2: Diagnostic Endpoints

#### POST `/api/v1/deployment-objects/{id}/diagnostics`

Request diagnostics for a deployment object. Creates a pending request for the agent.

**Authentication**: Admin PAK

**Response**: `202 Accepted`
```json
{
  "request_id": "uuid",
  "status": "pending",
  "expires_at": "2025-01-15T11:30:00Z"
}
```

#### GET `/api/v1/diagnostics/{request_id}`

Get diagnostic request status and results.

**Response (pending)**: `200 OK`
```json
{
  "request_id": "uuid",
  "status": "pending",
  "created_at": "2025-01-15T10:30:00Z"
}
```

**Response (completed)**: `200 OK`
```json
{
  "request_id": "uuid",
  "status": "completed",
  "result": {
    "collected_at": "2025-01-15T10:31:00Z",
    "pod_statuses": [
      {
        "name": "my-app-7d4f8b9c5-x2k9p",
        "namespace": "production",
        "phase": "Running",
        "conditions": [...],
        "container_statuses": [
          {
            "name": "app",
            "ready": false,
            "state": "waiting",
            "reason": "ImagePullBackOff",
            "message": "Back-off pulling image..."
          }
        ]
      }
    ],
    "events": [
      {
        "type": "Warning",
        "reason": "Failed",
        "message": "Failed to pull image...",
        "first_seen": "2025-01-15T10:00:00Z",
        "last_seen": "2025-01-15T10:30:00Z",
        "count": 15
      }
    ],
    "log_tails": {
      "my-app-7d4f8b9c5-x2k9p/app": [
        "2025-01-15T10:29:55Z Error connecting to database",
        "2025-01-15T10:29:56Z Retrying in 5 seconds..."
      ]
    }
  }
}
```

#### GET `/api/v1/agents/{agent_id}/diagnostics/pending`

Agent polls for pending diagnostic requests.

**Authentication**: Agent PAK

**Response**: `200 OK`
```json
{
  "requests": [
    {
      "id": "uuid",
      "deployment_object_id": "uuid"
    }
  ]
}
```

#### POST `/api/v1/diagnostics/{request_id}/result`

Agent posts diagnostic result.

**Authentication**: Agent PAK

**Request Body**: Diagnostic result payload (pod_statuses, events, log_tails)

---

## Agent Implementation

### Tier 1: Health Checker

The agent periodically checks health of resources it has deployed and reports status with each heartbeat.

```rust
pub struct HealthChecker {
    k8s_client: Client,
    agent_id: Uuid,
}

/// Known problematic conditions to detect
const DEGRADED_CONDITIONS: &[&str] = &[
    "ImagePullBackOff",
    "ErrImagePull",
    "CrashLoopBackOff",
    "CreateContainerConfigError",
    "InvalidImageName",
    "OOMKilled",
];

const PENDING_CONDITIONS: &[&str] = &[
    "Pending",
    "ContainerCreating",
    "PodInitializing",
];

impl HealthChecker {
    /// Check health of all resources for a deployment object
    pub async fn check_deployment_object(&self, deploy_obj: &DeploymentObject) -> HealthStatus {
        let mut status = HealthStatus {
            id: deploy_obj.id,
            status: "healthy".to_string(),
            summary: HealthSummary::default(),
            checked_at: Utc::now(),
        };
        
        // Find pods matching this deployment object's resources
        let pods = self.find_pods_for_deployment(deploy_obj).await?;
        
        status.summary.pods_total = pods.len();
        status.summary.pods_ready = pods.iter().filter(|p| p.is_ready()).count();
        
        // Check each pod for problematic conditions
        for pod in &pods {
            for container_status in pod.container_statuses() {
                if let Some(waiting) = &container_status.state.waiting {
                    if let Some(reason) = &waiting.reason {
                        if DEGRADED_CONDITIONS.contains(&reason.as_str()) {
                            status.summary.conditions.push(reason.clone());
                            status.status = "degraded".to_string();
                        }
                    }
                }
                
                // Check for OOMKilled in last termination
                if let Some(terminated) = &container_status.last_state.terminated {
                    if terminated.reason.as_deref() == Some("OOMKilled") {
                        status.summary.conditions.push("OOMKilled".to_string());
                        status.status = "degraded".to_string();
                    }
                }
            }
        }
        
        // If no pods exist when expected, mark as failing
        if status.summary.pods_total == 0 && deploy_obj.expects_pods() {
            status.status = "failing".to_string();
        }
        
        status
    }
}
```

### Tier 2: Diagnostic Handler

When the agent polls for work, it also checks for pending diagnostic requests.

```rust
pub struct DiagnosticHandler {
    k8s_client: Client,
    broker_client: BrokerClient,
}

impl DiagnosticHandler {
    /// Gather diagnostic data for a deployment object
    pub async fn gather_diagnostics(&self, deploy_obj: &DeploymentObject) -> DiagnosticResult {
        let pods = self.find_pods_for_deployment(deploy_obj).await?;
        
        let mut result = DiagnosticResult {
            collected_at: Utc::now(),
            pod_statuses: vec![],
            events: vec![],
            log_tails: HashMap::new(),
        };
        
        for pod in pods {
            // Capture full pod status
            result.pod_statuses.push(PodStatus::from(&pod));
            
            // Get recent events for this pod
            let events = self.get_pod_events(&pod, Duration::minutes(30)).await?;
            result.events.extend(events);
            
            // Tail logs from each container (last 50 lines)
            for container in pod.containers() {
                let key = format!("{}/{}", pod.name, container.name);
                let logs = self.tail_logs(&pod, &container.name, 50).await?;
                result.log_tails.insert(key, logs);
            }
        }
        
        result
    }
    
    /// Called during agent poll loop
    pub async fn process_pending_requests(&self) -> Result<()> {
        let pending = self.broker_client.get_pending_diagnostics().await?;
        
        for request in pending.requests {
            let deploy_obj = self.get_deployment_object(request.deployment_object_id).await?;
            let result = self.gather_diagnostics(&deploy_obj).await?;
            self.broker_client.post_diagnostic_result(request.id, result).await?;
        }
        
        Ok(())
    }
}
```

### Configuration

```toml
[agent.health]
# Enable health checking (enabled by default)
enabled = true

# How often to check resource health (seconds)
# This runs in the poll loop, so this is "at least every N seconds"
check_interval = 60

[agent.diagnostics]
# Enable on-demand diagnostics (enabled by default)
enabled = true

# Max log lines to tail per container
log_tail_lines = 50

# How far back to look for K8s events (minutes)
event_lookback_minutes = 30
```

### Helm Chart Updates

```yaml
# charts/brokkr-agent/values.yaml
health:
  enabled: true
  checkIntervalSeconds: 60

diagnostics:
  enabled: true
  logTailLines: 50
  eventLookbackMinutes: 30
```

---

## Broker Diagnostic Cleanup

### Background Cleanup Task

Diagnostic requests and results are ephemeral - clean up after 1 hour.

```rust
pub async fn start_diagnostic_cleanup_task(dal: DAL, config: DiagnosticConfig) {
    let interval = config.cleanup_interval;  // Default: 15 minutes
    
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        
        loop {
            ticker.tick().await;
            
            let now = Utc::now();
            
            // Delete expired diagnostic requests (cascades to results)
            match dal.diagnostic_requests().delete_expired(now).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!("Diagnostic cleanup: deleted {} expired requests", deleted);
                    }
                }
                Err(e) => {
                    error!("Diagnostic cleanup failed: {}", e);
                }
            }
        }
    });
}
```

### Configuration

```toml
[broker.diagnostics]
# How long diagnostic requests/results are kept
retention_minutes = 60

# How often to run cleanup
cleanup_interval_minutes = 15
```

### DAL Method

```rust
impl DiagnosticRequestsDAL {
    /// Hard delete all diagnostic requests past their expiry time
    pub fn delete_expired(&self, now: DateTime<Utc>) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get()?;
        diesel::delete(
            diagnostic_requests::table.filter(diagnostic_requests::expires_at.lt(now))
        ).execute(conn)
    }
}
```

---

## Implementation Phases

### Phase 1: Health Status (Tier 1)
- [x] Add deployment_health table (separate table, not columns on deployment_objects)
- [x] Create DeploymentHealthDAL with health status methods
- [x] Implement `PATCH /agents/{id}/health-status` endpoint
- [x] Implement `GET /deployment-objects/{id}/health` endpoint
- [x] Implement `GET /stacks/{id}/health` endpoint
- [x] Add OpenAPI documentation for health endpoints

### Phase 2: Agent Health Checker
- [x] Implement `HealthChecker` component in agent
- [x] Add condition detection logic (ImagePullBackOff, CrashLoopBackOff, etc.)
- [x] Integrate health reporting into agent poll loop
- [x] Add health configuration to agent config
- [x] Update agent Helm chart with health settings

### Phase 3: Diagnostic System (Tier 2)
- [x] Create database migration for diagnostic_requests and diagnostic_results tables
- [x] Implement `DiagnosticRequestsDAL` and `DiagnosticResultsDAL`
- [x] Implement `POST /deployment-objects/{id}/diagnostics` endpoint
- [x] Implement `GET /diagnostics/{id}` endpoint
- [x] Implement `GET /agents/{id}/diagnostics/pending` endpoint
- [x] Implement `POST /diagnostics/{id}/result` endpoint (includes /claim)
- [x] Add OpenAPI documentation for diagnostic endpoints

### Phase 4: Agent Diagnostic Handler
- [x] Implement `DiagnosticHandler` component in agent
- [x] Implement pod status gathering
- [x] Implement K8s event collection
- [x] Implement log tailing
- [x] Integrate diagnostic polling into agent loop
- [x] Add diagnostic configuration to agent config

### Phase 5: Cleanup & Polish
- [x] Implement diagnostic cleanup background task
- [x] Add broker diagnostic configuration
- [x] Integration tests for health status flow
- [x] Integration tests for diagnostic request/response flow
- [ ] Documentation updates

---

## Security Considerations

1. **Authentication**: All endpoints require valid PAK
2. **Authorization**: Agents can only report health/diagnostics for their own deployments
3. **Admin-only diagnostics**: Only admin PAK can request diagnostics (prevents agents from triggering unnecessary work)
4. **Input Validation**: Validate and sanitize all JSONB payloads
5. **Log content sanitization**: Truncate and sanitize log tails to prevent injection

## Performance Considerations

1. **Health status is lightweight**: Just pod counts and condition flags - minimal overhead per heartbeat
2. **Diagnostics are on-demand**: No continuous cost; only pay when investigating issues
3. **JSONB for flexibility**: Avoid schema changes for new K8s resource types
4. **Short retention**: 1-hour diagnostic TTL keeps tables small
5. **Indexed queries**: Health status on deployment_objects enables fast dashboard queries

## Open Questions

1. **Health history**: Should we keep historical health snapshots for trend analysis, or is current-state sufficient?
2. **Multi-tenant**: Should diagnostics be schema-per-tenant like other data?
3. **Work order diagnostics**: Should this system also provide diagnostics for failed work orders (builds)?

---

## Related Documents

- **BROKKR-T-0096**: Event Webhook System - enables alerting on health.degraded/health.failing events

---

## Status Updates

### 2025-12-20: Implementation Complete

**Completed Phases 1-5:**

**Phase 1 - Database & Broker Health Status (Tier 1):**
- Created `deployment_health` table with migration (`10_deployment_health`)
- Implemented `DeploymentHealthDAL` with upsert, batch upsert, and query methods
- Added health status API endpoints in broker

**Phase 2 - Agent Health Checker:**
- Implemented `HealthChecker` component with condition detection
- Integrated health reporting into agent heartbeat
- Added health configuration options

**Phase 3 - Diagnostic System (Tier 2):**
- Created `diagnostic_requests` and `diagnostic_results` tables (`11_diagnostics`)
- Implemented `DiagnosticRequestsDAL` and `DiagnosticResultsDAL`
- Added diagnostic API endpoints: create, get, pending, claim, submit result
- Added OpenAPI documentation for all endpoints

**Phase 4 - Agent Diagnostic Handler:**
- Implemented `DiagnosticsHandler` with pod status, event, and log collection
- Integrated 10-second diagnostic polling into agent main loop
- Added broker client methods for fetching/claiming/submitting diagnostics

**Phase 5 - Cleanup & Polish:**
- Implemented background cleanup task for expired diagnostics
- Added broker configuration for cleanup interval and max age
- Created comprehensive integration tests for DAL and API
- Tests compile successfully (require database for execution)

**Remaining:**
- Documentation updates in user-facing docs
- E2E testing with real Kubernetes cluster