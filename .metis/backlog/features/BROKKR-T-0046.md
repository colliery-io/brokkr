---
id: event-webhook-system
level: task
title: "Event Webhook System"
short_code: "BROKKR-T-0046"
created_at: 2025-12-20T02:19:55.756396+00:00
updated_at: 2025-12-20T02:19:55.756396+00:00
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

# Event Webhook System

A generic event notification system that delivers webhooks to external systems when events occur in Brokkr.

## Objective

Implement a flexible webhook delivery system that allows operators to subscribe to Brokkr events and receive HTTP callbacks when those events occur. This enables integration with alerting systems (PagerDuty, OpsGenie), chat platforms (Slack, Teams), CI/CD pipelines, and custom automation.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Operators can integrate Brokkr with their existing alerting and automation tooling without polling
- **Business Value**: Enables enterprise adoption by supporting standard webhook integrations; reduces operational burden
- **Effort Estimate**: M (Medium) - straightforward patterns, but needs reliable delivery

## Acceptance Criteria

### Webhook Registration
- [ ] API to register webhook subscriptions (URL, secret, event types)
- [ ] Support filtering by event type, agent, stack, or label selectors
- [ ] Webhook secrets stored securely (encrypted at rest)
- [ ] CRUD operations for webhook subscriptions

### Event Types
- [ ] `health.degraded` - Deployment health changed to degraded
- [ ] `health.failing` - Deployment health changed to failing
- [ ] `health.recovered` - Deployment health returned to healthy
- [ ] `deployment.applied` - Stack deployed to agent
- [ ] `deployment.failed` - Stack deployment failed
- [ ] `agent.offline` - Agent missed heartbeats
- [ ] `agent.online` - Agent came back online
- [ ] `workorder.completed` - Work order finished (success or failure)

### Delivery & Reliability
- [ ] Reliable delivery with exponential backoff retries
- [ ] Configurable retry policy (max attempts, backoff)
- [ ] Dead letter queue for failed deliveries
- [ ] Delivery status tracking per webhook

### Security
- [ ] HMAC signature on payloads (X-Brokkr-Signature header)
- [ ] Timestamp in payload to prevent replay attacks
- [ ] TLS required for webhook URLs (configurable for dev)

### Observability
- [ ] Delivery success/failure metrics
- [ ] Recent delivery history per subscription
- [ ] API to view delivery status and retry failed deliveries

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BROKER EVENT SYSTEM                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Event Sources                    Event Bus                              │
│  ┌──────────────────┐            ┌──────────────────┐                   │
│  │ Health Monitor   │───────────>│                  │                   │
│  │ (health.*)       │            │                  │                   │
│  ├──────────────────┤            │   In-Process     │                   │
│  │ Deployment Svc   │───────────>│   Event Channel  │                   │
│  │ (deployment.*)   │            │                  │                   │
│  ├──────────────────┤            │   (tokio mpsc)   │                   │
│  │ Agent Monitor    │───────────>│                  │                   │
│  │ (agent.*)        │            │                  │                   │
│  ├──────────────────┤            │                  │                   │
│  │ Work Order Svc   │───────────>│                  │                   │
│  │ (workorder.*)    │            └────────┬─────────┘                   │
│  └──────────────────┘                     │                              │
│                                           ▼                              │
│                              ┌──────────────────────┐                   │
│                              │  Webhook Dispatcher  │                   │
│                              │  - Match subscribers │                   │
│                              │  - Queue deliveries  │                   │
│                              └──────────┬───────────┘                   │
│                                         │                                │
│                                         ▼                                │
│                              ┌──────────────────────┐                   │
│                              │  Delivery Worker     │                   │
│                              │  - HTTP POST         │                   │
│                              │  - HMAC signing      │                   │
│                              │  - Retry on failure  │                   │
│                              └──────────┬───────────┘                   │
│                                         │                                │
└─────────────────────────────────────────┼────────────────────────────────┘
                                          │
                                          ▼
                              ┌──────────────────────┐
                              │  External Systems    │
                              │  - PagerDuty         │
                              │  - Slack             │
                              │  - Custom endpoints  │
                              └──────────────────────┘
```

---

## Database Schema

```sql
CREATE TABLE webhook_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Subscription details
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    secret_encrypted BYTEA NOT NULL,        -- Encrypted webhook secret for HMAC
    
    -- Event filtering
    event_types TEXT[] NOT NULL,            -- Array of event type patterns (e.g., 'health.*')
    filters JSONB,                          -- Optional filters: agent_id, stack_id, labels
    
    -- Delivery settings
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INT NOT NULL DEFAULT 5,
    timeout_seconds INT NOT NULL DEFAULT 30,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255)
);

CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE,
    
    -- Event info
    event_type VARCHAR(100) NOT NULL,
    event_id UUID NOT NULL,                 -- Idempotency key
    payload JSONB NOT NULL,
    
    -- Delivery status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, success, failed, dead
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ,
    last_error TEXT,
    
    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_webhook_deliveries_pending 
    ON webhook_deliveries(next_attempt_at) 
    WHERE status = 'pending';
CREATE INDEX idx_webhook_deliveries_subscription 
    ON webhook_deliveries(subscription_id, created_at DESC);
```

---

## API Endpoints

### POST `/api/v1/webhooks`

Create a webhook subscription.

```json
{
  "name": "PagerDuty Alerts",
  "url": "https://events.pagerduty.com/v2/enqueue",
  "secret": "whsec_...",
  "event_types": ["health.degraded", "health.failing", "agent.offline"],
  "filters": {
    "labels": { "env": "production" }
  }
}
```

### GET `/api/v1/webhooks`

List all webhook subscriptions.

### GET `/api/v1/webhooks/{id}`

Get subscription details including recent delivery stats.

### PATCH `/api/v1/webhooks/{id}`

Update subscription (URL, events, enabled status).

### DELETE `/api/v1/webhooks/{id}`

Delete subscription.

### GET `/api/v1/webhooks/{id}/deliveries`

List recent deliveries for a subscription.

### POST `/api/v1/webhooks/{id}/deliveries/{delivery_id}/retry`

Manually retry a failed delivery.

---

## Webhook Payload Format

```json
{
  "id": "evt_abc123",
  "type": "health.degraded",
  "timestamp": "2025-01-15T10:30:00Z",
  "data": {
    "deployment_object_id": "uuid",
    "stack_id": "uuid",
    "stack_name": "my-app",
    "agent_id": "uuid",
    "agent_name": "prod-cluster-1",
    "previous_status": "healthy",
    "current_status": "degraded",
    "conditions": ["ImagePullBackOff"],
    "summary": {
      "pods_ready": 2,
      "pods_total": 3
    }
  }
}
```

**Headers**:
```
Content-Type: application/json
X-Brokkr-Signature: sha256=abc123...
X-Brokkr-Event: health.degraded
X-Brokkr-Delivery-Id: dlv_xyz789
X-Brokkr-Timestamp: 1705315800
```

---

## Implementation Notes

### Technical Approach

1. **In-process event bus**: Use tokio mpsc channel for event distribution within broker
2. **Async delivery workers**: Background task pool processes delivery queue
3. **Exponential backoff**: 1s, 2s, 4s, 8s, 16s... up to max retries
4. **Idempotency**: Event ID in payload allows receivers to dedupe

### Dependencies

- **BROKKR-T-0045** (Deployment Health Monitoring): Provides health.* events
- Existing agent heartbeat system: Provides agent.* events
- Existing deployment system: Provides deployment.* events

### Risk Considerations

- **Slow webhooks blocking**: Use connection timeouts and async workers
- **Webhook endpoint down**: Retry with backoff, eventually dead-letter
- **Secret management**: Encrypt secrets at rest, never log

---

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Create database migrations for webhook tables
- [ ] Implement in-process event bus (tokio mpsc)
- [ ] Define event type enum and payload structures
- [ ] Implement webhook subscription DAL

### Phase 2: Subscription API
- [ ] Implement CRUD endpoints for webhook subscriptions
- [ ] Secret encryption/decryption utilities
- [ ] Event type pattern matching (wildcards)
- [ ] Filter matching logic (agent, stack, labels)

### Phase 3: Delivery System
- [ ] Implement delivery worker background task
- [ ] HMAC signature generation
- [ ] HTTP client with timeout and retries
- [ ] Exponential backoff logic
- [ ] Dead letter handling

### Phase 4: Event Integration
- [ ] Emit health.* events from health monitoring system
- [ ] Emit agent.* events from heartbeat system
- [ ] Emit deployment.* events from deployment system
- [ ] Emit workorder.* events from work order system

### Phase 5: Observability & Polish
- [ ] Delivery status API endpoints
- [ ] Manual retry endpoint
- [ ] Prometheus metrics (deliveries, latency, failures)
- [ ] Integration tests
- [ ] Documentation

---

## Open Questions

1. **Event persistence**: Should events be persisted independently of deliveries for replay/audit?
2. **Rate limiting**: Should we rate-limit deliveries per subscription to protect endpoints?
3. **Batching**: Should we support batching multiple events into single webhook call?
4. **Subscription validation**: Should we verify webhook URL is reachable on create?

---

## Related Documents

- **BROKKR-T-0045**: Deployment Health Monitoring - primary consumer of health.* events

---

## Status Updates

*To be added during implementation*