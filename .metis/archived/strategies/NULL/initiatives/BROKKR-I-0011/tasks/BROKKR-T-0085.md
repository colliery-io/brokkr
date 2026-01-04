---
id: create-data-flows-md-documentation
level: task
title: "Create data-flows.md documentation"
short_code: "BROKKR-T-0085"
created_at: 2025-12-30T02:06:09.122671+00:00
updated_at: 2025-12-30T02:22:52.193482+00:00
parent: BROKKR-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0011
---

# Create data-flows.md documentation

## Parent Initiative

[[BROKKR-I-0011]] - Architecture Reference Documentation

## Objective

Create a new documentation page with sequence diagrams and explanations showing how data flows through the Brokkr system, from deployment creation through to resource application in target clusters.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Document deployment object lifecycle (create → target → apply → status)
- [ ] Document event flow (agent events → broker → webhooks)
- [ ] Document authentication flows (PAK verification, admin tokens, generators)
- [ ] Include sequence diagrams for each major flow
- [ ] File created at `docs/content/explanation/data-flows.md`

## Documentation Sections

### Content Outline

1. **Deployment Lifecycle**
   - Sequence diagram: Stack creation → Deployment object creation → Agent targeting
   - Sequence diagram: Agent poll → Fetch deployment objects → Apply to cluster → Report status
   - State transitions for deployment objects

2. **Event Flow**
   - Agent event creation and reporting
   - Event storage in broker database
   - Webhook dispatch for subscribed events
   - Audit log generation

3. **Authentication Flows**
   - PAK (Pre-Authentication Key) verification sequence
   - Admin token authentication
   - Generator API key authentication
   - Token refresh/rotation (if applicable)

4. **Reconciliation Loop**
   - Agent polling interval
   - Desired state vs actual state comparison
   - Apply/update/delete decision logic
   - Status reporting back to broker

5. **Work Order Flow** (if applicable)
   - Work order creation
   - Agent execution
   - Result reporting

### Location

Create: `docs/content/explanation/data-flows.md`

## Implementation Notes

- Use Mermaid sequence diagrams for clarity
- Reference actual API endpoints and database tables where helpful
- Keep diagrams focused - one per major flow