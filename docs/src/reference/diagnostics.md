# Diagnostics Reference

Brokkr provides an on-demand diagnostic system for collecting Kubernetes pod information, events, and logs from remote clusters. Administrators request diagnostics through the broker API, and agents collect the data from their local clusters.

## Diagnostic Request Lifecycle

```
Created (pending) → Claimed (by agent) → Result submitted (completed)
                  → Expired (past retention)
                  → Failed (agent error)
```

### Status Values

| Status | Description |
|--------|-------------|
| `pending` | Request created, waiting for agent to claim |
| `claimed` | Agent has claimed the request and is collecting data |
| `completed` | Agent submitted diagnostic results |
| `failed` | Agent encountered an error during collection |
| `expired` | Request exceeded its retention period without completion |

---

## Data Model

### DiagnosticRequest

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `agent_id` | UUID | Target agent to collect from |
| `deployment_object_id` | UUID | Deployment object to diagnose |
| `status` | String | Current status (see above) |
| `requested_by` | String? | Who requested the diagnostic (free-text) |
| `created_at` | DateTime | Request creation time |
| `claimed_at` | DateTime? | When agent claimed the request |
| `completed_at` | DateTime? | When result was submitted |
| `expires_at` | DateTime | When the request expires |

### DiagnosticResult

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `request_id` | UUID | Associated diagnostic request |
| `pod_statuses` | String (JSON) | Pod status information |
| `events` | String (JSON) | Kubernetes events |
| `log_tails` | String? (JSON) | Container log tails (last 100 lines per container) |
| `collected_at` | DateTime | When data was collected on the agent |
| `created_at` | DateTime | Record creation time |

---

## API Endpoints

### Create Diagnostic Request

```
POST /api/v1/deployment-objects/{deployment_object_id}/diagnostics
```

**Auth:** Admin only.

**Request body:**

```json
{
  "agent_id": "uuid-of-target-agent",
  "requested_by": "oncall-engineer",
  "retention_minutes": 60
}
```

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `agent_id` | UUID | Yes | — | Must be a valid agent |
| `requested_by` | String | No | null | Free-text identifier |
| `retention_minutes` | Integer | No | 60 | 1-1440 (max 24 hours) |

**Response:** `201 Created`

```json
{
  "id": "diag-uuid",
  "agent_id": "agent-uuid",
  "deployment_object_id": "do-uuid",
  "status": "pending",
  "requested_by": "oncall-engineer",
  "created_at": "2025-01-15T10:00:00Z",
  "expires_at": "2025-01-15T11:00:00Z"
}
```

---

### Get Diagnostic

```
GET /api/v1/diagnostics/{id}
```

**Auth:** Admin or the target agent.

**Response:** `200 OK`

If the diagnostic is completed, the response includes the result:

```json
{
  "request": {
    "id": "diag-uuid",
    "status": "completed",
    "claimed_at": "2025-01-15T10:00:15Z",
    "completed_at": "2025-01-15T10:00:20Z"
  },
  "result": {
    "pod_statuses": "[{\"name\": \"myapp-abc12\", \"namespace\": \"default\", \"phase\": \"Running\", ...}]",
    "events": "[{\"event_type\": \"Normal\", \"reason\": \"Pulled\", ...}]",
    "log_tails": "{\"myapp-abc12/myapp\": \"2025-01-15 10:00:00 INFO Starting...\\n...\"}",
    "collected_at": "2025-01-15T10:00:18Z"
  }
}
```

---

### Get Pending Diagnostics (Agent)

```
GET /api/v1/agents/{agent_id}/diagnostics/pending
```

**Auth:** Agent (own ID only).

Returns all `pending` diagnostic requests for the agent.

**Response:** `200 OK` — `DiagnosticRequest[]`

---

### Claim Diagnostic Request

```
POST /api/v1/diagnostics/{id}/claim
```

**Auth:** Agent.

Transitions the request from `pending` to `claimed`. Only one agent can claim a request.

**Response:** `200 OK`

---

### Submit Diagnostic Result

```
POST /api/v1/diagnostics/{id}/result
```

**Auth:** Agent (must have claimed the request).

**Request body:**

```json
{
  "pod_statuses": "[{\"name\": \"myapp-abc12\", \"namespace\": \"default\", \"phase\": \"Running\", \"conditions\": [{\"condition_type\": \"Ready\", \"status\": \"True\"}], \"containers\": [{\"name\": \"myapp\", \"ready\": true, \"restart_count\": 0, \"state\": \"running\"}]}]",
  "events": "[{\"event_type\": \"Normal\", \"reason\": \"Pulled\", \"message\": \"Successfully pulled image\", \"involved_object_kind\": \"Pod\", \"involved_object_name\": \"myapp-abc12\", \"count\": 1}]",
  "log_tails": "{\"myapp-abc12/myapp\": \"2025-01-15 10:00:00 INFO Starting server on :8080\\n2025-01-15 10:00:01 INFO Ready to accept connections\"}",
  "collected_at": "2025-01-15T10:00:18Z"
}
```

**Response:** `201 Created`

---

## Collected Data

### Pod Statuses

Each pod status includes:

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Pod name |
| `namespace` | String | Pod namespace |
| `phase` | String | Pod phase (Running, Pending, Failed, etc.) |
| `conditions` | Array | Pod conditions (Ready, Initialized, etc.) |
| `containers` | Array | Container statuses |

Container status fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Container name |
| `ready` | Boolean | Whether the container is ready |
| `restart_count` | Integer | Number of restarts |
| `state` | String | Current state (running, waiting, terminated) |
| `state_reason` | String? | Reason for waiting/terminated state |
| `state_message` | String? | Message for waiting/terminated state |

### Events

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | String? | Normal or Warning |
| `reason` | String? | Short reason string |
| `message` | String? | Human-readable message |
| `involved_object_kind` | String? | Kind of involved object (Pod, ReplicaSet, etc.) |
| `involved_object_name` | String? | Name of involved object |
| `count` | Integer? | Number of occurrences |
| `first_timestamp` | String? | First occurrence |
| `last_timestamp` | String? | Last occurrence |

### Log Tails

A JSON object mapping `pod-name/container-name` to the last 100 lines of logs:

```json
{
  "myapp-abc12/myapp": "line 1\nline 2\n...",
  "myapp-abc12/sidecar": "line 1\nline 2\n..."
}
```

The maximum log lines collected per container is 100 (configured via `MAX_LOG_LINES`).

---

## Automatic Cleanup

The broker runs a background task that periodically cleans up diagnostic data:

| Setting | Default | Description |
|---------|---------|-------------|
| `broker.diagnostic_cleanup_interval_seconds` | 900 (15 min) | How often cleanup runs |
| `broker.diagnostic_max_age_hours` | 1 | Max age for completed/expired/failed diagnostics |

The cleanup task:
1. Expires pending requests past their `expires_at` time
2. Deletes completed, expired, and failed requests older than `diagnostic_max_age_hours`
3. Deletes associated diagnostic results

---

## Related Documentation

- [How-To: Running On-Demand Diagnostics](../how-to/diagnostics.md) — step-by-step guide
- [Monitoring Deployment Health](../how-to/deployment-health.md) — continuous health monitoring
- [Health Endpoints](./health-endpoints.md) — broker and agent health checks
