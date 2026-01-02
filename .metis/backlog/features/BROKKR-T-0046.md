---
id: event-webhook-system
level: task
title: "Event Webhook System"
short_code: "BROKKR-T-0046"
created_at: 2025-12-20T02:19:55.756396+00:00
updated_at: 2025-12-29T14:12:51.967030+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Webhook Registration
- [ ] API to register webhook subscriptions (URL, auth_header, event types)
- [ ] Support filtering by event type, agent, stack, or label selectors
- [ ] URL and auth_header encrypted at rest
- [ ] Optional `validate` flag to test endpoint on create
- [ ] CRUD operations for webhook subscriptions

### Event Types

Events are focused on actions and state changes within the Brokkr platform itself, not monitoring of deployed resources. Agent liveness is handled via TTL on heartbeat, not discrete events.

#### Agent Events

| Event              | Description                                |
|--------------------|--------------------------------------------|
| agent.registered   | A new agent has registered with the broker |
| agent.deregistered | An agent has been removed from the system  |

#### Stack Events

| Event         | Description                  |
|---------------|------------------------------|
| stack.created | A new stack has been created |
| stack.deleted | A stack has been deleted     |

#### Deployment Object Events

| Event              | Description                                                      |
|--------------------|------------------------------------------------------------------|
| deployment.created | A new deployment object has been created and queued              |
| deployment.applied | An agent successfully applied a deployment object to its cluster |
| deployment.failed  | An agent failed to apply a deployment object                     |
| deployment.deleted | A deployment object deletion marker has been presented           |

#### Work Order Events

| Event               | Description                                      |
|---------------------|--------------------------------------------------|
| workorder.created   | A new work order has been created                |
| workorder.claimed   | An agent has claimed a work order                |
| workorder.completed | An agent has successfully completed a work order |
| workorder.failed    | A work order execution failed                    |

*Note: Build events (Shipwright) are covered by work order events since builds are executed as work orders.*

### Delivery & Reliability
- [ ] Reliable delivery with exponential backoff retries
- [ ] Configurable retry policy (max attempts, backoff)
- [ ] Dead letter queue for failed deliveries
- [ ] Delivery status tracking per webhook

### Security
- [ ] Optional auth header per subscription (for endpoints that require it)
- [ ] URLs may contain embedded tokens (e.g., Slack incoming webhooks)
- [ ] TLS recommended for webhook URLs (warning on HTTP, configurable)

### Observability
- [ ] Delivery success/failure metrics
- [ ] Recent delivery history per subscription
- [ ] API to view delivery status and retry failed deliveries

---

## Architecture

Database-centric event delivery with distributed agents. No in-memory event bus - the database is the event store and delivery queue.

### Delivery Targeting

Webhook subscriptions specify who delivers via labels/annotations (same model as deployments and work orders):

| Target Labels | Who Delivers |
|---------------|--------------|
| NULL / empty  | Broker delivers directly |
| `["env:prod", "region:us-east"]` | Any agent matching those labels can claim and deliver |

Single delivery model - first agent to claim a delivery owns it. No duplicate sends.

### Event Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              BROKER                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Broker-Side Events (DAL inserts directly)                              │
│  ┌──────────────────┐                                                   │
│  │ Agent DAL        │──┐  agent.registered, agent.deregistered          │
│  ├──────────────────┤  │                                                │
│  │ Stack DAL        │──┼─► INSERT webhook_deliveries                    │
│  ├──────────────────┤  │   (target_labels from subscription)            │
│  │ Deployment DAL   │──┘   deployment.created, deployment.deleted       │
│  └──────────────────┘                                                   │
│                                                                          │
│  Agent-Side Events (reported via API)                                   │
│  ┌──────────────────┐                                                   │
│  │ Agent reports    │───► INSERT webhook_deliveries                     │
│  │ event outcomes   │     deployment.applied, deployment.failed         │
│  └──────────────────┘     workorder.claimed, .completed, .failed        │
│                                                                          │
│  Broker Delivery Worker (for target_labels = NULL)                      │
│  ┌──────────────────┐                                                   │
│  │ Poll pending     │───► Deliver via HTTP POST                         │
│  │ broker-targeted  │                                                   │
│  └──────────────────┘                                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                    AGENT (per infrastructure partition)                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Agent polls broker for pending activities (single transaction):        │
│  - Deployments matching agent's labels                                  │
│  - Work orders matching agent's labels                                  │
│  - Webhook deliveries matching agent's labels (claim on fetch)          │
│                                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                          │
│  │ Reconciler       │     │ Delivery Worker  │                          │
│  │ - Apply manifests│     │ - HTTP POST to   │                          │
│  │ - Report events  │     │   partition-local│                          │
│  └──────────────────┘     │   endpoints      │                          │
│                           └──────────────────┘                          │
│  ┌──────────────────┐                                                   │
│  │ Work Order Exec  │     Agent can reach endpoints inside              │
│  │ - Builds, etc    │     firewalled infrastructure that                │
│  │ - Report events  │     broker cannot reach.                          │
│  └──────────────────┘                                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Database as event store** - No in-memory bus. Events persisted as webhook_deliveries immediately. Crash-safe and horizontally scalable.

2. **Label-based targeting** - Consistent with deployments and work orders. Subscription specifies target_labels; matching agents can claim.

3. **Single delivery model** - First agent to claim wins. No duplicate deliveries.

4. **Pull-based consistency** - Agents poll for all pending work in one transaction.

---

## Database Schema

```sql
CREATE TABLE webhook_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Subscription details
    name VARCHAR(255) NOT NULL,
    url_encrypted BYTEA NOT NULL,           -- Encrypted; may contain embedded tokens (e.g., Slack URLs)
    auth_header_encrypted BYTEA,            -- Encrypted; optional: "Bearer xyz" or "Token abc"

    -- Event filtering
    event_types TEXT[] NOT NULL,            -- Array of event type patterns (e.g., 'deployment.*')
    filters JSONB,                          -- Optional filters: agent_id, stack_id, labels

    -- Delivery targeting
    target_labels TEXT[],                   -- NULL = broker delivers; labels = matching agent delivers

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

    -- Targeting (copied from subscription at creation time)
    target_labels TEXT[],                   -- NULL = broker; labels = agent

    -- Delivery status
    -- pending:  waiting to be claimed
    -- acquired: claimed, being processed (has TTL via acquired_until)
    -- success:  delivered successfully
    -- failed:   attempt failed, will retry (goes back to pending)
    -- dead:     max retries exceeded
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Claim tracking
    acquired_by UUID,                       -- Agent ID or NULL for broker
    acquired_until TIMESTAMPTZ,             -- TTL - if exceeded, release back to pending

    -- Retry tracking
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,              -- When to retry after failure
    last_error TEXT,

    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_webhook_deliveries_pending
    ON webhook_deliveries(created_at)
    WHERE status = 'pending';

CREATE INDEX idx_webhook_deliveries_acquired_expired
    ON webhook_deliveries(acquired_until)
    WHERE status = 'acquired';

CREATE INDEX idx_webhook_deliveries_retry
    ON webhook_deliveries(next_retry_at)
    WHERE status = 'failed';

CREATE INDEX idx_webhook_deliveries_subscription
    ON webhook_deliveries(subscription_id, created_at DESC);
```

### Delivery State Machine

```
                    ┌──────────────────────────────────────┐
                    │                                      │
                    ▼                                      │
┌─────────┐    ┌──────────┐    ┌─────────┐           ┌────────┐
│ pending │───►│ acquired │───►│ success │           │  dead  │
└─────────┘    └──────────┘    └─────────┘           └────────┘
     ▲              │                                     ▲
     │              │ (TTL expired                        │
     │              │  or failure)                        │ (max retries)
     │              ▼                                     │
     │         ┌──────────┐                               │
     └─────────│  failed  │───────────────────────────────┘
               └──────────┘
                    │
                    │ (next_retry_at reached)
                    ▼
               back to pending
```

---

## API Endpoints

### POST `/api/v1/webhooks`

Create a webhook subscription.

```json
{
  "name": "PagerDuty Alerts",
  "url": "https://events.pagerduty.com/v2/enqueue",
  "auth_header": "Bearer pd-routing-key-here",
  "event_types": ["deployment.failed", "workorder.failed", "stack.deleted"],
  "filters": {
    "labels": { "env": "production" }
  },
  "validate": true
}
```

For Slack (token embedded in URL, no auth header needed):
```json
{
  "name": "Slack Alerts",
  "url": "https://hooks.slack.com/services/T00/B00/xxxxx",
  "event_types": ["deployment.*", "workorder.*"]
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
  "type": "deployment.applied",
  "timestamp": "2025-01-15T10:30:00Z",
  "data": {
    "deployment_object_id": "uuid",
    "stack_id": "uuid",
    "stack_name": "my-app",
    "agent_id": "uuid",
    "agent_name": "prod-cluster-1"
  }
}
```

**Headers**:
```
Content-Type: application/json
Authorization: <auth_header if configured>
X-Brokkr-Event: deployment.applied
X-Brokkr-Delivery-Id: dlv_xyz789
```

---

## Implementation Notes

### Technical Approach

1. **Database as event store**: DAL methods insert directly into webhook_deliveries. No in-memory bus.
2. **Claim with TTL**: Agents/broker claim deliveries with `acquired_until` timestamp. Expired claims released.
3. **Exponential backoff**: 1s, 2s, 4s, 8s, 16s... up to max retries via `next_retry_at`
4. **Idempotency**: Event ID in payload allows receivers to dedupe

### Dependencies

- Agent DAL: Emits agent.registered, agent.deregistered
- Stack DAL: Emits stack.created, stack.deleted
- Deployment DAL: Emits deployment.created, deployment.deleted
- Agent API: Receives deployment.applied, deployment.failed, workorder.* events from agents

### Risk Considerations

- **Slow webhooks blocking**: Use connection timeouts and async workers
- **Webhook endpoint down**: Retry with backoff, eventually dead-letter
- **Secret management**: Encrypt URL and auth_header at rest, never log decrypted values

---

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Update database schema for webhook tables (add target_labels, remove target_agent_id)
- [ ] Define event type constants and payload structures
- [ ] Implement webhook subscription DAL
- [ ] Implement webhook delivery DAL with label-based claiming

### Phase 2: Subscription API
- [ ] Implement CRUD endpoints for webhook subscriptions
- [ ] URL/auth_header encryption/decryption utilities
- [ ] Event type pattern matching (wildcards)
- [ ] Target label configuration

### Phase 3: Broker-Side Events
- [ ] Agent DAL: emit agent.registered, agent.deregistered
- [ ] Stack DAL: emit stack.created, stack.deleted
- [ ] Deployment DAL: emit deployment.created, deployment.deleted
- [ ] Match subscriptions and insert webhook_deliveries

### Phase 4: Agent-Side Events
- [ ] Agent API endpoint to receive event reports from agents
- [ ] Agent reports: deployment.applied, deployment.failed
- [ ] Agent reports: workorder.claimed, workorder.completed, workorder.failed
- [ ] Match subscriptions and insert webhook_deliveries

### Phase 5: Delivery Workers
- [ ] Broker delivery worker (poll for target_labels = NULL)
- [ ] Agent delivery worker (poll for deliveries matching agent's labels)
- [ ] HTTP client with timeout and retries
- [ ] Exponential backoff logic
- [ ] Dead letter handling

### Phase 6: Observability & Polish
- [ ] Delivery status API endpoints
- [ ] Manual retry endpoint
- [ ] Delivery cleanup background task (configurable retention)
- [ ] Prometheus metrics (deliveries, latency, failures)
- [ ] Integration tests
- [ ] Documentation

---

## Design Decisions

1. **Encryption key management**: Use existing broker secret if available, otherwise config file + env var override (standard Brokkr pattern).

2. **Event persistence**: No separate event storage. Events only exist as deliveries. Rationale: Can't guarantee downstream idempotence, so replay capability would be misleading.

3. **Delivery cleanup**: Single configurable retention period via config + env var (e.g., `webhook.delivery_retention_days`, default 7 days).

4. **Subscription validation**: Optional. Support `validate: true` flag on create that sends a test event and requires successful delivery.

---

## Related Documents

- **BROKKR-T-0091**: Webhook delivery should occur from agent (data plane) not broker (control plane) - superseded by this architecture

---

## Testing Scenarios

### Unit Tests

#### Event Type Pattern Matching
- [ ] Exact match: `deployment.applied` matches `deployment.applied`
- [ ] Exact mismatch: `deployment.applied` does not match `deployment.failed`
- [ ] Wildcard suffix: `deployment.*` matches `deployment.applied`, `deployment.failed`
- [ ] Wildcard all: `*` matches any event type
- [ ] Partial wildcard rejected: `deploy*.applied` is invalid pattern

#### Label Matching
- [ ] NULL target_labels matches broker (NULL acquired_by)
- [ ] Empty array target_labels matches broker
- [ ] Single label match: agent with `env:prod` matches `["env:prod"]`
- [ ] Multi-label match: agent with `env:prod,region:us` matches `["env:prod", "region:us"]`
- [ ] Partial label mismatch: agent with `env:prod` does NOT match `["env:prod", "region:us"]`
- [ ] Agent without matching labels cannot claim delivery

#### State Machine Transitions
- [ ] pending → acquired: valid claim
- [ ] acquired → success: delivery succeeded
- [ ] acquired → failed: delivery failed, attempts incremented
- [ ] failed → pending: next_retry_at reached
- [ ] failed → dead: max_retries exceeded
- [ ] acquired → pending: acquired_until TTL expired
- [ ] Invalid transitions rejected (e.g., pending → success, dead → pending)

#### TTL Expiration
- [ ] Delivery with expired acquired_until is released to pending
- [ ] Delivery with valid acquired_until stays acquired
- [ ] Released delivery can be claimed by different agent

#### Exponential Backoff
- [ ] Attempt 1 failure: next_retry_at = now + 1s
- [ ] Attempt 2 failure: next_retry_at = now + 2s
- [ ] Attempt 3 failure: next_retry_at = now + 4s
- [ ] Attempt N failure: next_retry_at = now + 2^(N-1) seconds (capped at max)

#### Payload Serialization
- [ ] BrokkrEvent serializes to JSON correctly
- [ ] Event ID is UUID
- [ ] Timestamp is ISO8601
- [ ] Data field preserves nested structures

### Integration Tests

#### Subscription Management
- [ ] Create subscription with valid data succeeds
- [ ] Create subscription encrypts URL and auth_header
- [ ] Create subscription with empty event_types fails
- [ ] Create subscription with empty name fails
- [ ] Get subscription returns decrypted URL (for display)
- [ ] Update subscription modifies fields correctly
- [ ] Delete subscription cascades to deliveries
- [ ] List subscriptions returns all active subscriptions
- [ ] Disabled subscription does not receive new deliveries

#### Broker-Side Event Emission
- [ ] Agent registration creates delivery for matching subscriptions
- [ ] Agent deregistration creates delivery for matching subscriptions
- [ ] Stack creation creates delivery for matching subscriptions
- [ ] Stack deletion creates delivery for matching subscriptions
- [ ] Deployment object creation creates delivery for matching subscriptions
- [ ] Deployment deletion marker creates delivery for matching subscriptions
- [ ] No delivery created when no subscriptions match event type
- [ ] Delivery inherits target_labels from subscription

#### Agent-Side Event Reporting
- [ ] Agent reports deployment.applied → delivery created
- [ ] Agent reports deployment.failed → delivery created
- [ ] Agent reports workorder.claimed → delivery created
- [ ] Agent reports workorder.completed → delivery created
- [ ] Agent reports workorder.failed → delivery created
- [ ] Event report includes agent_id in payload

#### Broker Delivery Worker
- [ ] Worker polls for pending deliveries with target_labels = NULL
- [ ] Worker claims delivery (sets acquired_by, acquired_until)
- [ ] Worker delivers via HTTP POST to decrypted URL
- [ ] Worker includes Authorization header if auth_header set
- [ ] Worker includes X-Brokkr-Event header
- [ ] Worker includes X-Brokkr-Delivery-Id header
- [ ] Successful delivery → status = success, completed_at set
- [ ] Failed delivery (HTTP error) → status = failed, attempts incremented
- [ ] Failed delivery schedules next_retry_at with backoff
- [ ] Max retries exceeded → status = dead

#### Agent Delivery Worker
- [ ] Agent polls for pending deliveries matching its labels
- [ ] Agent claims delivery atomically (no duplicate claims)
- [ ] Agent delivers to partition-local endpoint
- [ ] Agent reports delivery result back to broker
- [ ] Agent handles endpoint timeout gracefully

#### TTL Recovery
- [ ] Expired acquired deliveries are released by background job
- [ ] Released delivery goes back to pending status
- [ ] Released delivery clears acquired_by and acquired_until
- [ ] Released delivery can be claimed by another worker

#### Retry Behavior
- [ ] Failed delivery with next_retry_at in future stays failed
- [ ] Background job moves failed → pending when next_retry_at reached
- [ ] Retry uses same subscription URL (re-decrypted)
- [ ] Retry increments attempts counter

#### End-to-End Scenarios
- [ ] E2E: Create subscription → trigger event → broker delivers → webhook received
- [ ] E2E: Create subscription with target_labels → trigger event → agent delivers → webhook received
- [ ] E2E: Delivery fails → retries with backoff → eventually succeeds
- [ ] E2E: Delivery fails max times → marked dead → no more retries
- [ ] E2E: Agent crashes mid-delivery → TTL expires → another agent delivers

---

## Status Updates

*To be added during implementation*