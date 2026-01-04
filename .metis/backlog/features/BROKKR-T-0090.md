---
id: uat-demo-end-to-end-workflow
level: task
title: "UAT Demo: End-to-end workflow demonstrating complete Brokkr feature set"
short_code: "BROKKR-T-0090"
created_at: 2025-12-31T02:37:15.926480+00:00
updated_at: 2026-01-02T20:06:06.305841+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# UAT Demo: End-to-end workflow demonstrating complete Brokkr feature set

## Objective

Create a comprehensive UAT (User Acceptance Testing) demo workflow in the UI that exercises the COMPLETE Brokkr feature set using REAL functionality - not fake database records. This demo serves as both a validation tool and a showcase of platform capabilities.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P0 - Critical (validates core product functionality)

### Business Justification
- **User Value**: Provides confidence that all advertised features actually work end-to-end
- **Business Value**: Serves as UAT validation, demo for stakeholders, and regression testing
- **Effort Estimate**: L (requires coordination of multiple subsystems)

## Demo Workflow Overview

**Entry Point:** `angreal local up` → Navigate to **Demo tab** (dedicated first-class tab)

## Updated Implementation Plan (2025-12-31)

### UI Design: Dedicated Demo Tab

The demo is now a **self-contained tab** (not buried in Admin) with:
- Visual phase cards showing real-time progress
- Status indicators (pending → running → complete)
- Live webhook event stream
- Cleanup button to remove demo resources

### Phase 1: Environment Check
**Purpose:** Validate prerequisites before demo starts
- Check broker API health (`/healthz`)
- Find and verify integration agent is ACTIVE with recent heartbeat
- Verify Shipwright/Tekton are installed (ClusterBuildStrategies exist)

**Visual:** Green checkmarks with live status indicators

### Phase 2: Container Build
**Purpose:** Demonstrate Shipwright container builds via Brokkr
- Create `build` type work order for webhook-catcher image
- Target to integration agent
- Poll work order status: PENDING → CLAIMED → BUILDING → COMPLETED
- Show BuildRun progress with duration timer
- Result: `registry:5000/webhook-catcher:demo` image ready

**Visual:** Progress bar with status transitions

### Phase 3: Agent Deployment
**Purpose:** Demonstrate Brokkr deploying another Brokkr agent
- Register new agent (`demo-staging-agent`) → get PAK
- Create stack with agent deployment YAML (ServiceAccount, RBAC, Deployment)
- Target stack to integration agent
- Poll for new agent to become ACTIVE with heartbeat

**Visual:** Step-by-step progress with heartbeat indicator

### Phase 4: Stack & Deployments
**Purpose:** Show core deployment workflow
- Create generator
- Create stack with labels
- Deploy multi-document YAML (nginx + http-echo services)
- Target to integration agent
- Show deployment status transitions

**Visual:** Deployment status indicators (Pending → Applied → Healthy)

### Phase 5: Webhooks
**Purpose:** Demonstrate event-driven notifications
- Create webhook subscription for multiple event types
- Show real-time webhook events as they're received
- Poll webhook-catcher `/messages` endpoint
- Display auto-scrolling event stream with timestamps

**Visual:** Live-updating event log

### Phase 6: Templates
**Purpose:** Show reusable configuration patterns
- Create parameterized template (ConfigMap with variables)
- Instantiate with sample parameters
- Show resulting deployment

**Visual:** Template → Parameters → Result flow

### Cleanup Function
Dedicated cleanup button that removes in dependency order:
1. Remove agent targets
2. Delete webhook subscriptions
3. Create deletion markers for deployments
4. Delete stacks
5. Delete templates
6. Deactivate demo agents (keep for audit)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Demo runs from clean `angreal local up` without manual intervention
- [ ] Each phase displays clear progress and status
- [ ] Build work order creates real BuildRun and waits for completion
- [ ] Secondary agent registers and heartbeats to broker
- [ ] Webhook server is deployed via the secondary agent (not initial)
- [ ] Webhook events are actually delivered and can be queried
- [ ] Demo handles errors gracefully with clear error messages
- [ ] Demo can be re-run without conflicts (idempotent or cleans up first)

## Features Exercised

| Feature | How It's Tested |
|---------|-----------------|
| Agent registration | Secondary agent registers via Brokkr-deployed manifests |
| Agent heartbeat | Heartbeat indicator shows connection state |
| Build work orders | Shipwright Build + BuildRun for webhook-catcher |
| Stack targeting (labels) | Stacks targeted to specific agents by label |
| Deployment objects | Multi-document YAML applied to cluster |
| Deployment health | Poll for pod Ready status |
| Webhooks | Subscription + delivery verification |
| Work order status | Polling until COMPLETED/FAILED |

## Implementation Notes

### Required Components

1. **Webhook Catcher App** (`examples/webhook-catcher/`)
   - Simple HTTP server that:
     - `POST /webhook` - Receives and stores webhook payloads
     - `GET /messages` - Returns received messages as JSON
     - `GET /healthz` - Health check endpoint
   - Include Dockerfile for Shipwright build

2. **Demo Script Updates** (`examples/ui-slim/src/api.js`)
   - `runDemoSetup()` - Orchestrates all phases
   - `waitForBuildCompletion()` - Polls build work order
   - `waitForAgentActive()` - Polls for new agent registration
   - `waitForDeploymentHealthy()` - Polls deployment health
   - `queryWebhookMessages()` - Fetches received webhooks

3. **UI Updates** (`examples/ui-slim/src/App.js`)
   - Demo progress panel with phase indicators
   - Real-time status updates
   - Error display with details

### Technical Dependencies

- **BROKKR-T-0089**: Shipwright test coverage (build work orders must work)
- Tekton Pipelines v0.59+ installed in K3s
- Shipwright Build v0.18.1 with sample strategies
- Local container registry accessible from K3s

### Sequencing

```
angreal local up
       ↓
   K3s + Tekton + Shipwright ready
       ↓
   Initial agent ACTIVE
       ↓
   [Phase 2] Build webhook-catcher image
       ↓
   [Phase 3] Deploy secondary agent
       ↓
   Secondary agent ACTIVE
       ↓
   [Phase 4] Deploy webhook-catcher via secondary agent
       ↓
   Webhook server Ready
       ↓
   [Phase 5] Create webhook subscription
       ↓
   [Phase 6] Create deployment → webhook fires → verify receipt
       ↓
   [Phase 7] Display summary
```

### Risk Considerations

- Build phase is slow (2-5 minutes) - need clear progress indication
- Secondary agent startup may take 30-60 seconds
- Webhook delivery has retry logic - may need to wait for delivery
- K3s networking: webhook-catcher must be reachable from broker

## Blocked By

- ~~[[BROKKR-T-0091]] - Webhook delivery should occur from agent (data plane) not broker (control plane)~~ ✅ COMPLETED

## Files to Modify

| File | Changes |
|------|---------|
| `examples/ui-slim/src/App.js` | Add DemoPanel component (~300 lines), add 'demo' to nav |
| `examples/ui-slim/src/api.js` | Add phased demo functions, cleanup function |
| `examples/ui-slim/src/styles.css` | Add `.demo-phase`, `.demo-step`, `.event-stream` styles |
| `examples/webhook-catcher/` | ✅ Already created (Python app) |
| `.angreal/files/docker-compose.yaml` | ✅ Already updated (webhook-catcher service) |

## Status Updates

- 2025-12-31: Ticket created. Demo currently uses fake database records instead of real functionality.
- 2025-12-31: BROKKR-T-0091 completed - webhook agent-side delivery now implemented. Blocker removed.
- 2025-12-31: Plan updated to 6-phase dedicated Demo tab. webhook-catcher app created. Implementation starting.
- 2025-12-31: **Implementation complete.** DemoPanel component added to App.js (~600 lines), CSS styles added, API functions added. Demo tab now available in UI navigation. Ready for testing.