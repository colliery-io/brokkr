# Configuring Webhooks

Brokkr's webhook system enables external systems to receive real-time notifications when events occur. This guide covers creating webhook subscriptions, configuring delivery options, and integrating with external services.

## Overview

Webhooks provide HTTP callbacks for events such as deployments applied or failed, work orders completed, agents registered, and stacks created or deleted. The full catalog is in the [Event Types reference](../reference/webhooks.md#event-types).

Brokkr supports two delivery modes:
- **Broker delivery** (default): The broker sends webhooks directly
- **Agent delivery**: An agent in the target cluster delivers webhooks, enabling access to in-cluster services

## Prerequisites

- Admin PAK for creating webhook subscriptions
- Target endpoint accessible from the broker or agent (depending on delivery mode)
- HTTPS recommended for production endpoints
- A webhook encryption key configured on the broker — see below

## Setting the Encryption Key

Do this **before** creating any subscription. Webhook URLs and auth headers are encrypted at rest, and if `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` is unset the broker generates a random key that it discards when the process exits. Subscriptions created under a random key can never deliver again after a restart, and there is no way to recover their stored URL and auth header.

Generate a key:

```bash
openssl rand -hex 32
```

For Helm installs, put it in a Kubernetes Secret and point the chart at it rather than writing it into your values file:

```bash
kubectl create secret generic brokkr-webhook-key \
  --from-literal=BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY="$(openssl rand -hex 32)"

helm upgrade --install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set broker.webhookEncryptionKeyExistingSecret=brokkr-webhook-key
```

To set the key inline instead — acceptable for development, but it renders into the broker ConfigMap in plaintext:

```bash
helm upgrade --install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set broker.webhookEncryptionKey=<64-hex-character-key>
```

Store the key somewhere you can retrieve it. If it is lost, the only repair is deleting and recreating every subscription.

Once at least one subscription exists, the broker refuses to start with the key unset, and logs an error naming `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`. If you hit that, see [Broker Refuses to Start](#broker-refuses-to-start).

## Creating a Webhook Subscription

### Basic Webhook (Broker Delivery)

Create a webhook subscription using the API:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Deployment Notifications",
    "url": "https://my-service.example.com/webhooks/brokkr",
    "event_types": ["deployment.applied", "deployment.failed"],
    "auth_header": "Bearer my-webhook-secret"
  }'
```

Response:
```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "Deployment Notifications",
  "has_url": true,
  "has_auth_header": true,
  "event_types": ["deployment.applied", "deployment.failed"],
  "enabled": true,
  "max_retries": 5,
  "timeout_seconds": 30,
  "created_at": "2025-01-02T10:00:00Z"
}
```

### Webhook with Agent Delivery

For in-cluster targets that the broker cannot reach, configure agent delivery using `target_labels`:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "In-Cluster Alerts",
    "url": "http://alertmanager.monitoring.svc.cluster.local:9093/api/v2/alerts",
    "event_types": ["deployment.failed", "workorder.failed"],
    "target_labels": ["env:production"]
  }'
```

When `target_labels` is set:
1. Deliveries are queued for agents matching ALL specified labels
2. The matching agent fetches pending deliveries during its polling loop
3. The agent delivers the webhook from inside the cluster
4. The agent reports success/failure back to the broker

### Wildcard Event Types

Subscribe to multiple events using wildcards:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "All Deployment Events",
    "url": "https://webhook.example.com/deployments",
    "event_types": ["deployment.*"]
  }'
```

See [Wildcard Patterns](../reference/webhooks.md#wildcard-patterns) for the supported patterns.

## Configuring Delivery Options

### Retry Settings

Configure retry behavior for failed deliveries:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Critical Alerts",
    "url": "https://pagerduty.example.com/webhook",
    "event_types": ["deployment.failed"],
    "max_retries": 10,
    "timeout_seconds": 60
  }'
```

Retry behavior:
- `max_retries` counts **total attempts**, not retries after the first one. The default of 5 means 5 attempts; the `10` above means 10 attempts. Accepted range is 0-10.
- Retryable failures use exponential backoff: 2, 4, 8, 16... seconds
- Retryable failures are HTTP 408, 425, 429, any 5xx, connection failures, and timeouts
- Every other 4xx — 400, 401, 403, 404, 422 and so on — is permanent. The delivery is marked `dead` on the **first** attempt with the status recorded in `last_error`, and no retries are made. If you expect retries against an endpoint that returns 404 while it warms up, that will not happen; fix the endpoint or the URL.
- `timeout_seconds` applies per request and is honored on every delivery path. Accepted range is 1-300 seconds, default 30. A value above 30 now genuinely waits that long.

Because the broker attempts a batch of deliveries sequentially, a long timeout on an unresponsive endpoint delays other webhooks. Prefer a timeout that reflects how fast the endpoint actually answers.

### Filters

Filter events down to a single agent or a single stack. Filters are evaluated before the delivery record is created, so excluded events produce no delivery at all:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production Stack Alerts",
    "url": "https://slack.example.com/webhook",
    "event_types": ["deployment.created", "deployment.deleted"],
    "filters": {
      "stack_id": "b7e4d1c0-1111-4222-8333-444455556666"
    }
  }'
```

Only `agent_id` and `stack_id` are recognized. Two rules decide what you actually receive:

- Both fields must match if both are set.
- **An event that does not carry the field you filtered on is never delivered.** A `stack_id` filter receives nothing from event types whose payload has no `stack_id` — `workorder.*` and the `agent.*` events, for instance. The whole `deployment.*` family carries `stack_id`, so filtering `deployment.*` by stack delivers the full lifecycle for that stack. The one gap is an event whose `stack_id` is `null` because the deployment object was already soft-deleted when the event was emitted; a JSON `null` counts as absent, not as a value.

Check the [filter fields by event type table](../reference/webhooks.md#filter-fields-by-event-type) before setting a filter, and confirm the response echoes the filter you intended.

A `labels` key inside `filters` is **rejected with 422** — it never filtered anything. For label-based routing use `target_labels`, described under [Webhook with Agent Delivery](#webhook-with-agent-delivery). If you are updating a script that sent `filters.labels`, drop the key; see [Removed Fields](../reference/webhooks.md#removed-fields) for the full migration.

Subscriptions already stored with a legacy `labels` key are unaffected and keep delivering. They only need attention if you send a `PUT` that echoes their filters back — strip `labels` from the object first, or the update is rejected.

To confirm the filter the broker will actually evaluate:

```bash
curl "http://broker:3000/api/v1/webhooks/{webhook_id}" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

The `filters` object in the response is exactly what is applied at emission time.

## Managing Webhooks

### List All Webhooks

```bash
curl "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Get Webhook Details

```bash
curl "http://broker:3000/api/v1/webhooks/{webhook_id}" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Update a Webhook

```bash
curl -X PUT "http://broker:3000/api/v1/webhooks/{webhook_id}" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": false
  }'
```

### Delete a Webhook

```bash
curl -X DELETE "http://broker:3000/api/v1/webhooks/{webhook_id}" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Test a Webhook

Send a test event to verify connectivity:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks/{webhook_id}/test" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

The test is sent immediately, using the subscription's own `timeout_seconds`, and creates no delivery record. Use this after creating a subscription instead of the removed `validate` field, which is now rejected with 422.

## Viewing Delivery Status

### List Deliveries for a Subscription

```bash
curl "http://broker:3000/api/v1/webhooks/{webhook_id}/deliveries" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

### Filter by Status

```bash
# Show only failed deliveries
curl "http://broker:3000/api/v1/webhooks/{webhook_id}/deliveries?status=failed" \
  -H "Authorization: Bearer $ADMIN_PAK"

# Show only dead (max retries exceeded)
curl "http://broker:3000/api/v1/webhooks/{webhook_id}/deliveries?status=dead" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

See [Delivery Status](../reference/webhooks.md#delivery-status) for what each status means and the state transitions between them.

## Webhook Payload Format

Deliveries are JSON POSTs with your configured auth header and a body containing `id`, `event_type`, `timestamp`, and event-specific `data`. Agent-delivered webhooks additionally carry `X-Brokkr-Event-Type` and `X-Brokkr-Delivery-Id` headers; broker-delivered webhooks do not, so do not route on them unless the subscription uses `target_labels`. See the [Webhook Payload Format reference](../reference/webhooks.md#webhook-payload-format) for headers, body structure, and example payloads.

## Common Patterns

### Slack Integration

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Slack Deployment Alerts",
    "url": "https://hooks.slack.com/services/T00/B00/XXX",
    "event_types": ["deployment.applied", "deployment.failed"]
  }'
```

### PagerDuty Integration

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "PagerDuty Critical Alerts",
    "url": "https://events.pagerduty.com/v2/enqueue",
    "event_types": ["deployment.failed", "workorder.failed"],
    "auth_header": "Token token=your-pagerduty-token",
    "max_retries": 10
  }'
```

### In-Cluster Alertmanager

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alertmanager Notifications",
    "url": "http://alertmanager.monitoring.svc.cluster.local:9093/api/v2/alerts",
    "event_types": ["deployment.failed"],
    "target_labels": ["role:monitoring"]
  }'
```

## Troubleshooting

### Webhooks Not Being Delivered

1. Check if the subscription is enabled:
   ```bash
   curl "http://broker:3000/api/v1/webhooks/{id}" \
     -H "Authorization: Bearer $ADMIN_PAK"
   ```

2. Check delivery status for failures:
   ```bash
   curl "http://broker:3000/api/v1/webhooks/{id}/deliveries?status=failed" \
     -H "Authorization: Bearer $ADMIN_PAK"
   ```

3. Verify endpoint is reachable from broker/agent

4. If the delivery list is **empty** rather than failing, no deliveries were ever created. Either no matching event occurred, or a filter excluded them. Confirm the subscription's `filters` against the [filter fields by event type table](../reference/webhooks.md#filter-fields-by-event-type) — a filter on a field the event type does not carry excludes that event type entirely. Clear the filter to confirm:
   ```bash
   curl -X PUT "http://broker:3000/api/v1/webhooks/{id}" \
     -H "Authorization: Bearer $ADMIN_PAK" \
     -H "Content-Type: application/json" \
     -d '{"filters": null}'
   ```
   Broker logs also record an error for a subscription whose stored filter cannot be parsed; such a subscription delivers nothing until the filter is rewritten.

### Deliveries Dead on the First Attempt

A delivery in `dead` with `attempts: 1` means the failure was classified as permanent. Read `last_error`:

```bash
curl "http://broker:3000/api/v1/webhooks/{id}/deliveries?status=dead" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

- `HTTP 404` / `HTTP 401` / `HTTP 403` / `HTTP 422` and other 4xx statuses are not retried. Fix the URL, the auth header, or the receiving handler, then recreate the events or wait for new ones — dead deliveries are not replayed.
- `Failed to decrypt URL` or `Failed to decrypt auth header` means the broker's encryption key does not match the one that created the subscription. See below.

### Deliveries Failing to Decrypt

Check the broker's metrics endpoint:

```bash
curl -s "http://broker:3000/metrics" | grep brokkr_webhook_decrypt_failures_total
```

Any nonzero value means `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` differs from the key those subscriptions were created with — most often because they were created while the key was unset and the broker has since restarted. The `field` label shows whether the URL or the auth header failed; `path` shows whether the broker or an agent hit it.

To fix, either restore the original key and restart the broker, or delete and recreate the affected subscriptions under the current key. Updating a subscription's `url` and `auth_header` re-encrypts them under the current key and repairs it in place:

```bash
curl -X PUT "http://broker:3000/api/v1/webhooks/{id}" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://my-service.example.com/webhooks/brokkr",
    "auth_header": "Bearer my-webhook-secret"
  }'
```

### Broker Refuses to Start

If the broker exits at startup with an error naming `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`, the key is unset while webhook subscriptions already exist. The broker stops rather than come up with a random key that would make those subscriptions permanently undeliverable.

1. Set the key the subscriptions were created with and restart. Delivery resumes.
2. If that key is lost, set a fresh one (`openssl rand -hex 32`), restart, then delete and recreate the subscriptions — their stored URLs and auth headers cannot be recovered.

### Changing Delivery Batch Size or Interval Has No Effect

`broker.webhookDeliveryIntervalSeconds` and `broker.webhookDeliveryBatchSize` are read once when the broker starts. A configuration reload does not apply them; restart the broker pod after changing either.

### Agent-Delivered Webhooks Failing

1. Verify agent has matching labels (labels are a subresource, not part of the agent object):
   ```bash
   curl "http://broker:3000/api/v1/agents/{agent_id}/labels" \
     -H "Authorization: Bearer $ADMIN_PAK"
   ```

2. Check agent logs for delivery errors:
   ```bash
   kubectl logs -l app.kubernetes.io/name=brokkr-agent -c agent
   ```

3. Ensure the agent is ACTIVE and polling

### Deliveries Stuck in "Acquired" State

Deliveries have a 60-second TTL. If they remain acquired longer, they'll be released back to pending. This can happen if:
- The delivering agent/broker crashed mid-delivery
- Network issues prevented result reporting

The system automatically recovers these deliveries.

## Related Documentation

- [Webhooks Reference](../reference/webhooks.md) - Complete API reference
- [Event Types](../reference/webhooks.md#event-types) - List of all event types
- [Architecture](../explanation/architecture.md) - How webhooks fit into Brokkr
