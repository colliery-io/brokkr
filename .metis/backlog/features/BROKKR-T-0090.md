---
id: uat-demo-end-to-end-workflow
level: task
title: "UAT Demo: End-to-end workflow demonstrating complete Brokkr feature set"
short_code: "BROKKR-T-0090"
created_at: 2025-12-31T02:37:15.926480+00:00
updated_at: 2025-12-31T16:16:31.784598+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/todo"


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

**Entry Point:** `angreal local up` → Navigate to Admin Panel → Click "Demo"

### Phase 1: Environment Validation
- Verify docker-compose stack is healthy
- Verify K3s cluster is accessible
- Verify Tekton Pipelines are installed and running
- Verify Shipwright Build is installed with ClusterBuildStrategies available
- Verify initial agent (from docker-compose) is ACTIVE and heartbeating

### Phase 2: Build Work Order - Webhook Server
**Goal:** Use Shipwright to build our webhook-catcher container image

1. Create a `build` type work order with:
   - Git source: `examples/webhook-catcher/` (or similar simple Go/Node app)
   - Strategy: `buildah` or `kaniko` ClusterBuildStrategy
   - Output: Push to local registry (`localhost:5050/webhook-catcher:demo`)
2. Submit work order to the initial agent
3. Poll work order status until COMPLETED
4. Display: BuildRun name, duration, image digest

### Phase 3: Deploy Secondary Agent via Brokkr
**Goal:** Demonstrate Brokkr deploying Brokkr (agent self-deployment)

1. Create a generator + stack for "demo-agents"
2. Create deployment object containing:
   - ServiceAccount with RBAC permissions
   - ConfigMap with agent configuration
   - Deployment for brokkr-agent pointing to broker
3. Target the stack to the initial agent (by label or explicit target)
4. Wait for initial agent to apply the deployment
5. Wait for secondary agent to register with broker and become ACTIVE
6. Display: New agent name, cluster, heartbeat status

### Phase 4: Deploy Webhook Server via Secondary Agent
**Goal:** Deploy the built webhook-catcher image using the NEW agent

1. Create a generator + stack for "webhook-apps"
2. Create deployment object containing:
   - Namespace: `demo-webhooks`
   - Deployment: webhook-catcher using built image
   - Service: ClusterIP exposing the webhook endpoint
3. Add label to target the SECONDARY agent specifically
4. Wait for deployment to be applied
5. Poll deployment health until pods are Ready
6. Display: Pod status, service endpoint

### Phase 5: Configure Webhook Subscription
**Goal:** Set up Brokkr to send events to our webhook server

1. Create webhook subscription:
   - Name: `demo-webhook-receiver`
   - URL: `http://webhook-catcher.demo-webhooks.svc.cluster.local:8080/webhook`
   - Events: `deployment.created`, `deployment.updated`, `health.changed`
   - Enabled: true
2. Display: Webhook ID, subscribed events

### Phase 6: Trigger Webhook Events
**Goal:** Create activity that fires webhooks and verify receipt

1. Create a simple deployment (just a Namespace or ConfigMap)
2. Target it to any agent
3. Wait for deployment to be processed
4. Query webhook-catcher's `/messages` endpoint (via port-forward or ingress)
5. Display: Received webhook payloads with timestamps

### Phase 7: Verification & Cleanup Summary
**Goal:** Show what was demonstrated and current state

Display summary:
- ✅ Build work order: Built `webhook-catcher:demo` image
- ✅ Agent deployment: Secondary agent running in K3s
- ✅ Application deployment: Webhook server running via secondary agent
- ✅ Webhook delivery: N messages received by webhook server

Optional: Offer cleanup button to remove demo resources

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

- [[BROKKR-T-0091]] - Webhook delivery should occur from agent (data plane) not broker (control plane)

## Status Updates

- 2025-12-31: Ticket created. Demo currently uses fake database records instead of real functionality.