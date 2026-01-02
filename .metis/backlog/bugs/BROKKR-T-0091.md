---
id: webhook-delivery-should-occur-from
level: task
title: "Webhook delivery should occur from agent (data plane) not broker (control plane)"
short_code: "BROKKR-T-0091"
created_at: 2025-12-31T16:29:40.111362+00:00
updated_at: 2026-01-02T12:47:19.450382+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Webhook delivery should occur from agent (data plane) not broker (control plane)

## Objective

Refactor webhook delivery so that HTTP calls are made from the agent (data plane) rather than the broker (control plane). This enables webhooks to reach targets within cluster networks that the broker cannot access.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment

- **Affected Users**: All users attempting to use webhooks with in-cluster targets
- **Reproduction Steps**: 
  1. Deploy Brokkr with broker running outside the target cluster (standard CP/DP separation)
  2. Create a webhook subscription pointing to an in-cluster service (e.g., `http://my-service.my-namespace.svc.cluster.local:8080/webhook`)
  3. Trigger an event that should fire the webhook
- **Expected vs Actual**: 
  - **Expected**: Webhook is delivered to the in-cluster service
  - **Actual**: Webhook delivery fails because broker cannot reach cluster-internal DNS/network

## The Problem

Brokkr follows a control plane / data plane separation architecture:
- **Broker (Control Plane)**: Manages configuration, state, and orchestration
- **Agent (Data Plane)**: Runs inside target clusters, applies deployments, has cluster network access

Currently, webhook HTTP calls are made directly from the broker. This breaks when:
1. Webhook target is a service inside a Kubernetes cluster (cluster DNS not resolvable from broker)
2. Webhook target is in a private network only accessible from the data plane
3. Network policies restrict ingress to cluster-internal sources only

## Proposed Solution

### Architecture Change

**Current Flow:**
```
Event → Broker determines webhook → Broker calls HTTP endpoint → (fails if unreachable)
```

**New Flow:**
```
Event → Broker determines delivery mode → Route to broker OR agent
     → If agent: Agent polls for pending deliveries → Agent calls HTTP endpoint → Reports result
     → If broker: Current behavior (direct HTTP call)
```

### Delivery Mode Model

Webhook subscriptions gain a `delivery_mode` field with three options:

| Mode | Behavior |
|------|----------|
| `broker` | Broker delivers directly (current behavior, **default**) |
| `agent` | Agent delivers, routed by labels or event context |
| `auto` | Smart routing based on event type |

Plus an optional `delivery_target_labels` array for label-based agent targeting.

**Routing Logic:**

| Mode | Labels Set | Behavior |
|------|------------|----------|
| `broker` | ignored | Broker delivers (backwards compatible) |
| `agent` | yes | Route to agent(s) matching labels |
| `agent` | no | Route to agent from event context (e.g., agent that reported health change) |
| `auto` | optional | Smart routing based on event type (see below) |

**Auto Mode Routing Rules:**

| Event Type | Delivery Source | Reason |
|------------|-----------------|--------|
| `agent.degraded`, `agent.offline` | Broker | Can't rely on degraded/offline agent |
| `health.degraded`, `health.recovered` | Contextual agent | Agent is in-cluster, can reach internal targets |
| `workorder.completed` | Contextual agent | Agent just finished it, has network access |
| `stack.created`, `deployment_object.created` | Broker | Broker-side event, no agent context yet |

**Example Scenarios:**

1. "Alert my in-cluster monitoring when deployments fail"
   ```json
   { "delivery_mode": "agent", "delivery_target_labels": ["role:monitoring"] }
   ```

2. "Send to PagerDuty (external URL)"
   ```json
   { "delivery_mode": "broker" }
   ```

3. "Smart routing - let the system decide"
   ```json
   { "delivery_mode": "auto" }
   ```

### Implementation Approach

1. **New Agent Endpoint**: `GET /api/v1/agents/:id/webhooks/pending`
   - Returns queued webhook deliveries for this agent
   - Similar pattern to work orders and diagnostics

2. **New Agent Endpoint**: `POST /api/v1/webhook-deliveries/:delivery_id/result`
   - Agent reports delivery success/failure
   - Includes response status, timing, error details

3. **Webhook Subscription Schema Changes**:
   - Add `delivery_mode`: enum (`broker`, `agent`, `auto`) - default `broker`
   - Add `delivery_target_labels`: text[] (nullable) - labels for agent matching

4. **Broker Dispatch Logic Changes**:
   - Evaluate `delivery_mode` when creating delivery
   - For `agent` mode: set `target_agent_id` based on labels or event context
   - For `auto` mode: apply routing rules based on event type
   - For `broker` mode: existing behavior (immediate dispatch)

5. **Agent Webhook Delivery Loop**:
   - Poll for pending webhook deliveries (alongside work orders, diagnostics)
   - Execute HTTP calls with configured timeout/retries
   - Report results back to broker

6. **Delivery Status Tracking**:
   - `webhook_deliveries` table gains `target_agent_id` and `delivered_by_agent_id` columns
   - Status reflects agent delivery attempts

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Webhook subscriptions support `delivery_mode` field (`broker`, `agent`, `auto`)
- [ ] Webhook subscriptions support `delivery_target_labels` for label-based agent targeting
- [ ] Agent can poll for pending webhook deliveries assigned to it
- [ ] Agent successfully delivers webhooks to in-cluster services
- [ ] Agent reports delivery success/failure back to broker
- [ ] `broker` mode: existing behavior preserved (backwards compatible, default)
- [ ] `agent` mode with labels: routes to matching agent(s)
- [ ] `agent` mode without labels: routes based on event context
- [ ] `auto` mode: smart routing based on event type (agent failures → broker, health events → contextual agent)
- [ ] Delivery retries work correctly when agent delivers
- [ ] Webhook delivery audit trail includes which agent delivered

## Implementation Notes

### Database Changes

**`webhook_subscriptions` table:**
- Add `delivery_mode VARCHAR(10) NOT NULL DEFAULT 'broker'` - enum: broker, agent, auto
- Add `delivery_target_labels TEXT[]` - nullable, labels for agent matching

**`webhook_deliveries` table:**
- Add `target_agent_id UUID REFERENCES agents(id)` - nullable, which agent should deliver
- Add `delivered_by_agent_id UUID REFERENCES agents(id)` - nullable, which agent actually delivered

### Broker Changes

**New API Endpoints:**
- `GET /api/v1/agents/:id/webhooks/pending` - agent polls for assigned deliveries
- `POST /api/v1/webhook-deliveries/:id/result` - agent reports delivery result

**Dispatch Logic (`event_bus.rs` / `background_tasks.rs`):**
- When creating delivery, evaluate `delivery_mode`:
  - `broker`: set `target_agent_id = NULL`, broker background task delivers (existing flow)
  - `agent`: set `target_agent_id` based on labels or event context
  - `auto`: apply routing rules, set `target_agent_id` accordingly
- Broker delivery task: skip deliveries where `target_agent_id IS NOT NULL`
- Agent delivery query: `WHERE target_agent_id = :agent_id AND status = 'pending'`

**Event Context Extraction:**
- Health events: extract `agent_id` from event data
- Work order events: extract `claimed_by` agent from event data
- Broker-side events: no agent context, use broker delivery

### Agent Changes

**New polling loop** in `commands.rs`:
```rust
let mut webhook_interval = interval(Duration::from_secs(10));
_ = webhook_interval.tick() => {
    // fetch_pending_webhook_deliveries()
    // for each: deliver HTTP, report result
}
```

**New broker communication** in `broker.rs`:
- `fetch_pending_webhook_deliveries()` - GET pending deliveries
- `report_webhook_delivery_result()` - POST success/failure

**HTTP delivery logic:**
- Use reqwest with configurable timeout
- Classify errors as retryable (5xx, timeout, connection) vs non-retryable (4xx except 429)
- Report result with status code, error message, timing

### Risk Considerations

- **Agent offline**: Deliveries queue up with `target_agent_id` set. Need timeout/expiry or fallback to broker after N failed attempts.
- **Label matching no agents**: If no agents match labels, delivery should fail immediately or fall back to broker.
- **Ordering**: Webhooks may deliver out of order if multiple agents match. Document as expected behavior.
- **Performance**: Additional polling overhead on agents (mitigated by 10s interval, batching).

## Implementation Plan

### Step 1: Database Schema Changes
**File:** `crates/brokkr-models/migrations/13_webhooks/up.sql`

Add columns to `webhook_subscriptions`:
```sql
delivery_mode VARCHAR(10) NOT NULL DEFAULT 'broker',
delivery_target_labels TEXT[]
```

Add columns to `webhook_deliveries`:
```sql
target_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
delivered_by_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL
```

Add index for efficient agent polling:
```sql
CREATE INDEX idx_webhook_deliveries_target_agent
  ON webhook_deliveries(target_agent_id)
  WHERE target_agent_id IS NOT NULL AND status = 'pending';
```

### Step 2: Update Models
**File:** `crates/brokkr-models/src/models/webhooks.rs`

- Add `delivery_mode: String` and `delivery_target_labels: Option<Vec<String>>` to `WebhookSubscription`
- Add `target_agent_id: Option<Uuid>` and `delivered_by_agent_id: Option<Uuid>` to `WebhookDelivery`
- Update `NewWebhookSubscription` and `NewWebhookDelivery` structs

### Step 3: Update DAL Layer
**Files:**
- `crates/brokkr-broker/src/dal/webhook_subscriptions.rs`
- `crates/brokkr-broker/src/dal/webhook_deliveries.rs`

- Update CRUD operations to handle new fields
- Add `find_pending_for_agent(agent_id)` query
- Add `update_delivery_result(id, agent_id, result)` function

### Step 4: Update Broker API - Subscription Endpoints
**File:** `crates/brokkr-broker/src/api/v1/webhooks.rs`

- Accept `delivery_mode` and `delivery_target_labels` in create/update DTOs
- Validate `delivery_mode` is one of: `broker`, `agent`, `auto`
- Return new fields in responses

### Step 5: Add Agent Webhook Endpoints
**File:** `crates/brokkr-broker/src/api/v1/webhooks.rs`

New endpoint: `GET /api/v1/agents/:id/webhooks/pending`
- Authenticate agent by PAK
- Query deliveries where `target_agent_id = :agent_id AND status = 'pending' AND next_attempt_at <= now()`
- Return decrypted URL and auth header
- Limit batch size to 10

New endpoint: `POST /api/v1/webhook-deliveries/:id/result`
- Accept: `{ success: bool, status_code: Option<i32>, error: Option<String>, duration_ms: i64 }`
- Update delivery status, set `delivered_by_agent_id`
- Schedule retry on failure (if retryable)

### Step 6: Update Event Dispatch Logic
**File:** `crates/brokkr-broker/src/utils/event_bus.rs`

When creating `NewWebhookDelivery`:
1. Get subscription's `delivery_mode`
2. If `broker`: set `target_agent_id = NULL` (existing flow)
3. If `agent`: resolve agent from labels or event context
4. If `auto`: apply routing rules based on event type

Add helper: `resolve_delivery_agent(subscription, event) -> Option<Uuid>`

### Step 7: Modify Broker Delivery Task
**File:** `crates/brokkr-broker/src/utils/background_tasks.rs`

Change query in webhook delivery task:
```sql
WHERE status = 'pending'
  AND target_agent_id IS NULL  -- Only broker-delivered
  AND next_attempt_at <= now()
```

### Step 8: Agent - Add Webhook Polling Loop
**File:** `crates/brokkr-agent/src/cli/commands.rs`

Add to main `select!` loop:
```rust
let mut webhook_interval = interval(Duration::from_secs(10));

_ = webhook_interval.tick() => {
    if agent.status != "ACTIVE" { continue; }
    match webhooks::process_pending_deliveries(&config, &client, &agent).await {
        Ok(count) => { /* log if count > 0 */ }
        Err(e) => { /* log error */ }
    }
}
```

### Step 9: Agent - Broker Communication
**File:** `crates/brokkr-agent/src/webhooks.rs` (new file)

```rust
pub async fn fetch_pending_deliveries(client, agent_id) -> Result<Vec<WebhookDelivery>>
pub async fn report_delivery_result(client, delivery_id, result) -> Result<()>
pub async fn process_pending_deliveries(config, client, agent) -> Result<usize>
```

### Step 10: Agent - HTTP Delivery Logic
**File:** `crates/brokkr-agent/src/webhooks.rs`

```rust
async fn deliver_webhook(url, auth_header, payload, timeout_secs) -> DeliveryResult {
    // POST with configurable timeout
    // Classify errors: retryable (5xx, timeout) vs non-retryable (4xx)
    // Return status_code, error, duration_ms
}
```

### Files Summary

| File | Changes |
|------|---------|
| `crates/brokkr-models/migrations/13_webhooks/up.sql` | Add columns |
| `crates/brokkr-models/src/models/webhooks.rs` | Add fields to structs |
| `crates/brokkr-models/src/schema.rs` | Regenerate with diesel |
| `crates/brokkr-broker/src/api/v1/webhooks.rs` | New endpoints, update existing |
| `crates/brokkr-broker/src/api/v1/agents.rs` | Route new webhook endpoint |
| `crates/brokkr-broker/src/dal/webhook_subscriptions.rs` | Update queries |
| `crates/brokkr-broker/src/dal/webhook_deliveries.rs` | Add agent queries |
| `crates/brokkr-broker/src/utils/event_bus.rs` | Delivery routing logic |
| `crates/brokkr-broker/src/utils/background_tasks.rs` | Filter broker-only |
| `crates/brokkr-agent/src/cli/commands.rs` | Add webhook interval |
| `crates/brokkr-agent/src/webhooks.rs` | New file - delivery logic |
| `crates/brokkr-agent/src/lib.rs` | Export webhooks module |
| `tests/e2e/src/api.rs` | Add test helpers |
| `tests/e2e/src/scenarios.rs` | Add E2E test |

## Status Updates

- 2025-12-31: Bug identified during UAT demo planning. Webhooks cannot reach in-cluster services because broker makes HTTP calls directly.
- 2025-12-31: Design discussion - refined to use `delivery_mode` (broker/agent/auto) with label-based targeting instead of explicit agent IDs. Consistent with Brokkr's existing targeting model.
- 2025-12-31: Implementation complete. All changes implemented and unit tests passing:
  - Database schema updated with `delivery_mode`, `delivery_target_labels`, `target_agent_id`, `delivered_by_agent_id`
  - Broker API extended with agent webhook endpoints (`GET /agents/:id/webhooks/pending`, `POST /webhook-deliveries/:id/result`)
  - Event dispatch logic updated to route webhooks based on delivery mode
  - Agent webhooks module created with polling loop and HTTP delivery logic
  - All unit tests passing (models: 7/7, event_bus: 8/8, agent webhooks: 4/4)