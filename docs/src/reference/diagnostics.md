# Diagnostics Reference

Brokkr provides an on-demand diagnostic system for collecting Kubernetes pod information, events, and logs from remote clusters. Administrators request diagnostics through the broker API, and agents collect the data from their local clusters.

## Diagnostic Request Lifecycle

```
                     agent claims                agent submits result
  pending ─────────────────────────→ claimed ──────────────────────────→ completed
     │                                  │
     │ past expires_at,                 │ past expires_at,
     │ never claimed                    │ no result submitted
     ↓                                  ↓
  expired                            failed
```

Every request reaches a terminal state. Which one it reaches records *why* the request ended:

- **`completed`** — an agent submitted a result. This is the outcome even when the collection itself hit an error; see [Collection Errors](#collection-errors).
- **`expired`** — the deadline passed and no agent ever claimed the request. Routine: the target agent is offline, is not polling, or has nothing to say.
- **`failed`** — an agent claimed the request and never came back with a result before the deadline. The agent crashed, was evicted, lost its PAK, or its pod was rescheduled while holding the request.

Both terminal transitions are performed by the broker's expiry sweep, which is the first step of the [automatic cleanup](#automatic-cleanup) task.

### Status Values

| Status | Description | Emitted |
|--------|-------------|---------|
| `pending` | Request created, waiting for agent to claim | Yes — on creation |
| `claimed` | Agent has claimed the request and is collecting data | Yes — on `POST /diagnostics/{id}/claim` |
| `completed` | Agent submitted a diagnostic result, whether or not collection succeeded | Yes — on `POST /diagnostics/{id}/result` |
| `failed` | Agent claimed the request but never submitted a result before `expires_at` | Yes — by the expiry sweep |
| `expired` | Deadline passed with the request still unclaimed | Yes — by the expiry sweep |

`failed` and `expired` are not interchangeable, and the difference is the useful part: `expired` says nobody picked the work up, `failed` says somebody picked it up and did not survive it. A run of `failed` diagnostics for one agent is a signal about that agent's stability; a run of `expired` ones is a signal that it is not connected. `claimed_at` is preserved on a `failed` request, so the agent and the time it accepted the work stay visible.

`failed` requests carry no `DiagnosticResult` — the agent never submitted one. `GET /api/v1/diagnostics/{id}` returns them with `result: null` and a `completed_at` marking when the sweep gave up on the request.

---

## Collection Errors

When an agent claims a request but fails to collect data — for example the Kubernetes API is unreachable, or the agent lacks RBAC permission to list pods, events, or logs — it does not abandon the request. It submits a well-formed result whose `events` array carries a single error object, and the broker transitions the request to `completed` exactly as it would for a successful collection.

A collection error is therefore **not** a `failed` request. The agent never sets `failed` itself; that status only comes from the expiry sweep, and it means the agent stopped answering entirely rather than answering with an error. An agent that reaches the cluster and is told "forbidden" reports that in the result payload below; an agent that dies mid-collection reports nothing and its request is swept to `failed`.

An error-bearing result has this shape:

```json
{
  "request": {
    "id": "diag-uuid",
    "status": "completed",
    "claimed_at": "2025-01-15T10:00:15Z",
    "completed_at": "2025-01-15T10:00:16Z"
  },
  "result": {
    "pod_statuses": "[]",
    "events": "[{\"error\":\"Failed to list pods in namespace default: ApiError: pods is forbidden: User \\\"system:serviceaccount:brokkr:brokkr-agent\\\" cannot list resource \\\"pods\\\"\"}]",
    "log_tails": null,
    "collected_at": "2025-01-15T10:00:16Z"
  }
}
```

The distinguishing marks are:

- `status` is `completed`, not `failed`
- `pod_statuses` is the empty array `[]`
- `events` contains exactly one object, and that object has an `error` key rather than the [event fields](#events)
- `log_tails` is `null`

Note that an empty `pod_statuses` alone does not mean an error occurred: a successful collection that attributed no pods also returns `[]` (see [Pod attribution](#pod-attribution)). The presence of an `error` key inside `events` is what separates the two.

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
  "events": "[{\"event_type\": \"Normal\", \"reason\": \"Pulled\", \"message\": \"Successfully pulled image\", \"involved_object\": \"myapp-abc12\", \"involved_object_kind\": \"Pod\", \"count\": 1}]",
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
| `involved_object` | String | Name of the object the event refers to. Falls back to `unknown` when the source event carries no name. |
| `involved_object_kind` | String? | Kind of the object the event refers to (`Pod`, `ReplicaSet`, …). Absent when the source event omits it. |
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
1. Sweeps requests past their `expires_at` time into a terminal state — still-`pending` requests become `expired`, still-`claimed` requests become `failed`
2. Deletes completed, expired, and failed requests older than `diagnostic_max_age_hours`
3. Deletes associated diagnostic results

Step 1 is what bounds the table: a request that an agent claims and never answers has no other route out of `claimed`, and step 2 only deletes terminal states. Both steps run on the same `diagnostic_cleanup_interval_seconds` tick, so a request can linger past its `expires_at` for up to one interval before its status changes.

---

## Scope of a Collection

### Namespaces

The agent searches the namespaces declared in the deployment object's manifests (`metadata.namespace`, with `default` for documents that omit it) — see `crates/brokkr-agent/src/cli/commands.rs`. Resources the deployment object creates in a namespace it does not declare are not searched.

### Pod attribution

Within those namespaces, `pod_statuses` and `log_tails` cover the pods attributed to the deployment object by `PodAttributor` (`crates/brokkr-agent/src/deployment_health.rs`) — the same resolver continuous health checking uses, so the two always agree. A pod is attributed when, in order:

1. it carries the `brokkr.io/deployment-object-id` **label** (manual opt-in; Brokkr does not add this itself),
2. it carries the same key as an **annotation** — bare `Pod` manifests applied by Brokkr are stamped with it,
3. an object in its **ownerReference chain** carries the annotation, walking up to four hops. This is the case that covers controller-managed workloads: Brokkr stamps the annotation on the top-level applied object, so `Deployment` → `ReplicaSet` → `Pod`, `Job` → `Pod`, and `StatefulSet`/`DaemonSet` → `Pod` all resolve to the object that produced them.

An empty `pod_statuses` therefore means the deployment object genuinely has no pods in the searched namespaces — for example, it applies only non-workload resources (ConfigMap, Service, CRD), or its workload has not created any pods yet.

### Events are namespace-scoped, by design

`events` contains the most recent 50 events in each searched namespace, **not** only the events for the attributed pods. This is deliberate:

- The Kubernetes events API cannot select by the involved object's labels or annotations, so there is no server-side filter equivalent to pod attribution.
- The events that explain a failure are frequently recorded against something other than the pod: `FailedCreate` on the ReplicaSet, `FailedScheduling` for a pod that was never created, or quota, PVC, and node events on the namespace. Narrowing to the attributed pods would discard exactly those.

Read `events` as "what is happening in this deployment object's namespaces", and use `involved_object` / `involved_object_kind` to decide what a given event refers to.

## Known Limitations

- Events are not attributed to the deployment object (see above); in a busy shared namespace they may be dominated by unrelated workloads.
- Attribution walks at most four ownerReference hops (`MAX_OWNER_DEPTH`). Deeper controller chains than `CronJob` → `Job` → `Pod` will not resolve.
- Log tails are capped at the last 100 lines per container (`MAX_LOG_LINES`) and are only available for pods that still exist; a pod that has already been replaced contributes nothing.

---

## Related Documentation

- [How-To: Running On-Demand Diagnostics](../how-to/diagnostics.md) — step-by-step guide
- [Monitoring Deployment Health](../how-to/deployment-health.md) — continuous health monitoring
- [Health Endpoints](./health-endpoints.md) — broker and agent health checks
