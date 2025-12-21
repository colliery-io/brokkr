---
id: remote-agent-log-aggregation-to
level: task
title: "Remote Agent Log Aggregation to Broker"
short_code: "BROKKR-T-0045"
created_at: 2025-12-14T21:09:43.602084+00:00
updated_at: 2025-12-14T21:09:43.602084+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Remote Agent Log Aggregation to Broker

Enable observation of deployed services on remote agents by aggregating workload logs back to the central broker.

## Objective

Implement a log aggregation system that allows operators to observe logs from services deployed by remote agents, even when the broker and agents are on different clusters. Logs should be short-lived (12 hour retention) with automatic hard deletion.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Operators can observe and debug deployed workloads on remote clusters without direct cluster access
- **Business Value**: Reduces time-to-resolution for deployment issues, enables centralized monitoring across multi-cluster deployments
- **Effort Estimate**: L (Large) - touches agent, broker, database, and API layers

## Acceptance Criteria

- [ ] Agent can collect logs from deployed pods in its cluster
- [ ] Agent batches and sends logs to broker via authenticated HTTP endpoint
- [ ] Broker stores logs with 12-hour retention policy
- [ ] Broker automatically hard-deletes logs older than retention period
- [ ] API endpoint allows querying logs by agent, deployment, time range
- [ ] Log volume is bounded (configurable max logs per agent/deployment)
- [ ] Works across clusters with only agent-to-broker egress (existing network model)

---

## Architecture Overview

```
Remote Cluster                              Central Broker
+---------------------------+               +---------------------------+
|  Agent Pod                |               |  Broker                   |
|  +---------------------+  |               |  +---------------------+  |
|  | Agent Container     |  |   HTTPS       |  | Log Ingest API      |  |
|  |  - Log Collector    |--+-------------->|  | POST /agents/{id}/  |  |
|  |  - Batch Buffer     |  |   (PAK auth)  |  |      logs           |  |
|  +---------------------+  |               |  +---------------------+  |
|           |               |               |           |               |
|           v               |               |           v               |
|  +---------------------+  |               |  +---------------------+  |
|  | K8s API             |  |               |  | PostgreSQL          |  |
|  | (pod logs)          |  |               |  | agent_logs table    |  |
|  +---------------------+  |               |  +---------------------+  |
+---------------------------+               |           |               |
                                            |           v               |
                                            |  +---------------------+  |
                                            |  | Cleanup Task        |  |
                                            |  | (12hr retention)    |  |
                                            |  +---------------------+  |
                                            +---------------------------+
```

---

## Database Schema

### New Table: `agent_logs`

```sql
CREATE TABLE agent_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    
    -- Log source identification
    deployment_object_id UUID REFERENCES deployment_objects(id) ON DELETE SET NULL,
    work_order_id UUID REFERENCES work_orders(id) ON DELETE SET NULL,
    
    -- Pod/container identification
    namespace VARCHAR(253) NOT NULL,
    pod_name VARCHAR(253) NOT NULL,
    container_name VARCHAR(63),
    
    -- Log content
    timestamp TIMESTAMPTZ NOT NULL,        -- When log was generated on agent
    level VARCHAR(10),                      -- ERROR, WARN, INFO, DEBUG, TRACE (optional)
    message TEXT NOT NULL,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- When broker received it
);

-- Indexes for common query patterns
CREATE INDEX idx_agent_logs_agent_id ON agent_logs(agent_id);
CREATE INDEX idx_agent_logs_deployment_object_id ON agent_logs(deployment_object_id) 
    WHERE deployment_object_id IS NOT NULL;
CREATE INDEX idx_agent_logs_work_order_id ON agent_logs(work_order_id) 
    WHERE work_order_id IS NOT NULL;
CREATE INDEX idx_agent_logs_namespace_pod ON agent_logs(namespace, pod_name);
CREATE INDEX idx_agent_logs_timestamp ON agent_logs(timestamp);
CREATE INDEX idx_agent_logs_created_at ON agent_logs(created_at);  -- For cleanup queries
```

### Design Decisions

1. **No soft delete** - Logs are ephemeral; hard delete only
2. **No `updated_at`** - Logs are immutable once written
3. **Optional `deployment_object_id`** - Links logs to known deployments when possible
4. **Optional `work_order_id`** - Links logs to work order execution (builds, etc.)
5. **`created_at` vs `timestamp`** - `timestamp` is agent-side time, `created_at` is broker receipt time

---

## Broker API

### POST `/api/v1/agents/{agent_id}/logs`

Bulk ingest logs from an agent.

**Authentication**: Agent PAK (Bearer token)

**Request Body**:
```json
{
  "logs": [
    {
      "timestamp": "2025-01-15T10:30:00.123Z",
      "namespace": "production",
      "pod_name": "my-app-7d4f8b9c5-x2k9p",
      "container_name": "app",
      "level": "INFO",
      "message": "Server started on port 8080",
      "deployment_object_id": "uuid-optional",
      "work_order_id": "uuid-optional"
    }
  ]
}
```

**Response**: `201 Created`
```json
{
  "received": 47,
  "stored": 47
}
```

**Rate Limiting**: Max 1000 logs per request, max 10 requests/minute per agent

### GET `/api/v1/agents/{agent_id}/logs`

Query logs for an agent.

**Authentication**: Admin or Agent PAK

**Query Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `namespace` | string | Filter by namespace |
| `pod_name` | string | Filter by pod name (supports prefix match) |
| `deployment_object_id` | UUID | Filter by deployment |
| `work_order_id` | UUID | Filter by work order |
| `level` | string | Filter by log level (ERROR, WARN, etc.) |
| `since` | ISO8601 | Logs after this timestamp |
| `until` | ISO8601 | Logs before this timestamp |
| `limit` | int | Max results (default 100, max 1000) |
| `offset` | int | Pagination offset |

**Response**: `200 OK`
```json
{
  "logs": [
    {
      "id": "uuid",
      "timestamp": "2025-01-15T10:30:00.123Z",
      "namespace": "production",
      "pod_name": "my-app-7d4f8b9c5-x2k9p",
      "container_name": "app",
      "level": "INFO",
      "message": "Server started on port 8080"
    }
  ],
  "total": 1523,
  "has_more": true
}
```

### GET `/api/v1/deployment-objects/{id}/logs`

Convenience endpoint to get logs for a specific deployment.

### GET `/api/v1/work-orders/{id}/logs`

Convenience endpoint to get logs for a specific work order execution.

---

## Agent Implementation

### Log Collection Strategy

The agent collects logs from pods it has deployed (tracked via `brokkr.io/agent-id` label).

```rust
// Pseudocode for log collection
pub struct LogCollector {
    k8s_client: Client,
    agent_id: Uuid,
    buffer: Vec<LogEntry>,
    buffer_size: usize,        // Default: 100
    flush_interval: Duration,  // Default: 30s
}

impl LogCollector {
    /// Collect logs from pods managed by this agent
    async fn collect_logs(&mut self) -> Result<()> {
        // Find pods with brokkr.io/agent-id label matching our agent
        let pods = self.find_managed_pods().await?;
        
        for pod in pods {
            // Stream logs since last collection
            let logs = self.get_pod_logs(&pod, self.last_collection_time).await?;
            
            for log_line in logs {
                self.buffer.push(LogEntry {
                    timestamp: log_line.timestamp,
                    namespace: pod.namespace.clone(),
                    pod_name: pod.name.clone(),
                    container_name: log_line.container.clone(),
                    message: log_line.message,
                    deployment_object_id: self.get_deployment_id(&pod),
                    level: self.parse_log_level(&log_line.message),
                });
            }
        }
        
        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    /// Send buffered logs to broker
    async fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        let payload = LogBatch { logs: self.buffer.drain(..).collect() };
        self.broker_client.post_logs(payload).await?;
        Ok(())
    }
}
```

### Configuration

New agent configuration options:

```toml
[agent.logging]
# Enable log collection and forwarding
enabled = true

# Maximum logs to buffer before sending
buffer_size = 100

# How often to flush logs to broker (seconds)
flush_interval = 30

# Only collect logs from pods with these labels
# Empty means collect from all managed pods
pod_selector = {}

# Log level filter (only forward logs at this level or above)
# Options: TRACE, DEBUG, INFO, WARN, ERROR
min_level = "INFO"

# Maximum message length (truncate longer messages)
max_message_length = 4096
```

### Helm Chart Updates

```yaml
# charts/brokkr-agent/values.yaml
logging:
  enabled: false  # Disabled by default
  bufferSize: 100
  flushIntervalSeconds: 30
  minLevel: "INFO"
  maxMessageLength: 4096
```

---

## Broker Log Retention

### Background Cleanup Task

```rust
// In broker startup
pub async fn start_log_cleanup_task(dal: DAL, config: LogRetentionConfig) {
    let retention = config.retention_duration;  // Default: 12 hours
    let interval = config.cleanup_interval;     // Default: 1 hour
    
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        
        loop {
            ticker.tick().await;
            
            let cutoff = Utc::now() - retention;
            match dal.agent_logs().delete_before(cutoff).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!("Log cleanup: deleted {} logs older than {}", deleted, cutoff);
                    }
                }
                Err(e) => {
                    error!("Log cleanup failed: {}", e);
                }
            }
        }
    });
}
```

### Configuration

```toml
[broker.log_retention]
# How long to keep logs
retention_hours = 12

# How often to run cleanup
cleanup_interval_minutes = 60

# Maximum logs to delete per cleanup run (prevent long-running deletes)
max_delete_batch = 10000
```

### DAL Method

```rust
impl AgentLogsDAL {
    /// Hard delete all logs created before the given timestamp
    pub fn delete_before(&self, cutoff: DateTime<Utc>) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get()?;
        diesel::delete(agent_logs::table.filter(agent_logs::created_at.lt(cutoff)))
            .execute(conn)
    }
}
```

---

## Implementation Phases

### Phase 1: Database & API Foundation
- [ ] Create database migration for `agent_logs` table
- [ ] Implement `AgentLogsDAL` with CRUD operations
- [ ] Implement `POST /agents/{id}/logs` endpoint
- [ ] Implement `GET /agents/{id}/logs` endpoint
- [ ] Add OpenAPI documentation

### Phase 2: Broker Cleanup Task
- [ ] Add log retention configuration to broker config
- [ ] Implement background cleanup task
- [ ] Add cleanup metrics/logging

### Phase 3: Agent Log Collection
- [ ] Add log collection configuration to agent
- [ ] Implement `LogCollector` component
- [ ] Implement log buffering and batched sending
- [ ] Integrate with main agent loop
- [ ] Update Helm chart with new configuration options

### Phase 4: Convenience APIs & Polish
- [ ] Implement `GET /deployment-objects/{id}/logs`
- [ ] Implement `GET /work-orders/{id}/logs`
- [ ] Add log streaming via SSE (optional, stretch goal)
- [ ] Integration tests

---

## Security Considerations

1. **Authentication**: All log endpoints require valid PAK
2. **Authorization**: Agents can only write logs for themselves; read access controlled by PAK role
3. **Input Validation**: Sanitize log messages to prevent injection attacks
4. **Rate Limiting**: Prevent log flooding with per-agent rate limits
5. **Size Limits**: Enforce max message length and batch size

## Performance Considerations

1. **Bulk Inserts**: Use batch inserts for log ingestion
2. **Index Strategy**: Indexes on common query patterns (agent_id, timestamp, namespace/pod)
3. **Cleanup Batching**: Delete in batches to avoid long-running transactions
4. **Connection Pooling**: Leverage existing Diesel connection pool

## Open Questions

1. Should we support log tailing/streaming via WebSocket or SSE?
2. Should logs be per-tenant in multi-tenant deployments?
3. Do we need log compression for storage efficiency?
4. Should we add Prometheus metrics for log volume?

---

## Status Updates

*To be added during implementation*