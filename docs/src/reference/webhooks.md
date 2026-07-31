# Webhooks Reference

This reference documents Brokkr's webhook system for receiving real-time event notifications via HTTP callbacks.

## Overview

Webhooks enable external systems to receive notifications when events occur in Brokkr. The system supports:
- Subscription by event type, narrowed further by optional payload filters
- Broker or agent-side delivery
- Retries with exponential backoff for transient failures; immediate termination for permanent ones
- Encrypted URL and authentication storage

## Event Types

The `data` object of each delivered payload carries exactly the fields listed below. Fields that are not listed are never present, and the filter rules in [Filters](#filters) depend on this.

### Agent Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `agent.registered` | Agent registered with broker | `agent_id`, `name`, `cluster_name`, `status`, `created_at` |
| `agent.deregistered` | Agent deregistered | `agent_id`, `deleted_at` |

### Stack Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `stack.created` | New stack created | `stack_id`, `name`, `description`, `created_at` |
| `stack.deleted` | Stack soft-deleted | `stack_id`, `deleted_at` |

### Deployment Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `deployment.created` | New deployment object created | `deployment_object_id`, `stack_id`, `sequence_id`, `created_at` |
| `deployment.applied` | Deployment successfully applied by agent | `agent_event_id`, `agent_id`, `deployment_object_id`, `stack_id`, `event_type`, `status`, `message`, `created_at` |
| `deployment.failed` | Deployment failed to apply | `agent_event_id`, `agent_id`, `deployment_object_id`, `stack_id`, `event_type`, `status`, `message`, `created_at` |
| `deployment.deleted` | Deployment object soft-deleted | `deployment_object_id`, `stack_id`, `deleted_at` |

`deployment.applied` and `deployment.failed` are the same payload with a different event type; `status` distinguishes them. Both carry the owning `stack_id` alongside `deployment_object_id`, so a consumer can correlate an apply to its stack without a second API call.

`deployment.applied` and `deployment.failed` emit `"stack_id": null` when the deployment object has been soft-deleted between the apply and the agent reporting it, so the owning stack is no longer resolvable.

`deployment.deleted` emits `"stack_id": null` when the deployment object row is no longer readable at deletion time.

### Work Order Events

| Event Type | Description | Payload Fields |
|------------|-------------|----------------|
| `workorder.created` | New work order created | `work_order_id`, `work_type`, `status`, `created_at` |
| `workorder.claimed` | Work order claimed by agent | `work_order_id`, `work_type`, `agent_id`, `claimed_at` |
| `workorder.completed` | Work order completed successfully | `work_order_log_id`, `work_type`, `success`, `result_message`, `agent_id`, `completed_at` |
| `workorder.failed` | Work order failed | `work_order_log_id`, `work_type`, `success`, `result_message`, `agent_id`, `completed_at` |

`workorder.completed` and `workorder.failed` emit `"agent_id": null` when the work order was never claimed by an agent.

### Wildcard Patterns

| Pattern | Matches |
|---------|---------|
| `agent.*` | All agent events |
| `stack.*` | All stack events |
| `deployment.*` | All deployment events |
| `workorder.*` | All work order events |
| `*` | All events |

## Filters

A subscription's optional `filters` object narrows an event-type match by comparing values carried in the event payload. Filters are evaluated when the event is emitted, before any delivery record is created, so a filtered-out event produces no delivery and appears nowhere in the delivery list.

### Shape

| Field | Type | Description |
|-------|------|-------------|
| `agent_id` | uuid | Deliver only events whose payload `agent_id` equals this value |
| `stack_id` | uuid | Deliver only events whose payload `stack_id` equals this value |

These are the only recognized filter fields. Label-based routing is `target_labels`, which is a separate delivery-mode setting and is not part of `filters`.

### Semantics

- A `filters` object that is absent, `null`, or empty (`{}`) matches every event of a subscribed type.
- Every field that is set must match. Setting both `agent_id` and `stack_id` requires the payload to carry both, with both values equal.
- **An event that does not carry a filtered field never matches.** A subscription filtering on `stack_id` receives nothing from event types whose payload has no `stack_id`, and one filtering on `agent_id` receives nothing from event types whose payload has no `agent_id`. Filters narrow; they never widen to "or the event did not say."
- A JSON `null` in the payload counts as absent, not as a value. A `stack_id` filter therefore excludes a `deployment.deleted`, `deployment.applied`, or `deployment.failed` event that emitted `"stack_id": null`, and an `agent_id` filter excludes a `workorder.completed` or `workorder.failed` event for an unclaimed work order.
- A stored filter that cannot be parsed as JSON **fails closed**: the subscription receives nothing at all, and the broker logs an error naming the subscription for each excluded event. Nothing in the API can write such a value; it indicates a hand-edited or externally written row.

### Filter Fields by Event Type

Which filter field is usable depends on what each event payload carries:

| Event Type | `agent_id` filter | `stack_id` filter |
|------------|-------------------|-------------------|
| `agent.registered` | Usable | Never matches |
| `agent.deregistered` | Usable | Never matches |
| `stack.created` | Never matches | Usable |
| `stack.deleted` | Never matches | Usable |
| `deployment.created` | Never matches | Usable |
| `deployment.applied` | Usable | Usable, except when the payload's `stack_id` is `null` |
| `deployment.failed` | Usable | Usable, except when the payload's `stack_id` is `null` |
| `deployment.deleted` | Never matches | Usable, except when the payload's `stack_id` is `null` |
| `workorder.created` | Never matches | Never matches |
| `workorder.claimed` | Usable | Never matches |
| `workorder.completed` | Usable, except when the payload's `agent_id` is `null` | Never matches |
| `workorder.failed` | Usable, except when the payload's `agent_id` is `null` | Never matches |

"Never matches" means the subscription receives no deliveries for that event type while that filter field is set. Because `workorder.created` carries neither field, any filter at all excludes it.

A subscription combining a wildcard such as `deployment.*` with a `stack_id` filter therefore receives the whole deployment lifecycle for that stack — `deployment.created`, `deployment.applied`, `deployment.failed`, and `deployment.deleted`. The only gap is an event whose payload `stack_id` is `null` because the deployment object was no longer resolvable when the event was emitted.

### Removed Fields

Two fields that earlier versions accepted have been removed. Sending either one now returns `422 Unprocessable Entity`, with a message naming the removed field and its replacement.

| Removed field | Endpoints | Replacement |
|---------------|-----------|-------------|
| `filters.labels` | `POST /api/v1/webhooks`, `PUT /api/v1/webhooks/{id}` | `target_labels`, which performs label-based delivery routing |
| `validate` | `POST /api/v1/webhooks` | [`POST /api/v1/webhooks/{id}/test`](#test-subscription), which sends the test request that `validate` implied |

Neither field ever affected delivery: `filters.labels` was stored but never evaluated, and `validate` was parsed but never acted on. Rejecting them makes that visible rather than silent.

The rejection is machine-readable, so a client can detect and repair it without parsing prose:

```json
{
  "code": "unsupported_field",
  "details": { "field": "filters.labels", "use_instead": "target_labels" }
}
```

`field` is `validate` or `filters.labels`; `use_instead` is `POST /webhooks/{id}/test` or `target_labels` respectively. A key counts as present even when its value is `null` — writing the key down at all is taken to mean the caller expects it to do something.

Note that `validate` is rejected on creation only. It was never accepted on updates, so on `PUT` it is an unrecognised key like any other and is ignored.

**This is a breaking API change.** A client that still sends either field receives a 422 where it previously received a `201`/`200`. To migrate:

- Remove `labels` from the `filters` object. If it expressed the delivery target, set `target_labels` instead. If it expressed an agent or stack scope, use the `agent_id` or `stack_id` filter fields.
- Remove `validate` from creation requests. Call the test endpoint after creating the subscription if you want the same check.

The rejection applies to the **write path only**. Subscriptions already stored with a legacy `labels` key keep working and keep delivering: the key is ignored when the filter is read, so a filter containing nothing else matches every event of its subscribed types. Such a subscription's `filters` object comes back from the API without the `labels` key, showing exactly what is evaluated. No action is required unless you send a write that includes the removed field — including a `PUT` that echoes back a filter object read from the API, which must have `labels` stripped first.

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
    "stack_id": "uuid (optional)"
  },
  "target_labels": ["string (optional)"],
  "max_retries": 5,
  "timeout_seconds": 30
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Human-readable subscription name, 1-255 characters |
| `url` | string | required | Webhook endpoint URL (encrypted at rest) |
| `auth_header` | string | null | Authorization header value (encrypted at rest) |
| `event_types` | string[] | required | Event types to subscribe to, at least one |
| `filters` | object | null | Payload filters; see [Filters](#filters) |
| `target_labels` | string[] | null | Labels for agent-based delivery |
| `max_retries` | int | 5 | Total delivery attempts allowed, 0-10; see [Retry Behavior](#retry-behavior) |
| `timeout_seconds` | int | 30 | Per-request HTTP timeout in seconds, 1-300 |

Values outside the accepted range for `max_retries` or `timeout_seconds` are rejected with `422 Unprocessable Entity`.

The removed `validate` field and `filters.labels` are also rejected with `422 Unprocessable Entity`. See [Removed Fields](#removed-fields).

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

Omitted fields are left unchanged. Setting `filters` to `null` clears the filters; setting it to an object replaces them entirely. A `filters` object containing `labels` is rejected; see [Removed Fields](#removed-fields).

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

Sends a test event to the webhook endpoint, using the subscription's own `timeout_seconds`. The test event has event type `webhook.test` and a `data` object containing `message` and `subscription_id`. It is sent immediately from the broker and creates no delivery record, so it does not appear in the delivery list and is never retried.

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
3. Agent polls for pending deliveries on its own fixed 10-second webhook interval, independent of the heartbeat and reconciliation loops
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
| `failed` | Retryable failure, will retry after backoff |
| `dead` | Terminal: attempt budget exhausted, or a permanent failure that no retry can fix |

### State Transitions

```
pending → acquired → success
                  → failed → pending (after backoff)
                          → dead (attempt budget exhausted)
                  → dead (non-retryable response, on the first attempt)
```

### Retry Classification

Failures are classified by the HTTP status the endpoint returned. Both delivery paths — the broker's delivery worker and results reported by agents — apply the same policy.

| Outcome | Classification |
|---------|----------------|
| HTTP 2xx | Success |
| HTTP 408, 425, 429 | Retryable |
| Any HTTP 5xx | Retryable |
| Any other HTTP 4xx (400, 401, 403, 404, 422, ...) | **Terminal** |
| Transport error (connection refused, DNS failure, TLS failure) | Retryable |
| Timeout | Retryable |
| Any other status (for example an unfollowed 3xx) | Retryable |

A terminal failure sends the delivery to `dead` on the **first** attempt, regardless of remaining attempt budget, with the status recorded in `last_error` (for example `HTTP 404 Not Found: ...`). An endpoint that returns 404 or 401 therefore stops consuming attempts immediately rather than retrying to exhaustion.

A subscription whose encrypted URL or auth header cannot be decrypted is also terminal; see [Encryption at Rest](#encryption-at-rest).

### Retry Behavior

- `max_retries` is a **total-attempt** cap, not a count of retries after the first attempt. The default of 5 permits 5 attempts: 1 initial plus 4 retries. Setting it to `0` sends any retryable failure straight to `dead`.
- Exponential backoff between retries: 2^attempts seconds (2s, 4s, 8s, 16s...).
- The delivery moves to `dead` once its attempt count reaches `max_retries`.
- TTL: acquired deliveries expire after 60 seconds if no result is reported, and return to `pending` for another attempt.

### Timeouts

`timeout_seconds` is applied per request, to that subscription's deliveries only. Every path honors it: broker delivery, agent delivery, and the test endpoint. There is no broker-wide ceiling below it, so a subscription configured above 30 seconds genuinely waits that long before the attempt is treated as a timeout.

- Accepted range: 1 to 300 seconds; the default is 30.
- Values are clamped to a floor of 1 second at delivery time, which affects only rows written before the range was enforced.
- A timeout is a retryable failure and consumes one attempt.

Because the broker's delivery worker processes a batch sequentially, a long `timeout_seconds` on a slow endpoint delays other deliveries in the same batch.

## Webhook Payload Format

### HTTP Headers

Broker-delivered webhooks send:

```
Content-Type: application/json
Authorization: <configured auth_header>
```

Agent-delivered webhooks (subscriptions with `target_labels`) additionally include:

```
X-Brokkr-Event-Type: deployment.applied
X-Brokkr-Delivery-Id: a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

Consumers should not rely on the `X-Brokkr-*` headers for broker-delivered events.

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
    "agent_event_id": "f0e1d2c3-...",
    "agent_id": "e5f6g7h8-...",
    "deployment_object_id": "a1b2c3d4-...",
    "stack_id": "d4c3b2a1-...",
    "event_type": "APPLY",
    "status": "SUCCESS",
    "message": "Applied successfully",
    "created_at": "2025-01-02T10:00:00Z"
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

### Encryption at Rest

Webhook URLs and authentication headers contain sensitive information and are encrypted before storage:

- **Algorithm**: AES-256-GCM (Authenticated Encryption with Associated Data)
- **Key management**: Encryption key configured via `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` environment variable
- **Key format**: 64 hex characters, encoding a 32-byte key
- **Fields encrypted**: `url_encrypted`, `auth_header_encrypted`
- **Response handling**: API responses show `has_url: true` and `has_auth_header: true/false` rather than the actual values

When updating a subscription, provide the URL or auth header to re-encrypt with the current key.

#### Unset Key

When `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` is unset or empty, the broker generates a **random key for that process only** and logs a warning. Any subscription created while the key is unset is encrypted with a key that is discarded when the process exits, so after the next restart:

- The subscription still lists normally, because the API only reports `has_url` and `has_auth_header`.
- Its URL and auth header can never be decrypted again, so it can never deliver.
- Broker-path deliveries for it go straight to `dead` with a decryption error in `last_error`; agent-path deliveries do the same.

To make this unrecoverable state impossible to enter silently, the broker **refuses to start** when the key is unset and at least one webhook subscription row already exists. The startup error names `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` and states the row count. A fresh install with no subscriptions starts normally with the warning above.

A key that is set but wrong does not block startup. That fault is confined to webhooks and surfaces at delivery time as dead deliveries and an incrementing `brokkr_webhook_decrypt_failures_total` counter.

#### Recovery

If the broker refuses to start, or existing subscriptions have become undeliverable:

1. Set `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` to the key the subscriptions were originally created with, and restart. Existing subscriptions resume delivering.
2. If that key is lost, the stored URLs and auth headers are unrecoverable. Delete the affected subscriptions, set a fresh key (`openssl rand -hex 32`), restart, and recreate the subscriptions.

Updating a subscription's `url` and `auth_header` re-encrypts them under the currently active key, which repairs an individual subscription without deleting it.

#### Chart Values

| Value | Purpose |
|-------|---------|
| `broker.webhookEncryptionKey` | Sets the key directly. Rendered into the broker ConfigMap in plaintext. |
| `broker.webhookEncryptionKeyExistingSecret` | Name of a pre-existing Kubernetes Secret holding the key. When set, the key is injected via `secretKeyRef` and overrides `broker.webhookEncryptionKey`, keeping it out of values files and git. |
| `broker.webhookEncryptionKeyExistingSecretKey` | Key within that Secret. Defaults to `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`. |

The key is read once at startup. Changing it requires a pod restart, and invalidates every subscription encrypted under the previous key.

### Access Control

- **Admin-only access**: All webhook endpoints require admin PAK authentication
- **Agent authentication**: Agents use their PAK to fetch and report deliveries
- **TLS recommended**: Use HTTPS endpoints in production
- **Secret rotation**: Rotate auth headers by updating the subscription

## Data Retention

The webhook system automatically cleans up old delivery records to prevent unbounded database growth:

- **Retention period**: 7 days
- **Cleanup frequency**: Every hour
- **Scope**: Deliveries in a terminal state (`success` or `dead`) older than the retention period are permanently deleted; `pending`, `acquired`, and `failed` deliveries are not removed by this task
- **Subscriptions**: Deleted when explicitly removed; delivery history is cleaned up by the retention policy

Deliveries in terminal states (`success`, `dead`) are retained for the full 7-day period to support troubleshooting and audit requirements. Adjust retention by modifying the cleanup background task configuration if needed.

## Performance Characteristics

### Broker Delivery

- Background task polls every 5 seconds, set by `broker.webhook_delivery_interval_seconds`
- Batch size: 50 deliveries per cycle, set by `broker.webhook_delivery_batch_size`
- Both values are read once when the delivery worker starts. Changing them requires a broker restart; they are not applied by a configuration reload.
- Deliveries within a batch are attempted sequentially, single-threaded per broker instance

### Agent Delivery

- Polling interval: 10 seconds, fixed
- Batch size: an agent claims at most 10 deliveries per poll, fixed and independent of the broker's batch size
- Deliveries are attempted sequentially, single-threaded per agent
- TTL: 60 seconds for acquired deliveries

### Scaling Considerations

- Multiple broker instances share the delivery workload
- Agent delivery scales with number of matching agents
- Delivery latency: typically < 15 seconds from event to delivery

## Monitoring

### `brokkr_webhook_decrypt_failures_total`

Counter. Increments whenever a stored webhook URL or auth header cannot be decrypted with the broker's current encryption key. Any nonzero value means the key does not match the one those subscriptions were created with, and the affected deliveries are being marked `dead`.

| Label | Values | Meaning |
|-------|--------|---------|
| `field` | `url`, `auth_header` | Which encrypted field failed to decrypt |
| `path` | `broker`, `agent` | Which delivery path hit the failure |

Exposed on the broker's `/metrics` endpoint alongside the metrics cataloged in [Monitoring and Observability](./monitoring.md).

## Related Documentation

- [How to Configure Webhooks](../how-to/webhooks.md) - Step-by-step setup guide
- [Monitoring and Observability](./monitoring.md) - Full Prometheus metrics catalog
- [Architecture](../explanation/architecture.md) - System architecture overview
- [Data Flows](../explanation/data-flows.md) - Event flow through the system
