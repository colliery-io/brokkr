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
- Failed deliveries use exponential backoff: 2, 4, 8, 16... seconds
- After `max_retries` failures, deliveries are marked as "dead"
- Delivery timeouts count as failures

### Filters

Filter events by specific agents or stacks:

```bash
curl -X POST "http://broker:3000/api/v1/webhooks" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production Stack Alerts",
    "url": "https://slack.example.com/webhook",
    "event_types": ["deployment.*"],
    "filters": {
      "labels": {"env": "production"}
    }
  }'
```

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

Deliveries are JSON POSTs carrying `X-Brokkr-Event-Type` and `X-Brokkr-Delivery-Id` headers (plus your configured auth header) and a body with `id`, `event_type`, `timestamp`, and event-specific `data`. See the [Webhook Payload Format reference](../reference/webhooks.md#webhook-payload-format) for headers, body structure, and example payloads.

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
