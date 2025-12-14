---
title: "Work Orders"
description: "Reference documentation for Brokkr work orders"
weight: 4
---

# Work Orders

Work orders are transient operations that Brokkr routes to agents for execution. Unlike deployment objects which represent persistent desired state, work orders are one-time operations such as container builds, tests, or backups.

## Concepts

### Work Order vs Deployment Object

| Aspect | Deployment Object | Work Order |
|--------|------------------|------------|
| Purpose | Persistent state | One-time operation |
| Lifecycle | Applied, reconciled, deleted | Created, claimed, completed |
| Examples | Deployments, ConfigMaps | Container builds, tests |
| Storage | Permanent in stack | Moved to log after completion |

### Work Order Lifecycle

```
PENDING -> CLAIMED -> (success) -> work_order_log
                  \-> (failure) -> RETRY_PENDING -> PENDING (retry)
                                \-> work_order_log (max retries)
```

1. **PENDING**: Work order created, waiting for an agent to claim
2. **CLAIMED**: Agent has claimed the work order and is executing
3. **RETRY_PENDING**: Execution failed, waiting for retry backoff
4. **Completed**: Moved to `work_order_log` (success or max retries exceeded)

### Targeting

Work orders are routed to agents using the same targeting mechanisms as stacks:

- **Direct agent IDs**: Route to specific agents by UUID
- **Labels**: Route to agents with matching labels (OR logic)
- **Annotations**: Route to agents with matching annotations (OR logic)

An agent can claim a work order if it matches ANY of the specified targeting criteria.

## Work Types

### Build (`build`)

Container image builds using Shipwright. The `yaml_content` should contain a Shipwright Build specification.

See [Container Builds with Shipwright](../../how-to/shipwright-builds) for details.

## API Reference

### Create Work Order

```bash
POST /api/v1/work-orders
Authorization: Bearer <admin-pak>
Content-Type: application/json

{
  "work_type": "build",
  "yaml_content": "<shipwright-build-yaml>",
  "max_retries": 3,
  "backoff_seconds": 60,
  "claim_timeout_seconds": 3600,
  "targeting": {
    "labels": ["env=dev"],
    "annotations": {"capability": "builder"}
  }
}
```

**Parameters:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `work_type` | string | Yes | - | Type of work (e.g., "build") |
| `yaml_content` | string | Yes | - | YAML content for the work |
| `max_retries` | integer | No | 3 | Maximum retry attempts |
| `backoff_seconds` | integer | No | 60 | Base backoff for exponential retry |
| `claim_timeout_seconds` | integer | No | 3600 | Seconds before claimed work is considered stale |
| `targeting` | object | Yes | - | Targeting configuration |
| `targeting.agent_ids` | array | No | - | Direct agent UUIDs |
| `targeting.labels` | array | No | - | Agent labels to match |
| `targeting.annotations` | object | No | - | Agent annotations to match |

### List Work Orders

```bash
GET /api/v1/work-orders?status=PENDING&work_type=build
Authorization: Bearer <admin-pak>
```

**Query Parameters:**

| Parameter | Description |
|-----------|-------------|
| `status` | Filter by status (PENDING, CLAIMED, RETRY_PENDING) |
| `work_type` | Filter by work type |

### Get Work Order

```bash
GET /api/v1/work-orders/:id
Authorization: Bearer <admin-pak>
```

### Cancel Work Order

```bash
DELETE /api/v1/work-orders/:id
Authorization: Bearer <admin-pak>
```

### Get Pending Work Orders (Agent)

```bash
GET /api/v1/agents/:agent_id/work-orders/pending?work_type=build
Authorization: Bearer <agent-pak>
```

Returns work orders that the agent can claim based on targeting rules.

### Claim Work Order (Agent)

```bash
POST /api/v1/work-orders/:id/claim
Authorization: Bearer <agent-pak>
Content-Type: application/json

{
  "agent_id": "<agent-uuid>"
}
```

Atomically claims the work order. Returns 409 Conflict if already claimed.

### Complete Work Order (Agent)

```bash
POST /api/v1/work-orders/:id/complete
Authorization: Bearer <agent-pak>
Content-Type: application/json

{
  "success": true,
  "message": "sha256:abc123..."
}
```

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether the work completed successfully |
| `message` | string | Result message (image digest on success, error on failure) |

### Work Order Log

Completed work orders are moved to the log for audit purposes.

```bash
# List completed work orders
GET /api/v1/work-order-log?work_type=build&success=true&limit=100
Authorization: Bearer <admin-pak>

# Get specific completed work order
GET /api/v1/work-order-log/:id
Authorization: Bearer <admin-pak>
```

**Query Parameters:**

| Parameter | Description |
|-----------|-------------|
| `work_type` | Filter by work type |
| `success` | Filter by success status (true/false) |
| `agent_id` | Filter by agent that executed |
| `limit` | Maximum results to return |

## Retry Behavior

When a work order fails:

1. Agent reports failure via `/complete` with `success: false`
2. Broker increments `retry_count`
3. If `retry_count < max_retries`:
   - Status set to `RETRY_PENDING`
   - `next_retry_after` calculated with exponential backoff
   - After backoff period, status returns to `PENDING`
4. If `retry_count >= max_retries`:
   - Work order moved to `work_order_log` with `success: false`

**Backoff Formula:**
```
next_retry_after = now + (backoff_seconds * 2^retry_count)
```

## Stale Claim Detection

If an agent claims a work order but doesn't complete it within `claim_timeout_seconds`:

1. Broker background job detects stale claim
2. Work order reset to `PENDING` status
3. Different agent can claim and retry

This handles agent crashes or network partitions.

## Example: Container Build

```bash
# Create a build work order
curl -X POST http://localhost:3000/api/v1/work-orders \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "work_type": "build",
    "yaml_content": "apiVersion: shipwright.io/v1beta1\nkind: Build\nmetadata:\n  name: my-build\nspec:\n  source:\n    type: Git\n    git:\n      url: https://github.com/org/repo\n  strategy:\n    name: buildah\n    kind: ClusterBuildStrategy\n  output:\n    image: ttl.sh/my-image:latest",
    "targeting": {
      "labels": ["capability=builder"]
    }
  }'

# Check status
curl http://localhost:3000/api/v1/work-orders/$WORK_ORDER_ID \
  -H "Authorization: Bearer $ADMIN_PAK"

# View completed builds
curl "http://localhost:3000/api/v1/work-order-log?work_type=build" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

## Database Schema

Work orders use a two-table design:

- **`work_orders`**: Active queue for routing and retry management
- **`work_order_log`**: Permanent audit trail of completed work

This separation optimizes queue operations while maintaining complete execution history.
