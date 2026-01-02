---
title: "Webhooks Reference"
weight: 6
description: "Complete reference for Brokkr's webhook event notification system"
---

# Webhooks Reference

This reference documents Brokkr's webhook system for receiving real-time event notifications via HTTP callbacks.

## Overview

Webhooks enable external systems to receive notifications when events occur in Brokkr. The system supports:
- Subscription-based event filtering
- Broker or agent-side delivery
- Automatic retries with exponential backoff
- Encrypted URL and authentication storage

## Event Types

### Agent Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `agent.registered` | Agent registered with broker | `agent_id`, `name`, `cluster` |
| `agent.deregistered` | Agent deregistered | `agent_id`, `name` |

### Stack Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `stack.created` | New stack created | `stack_id`, `name`, `created_at` |
| `stack.deleted` | Stack soft-deleted | `stack_id`, `deleted_at` |

### Deployment Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `deployment.created` | New deployment object created | `deployment_object_id`, `stack_id`, `sequence_id` |
| `deployment.applied` | Deployment successfully applied by agent | `deployment_object_id`, `agent_id`, `status` |
| `deployment.failed` | Deployment failed to apply | `deployment_object_id`, `agent_id`, `error` |
| `deployment.deleted` | Deployment object soft-deleted | `deployment_object_id`, `stack_id` |

### Work Order Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `workorder.created` | New work order created | `work_order_id`, `work_type`, `status` |
| `workorder.claimed` | Work order claimed by agent | `work_order_id`, `agent_id`, `claimed_at` |
| `workorder.completed` | Work order completed successfully | `work_order_log_id`, `work_type`, `success`, `result_message` |
| `workorder.failed` | Work order failed | `work_order_log_id`, `work_type`, `success`, `result_message` |

### Wildcard Patterns

| Pattern | Matches |
|---------|---------|
| `agent.*` | All agent events |
| `stack.*` | All stack events |
| `deployment.*` | All deployment events |
| `workorder.*` | All work order events |
| `*` | All events |

## API Reference

### Subscription Endpoints

#### List Subscriptions

```
GET /api/v1/webhooks
Authorization: Bearer <admin_pak>
```

Response:
```json
[
  {
    "id": "uuid",
    "name": "string",
    "has_url": true,
    "has_auth_header": false,
    "event_types": ["deployment.*"],
    "filters": null,
    "target_labels": null,
    "enabled": true,
    "max_retries": 5,
    "timeout_seconds": 30,
    "created_at": "2025-01-02T10:00:00Z",
    "updated_at": "2025-01-02T10:00:00Z",
    "created_by": "admin"
  }
]
```

#### Create Subscription

```
POST /api/v1/webhooks
Authorization: Bearer <admin_pak>
Content-Type: application/json
```

Request body:
```json
{
  "name": "string (required)",
  "url": "string (required, http:// or https://)",
  "auth_header": "string (optional)",
  "event_types": ["string (required, at least one)"],
  "filters": {
    "agent_id": "uuid (optional)",
    "stack_id": "uuid (optional)",
    "labels": {"key": "value"}
  },
  "target_labels": ["string (optional)"],
  "max_retries": 5,
  "timeout_seconds": 30,
  "validate": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Human-readable subscription name |
| `url` | string | required | Webhook endpoint URL (encrypted at rest) |
| `auth_header` | string | null | Authorization header value (encrypted at rest) |
| `event_types` | string[] | required | Event types to subscribe to |
| `filters` | object | null | Filter events by agent/stack/labels |
| `target_labels` | string[] | null | Labels for agent-based delivery |
| `max_retries` | int | 5 | Maximum delivery retry attempts |
| `timeout_seconds` | int | 30 | HTTP request timeout |
| `validate` | bool | false | Send test request on creation |

Response: `201 Created` with subscription object

#### Get Subscription

```
GET /api/v1/webhooks/{id}
Authorization: Bearer <admin_pak>
```

Response: `200 OK` with subscription object

#### Update Subscription

```
PUT /api/v1/webhooks/{id}
Authorization: Bearer <admin_pak>
Content-Type: application/json
```

Request body (all fields optional):
```json
{
  "name": "string",
  "url": "string",
  "auth_header": "string or null",
  "event_types": ["string"],
  "filters": {},
  "target_labels": ["string"] or null,
  "enabled": true,
  "max_retries": 5,
  "timeout_seconds": 30
}
```

Response: `200 OK` with updated subscription object

#### Delete Subscription

```
DELETE /api/v1/webhooks/{id}
Authorization: Bearer <admin_pak>
```

Response: `204 No Content`

#### Test Subscription

```
POST /api/v1/webhooks/{id}/test
Authorization: Bearer <admin_pak>
```

Sends a test event to the webhook endpoint.

Response:
```json
{
  "success": true,
  "status_code": 200,
  "message": "Test delivery successful"
}
```

#### List Event Types

```
GET /api/v1/webhooks/event-types
Authorization: Bearer <admin_pak>
```

Response:
```json
[
  "agent.registered",
  "agent.deregistered",
  "stack.created",
  "stack.deleted",
  "deployment.created",
  "deployment.applied",
  "deployment.failed",
  "deployment.deleted",
  "workorder.created",
  "workorder.claimed",
  "workorder.completed",
  "workorder.failed"
]
```

### Delivery Endpoints

#### List Deliveries

```
GET /api/v1/webhooks/{id}/deliveries
Authorization: Bearer <admin_pak>
```

Query parameters:
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `status` | string | null | Filter by status |
| `limit` | int | 50 | Maximum results |
| `offset` | int | 0 | Pagination offset |

Response:
```json
[
  {
    "id": "uuid",
    "subscription_id": "uuid",
    "event_type": "deployment.applied",
    "event_id": "uuid",
    "payload": "{}",
    "target_labels": null,
    "status": "success",
    "acquired_by": null,
    "acquired_until": null,
    "attempts": 1,
    "last_attempt_at": "2025-01-02T10:00:00Z",
    "next_retry_at": null,
    "last_error": null,
    "created_at": "2025-01-02T10:00:00Z",
    "completed_at": "2025-01-02T10:00:01Z"
  }
]
```

## Delivery Modes

### Broker Delivery (Default)

When `target_labels` is null or empty, the broker delivers webhooks directly:

1. Event occurs and is emitted
2. Broker matches event to subscriptions
3. Broker creates delivery records
4. Background task claims and delivers via HTTP POST
5. Success/failure is recorded

Use for external endpoints accessible from the broker.

### Agent Delivery

When `target_labels` is set, matching agents deliver webhooks:

1. Event occurs and is emitted
2. Broker creates delivery with `target_labels`
3. Agent polls for pending deliveries during heartbeat loop
4. Agent claims deliveries matching its labels
5. Agent delivers via HTTP POST from inside cluster
6. Agent reports result back to broker

Use for in-cluster endpoints (e.g., `*.svc.cluster.local`) that the broker cannot reach.

#### Label Matching

An agent can claim a delivery only if it has **ALL** the specified target labels:

| Subscription Labels | Agent Labels | Can Claim? |
|---------------------|--------------|------------|
| `["env:prod"]` | `["env:prod", "region:us"]` | Yes |
| `["env:prod", "region:us"]` | `["env:prod"]` | No |
| `["env:prod"]` | `["env:staging"]` | No |

## Delivery Status

| Status | Description |
|--------|-------------|
| `pending` | Waiting to be claimed and delivered |
| `acquired` | Claimed by broker or agent, delivery in progress |
| `success` | Successfully delivered (HTTP 2xx) |
| `failed` | Delivery failed, will retry after backoff |
| `dead` | Max retries exceeded, no more attempts |

### State Transitions

```
pending → acquired → success
                  → failed → pending (after backoff)
                          → dead (if max_retries exceeded)
```

### Retry Behavior

- Exponential backoff: 2^attempts seconds (2s, 4s, 8s, 16s...)
- Retryable errors: HTTP 5xx, timeouts, connection failures
- Non-retryable errors: HTTP 4xx (except 429)
- TTL: Acquired deliveries expire after 60 seconds if no result reported

## Webhook Payload Format

### HTTP Headers

```
Content-Type: application/json
X-Brokkr-Event-Type: deployment.applied
X-Brokkr-Delivery-Id: a1b2c3d4-e5f6-7890-abcd-ef1234567890
Authorization: <configured auth_header>
```

### Body Structure

```json
{
  "id": "event-uuid",
  "event_type": "deployment.applied",
  "timestamp": "2025-01-02T10:00:00Z",
  "data": {
    // Event-specific fields
  }
}
```

### Example Payloads

#### deployment.applied

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "deployment.applied",
  "timestamp": "2025-01-02T10:00:00Z",
  "data": {
    "deployment_object_id": "a1b2c3d4-...",
    "agent_id": "e5f6g7h8-...",
    "status": "SUCCESS"
  }
}
```

#### workorder.completed

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "event_type": "workorder.completed",
  "timestamp": "2025-01-02T10:05:00Z",
  "data": {
    "work_order_log_id": "b2c3d4e5-...",
    "work_type": "custom",
    "success": true,
    "result_message": "Applied 3 resources successfully",
    "agent_id": "e5f6g7h8-...",
    "completed_at": "2025-01-02T10:05:00Z"
  }
}
```

#### workorder.failed

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "event_type": "workorder.failed",
  "timestamp": "2025-01-02T10:05:00Z",
  "data": {
    "work_order_log_id": "c3d4e5f6-...",
    "work_type": "build",
    "success": false,
    "result_message": "Build failed: Dockerfile not found",
    "agent_id": "e5f6g7h8-...",
    "completed_at": "2025-01-02T10:05:00Z"
  }
}
```

## Database Schema

### webhook_subscriptions

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `name` | VARCHAR(255) | Subscription name |
| `url_encrypted` | BYTEA | Encrypted webhook URL |
| `auth_header_encrypted` | BYTEA | Encrypted auth header (nullable) |
| `event_types` | TEXT[] | Event type patterns |
| `filters` | TEXT | JSON-encoded filters (nullable) |
| `target_labels` | TEXT[] | Labels for agent delivery (nullable) |
| `enabled` | BOOLEAN | Whether subscription is active |
| `max_retries` | INT | Max delivery attempts |
| `timeout_seconds` | INT | HTTP timeout |
| `created_at` | TIMESTAMP | Creation timestamp |
| `updated_at` | TIMESTAMP | Last update timestamp |
| `created_by` | VARCHAR(255) | Creator identifier |

### webhook_deliveries

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `subscription_id` | UUID | Foreign key to subscription |
| `event_type` | VARCHAR(100) | Event type |
| `event_id` | UUID | Idempotency key |
| `payload` | TEXT | JSON event payload |
| `target_labels` | TEXT[] | Copied from subscription |
| `status` | VARCHAR(20) | Delivery status |
| `acquired_by` | UUID | Agent ID (nullable, NULL = broker) |
| `acquired_until` | TIMESTAMP | TTL for claim |
| `attempts` | INT | Number of attempts |
| `last_attempt_at` | TIMESTAMP | Last attempt time |
| `next_retry_at` | TIMESTAMP | Next retry time |
| `last_error` | TEXT | Error from last attempt |
| `created_at` | TIMESTAMP | Creation timestamp |
| `completed_at` | TIMESTAMP | Completion timestamp |

## Security Considerations

- **URL and auth header encryption**: Stored encrypted at rest using AES-256-GCM
- **Admin-only access**: All webhook endpoints require admin PAK authentication
- **Agent authentication**: Agents use their PAK to fetch and report deliveries
- **TLS recommended**: Use HTTPS endpoints in production
- **Secret rotation**: Rotate auth headers by updating the subscription

## Performance Characteristics

### Broker Delivery

- Background task polls every 5 seconds
- Batch size: 10 deliveries per cycle
- Concurrent delivery: single-threaded per broker instance

### Agent Delivery

- Polling interval: 10 seconds
- Batch size: 10 deliveries per poll
- Concurrent delivery: single-threaded per agent
- TTL: 60 seconds for acquired deliveries

### Scaling Considerations

- Multiple broker instances share the delivery workload
- Agent delivery scales with number of matching agents
- Delivery latency: typically < 15 seconds from event to delivery

## Related Documentation

- [How to Configure Webhooks](/how-to/webhooks) - Step-by-step setup guide
- [Architecture](/explanation/architecture) - System architecture overview
- [Data Flows](/explanation/data-flows) - Event flow through the system
