---
id: operational-observability-metrics
level: initiative
title: "Agent Fleet Operational Data"
short_code: "BROKKR-I-0013"
created_at: 2025-12-30T13:55:33.567012+00:00
updated_at: 2025-12-30T13:55:33.567012+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: operational-observability-metrics
---

# Agent Fleet Operational Data Initiative

## Context **[REQUIRED]**

As Brokkr serves as a lynchpin component for dynamic orchestration platforms, operators and orchestrators need visibility into agent fleet health and resource availability. This is **platform data** (part of Brokkr's data model), not system telemetry (covered separately by OTEL integration).

Currently agents report:
- Heartbeat (liveness)
- Status (basic state)
- Labels/annotations (static targeting metadata)

Missing:
- Cluster resource availability (CPU/memory headroom)
- Agent-side error rates and health indicators
- Richer reconciliation state on deployment objects
- Broker-computed queue depths and backpressure metrics

This data enables:
- Faster debugging when deployments fail ("is this an infra problem?")
- Orchestrator visibility into fleet capacity
- Operational dashboards via Brokkr API (not external monitoring)

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Enrich agent heartbeat with optional resource/health snapshot
- Store and expose agent operational state via broker API
- Expose broker-computed metrics (queue depth, backpressure) via API
- Enrich deployment object status with reconciliation details (retry count, failure reason, duration)

**Non-Goals:**
- Using this data for placement decisions (labels handle targeting)
- Validating agent-reported data against reality (trust the agent)
- Replacing OTEL telemetry (that's for monitoring Brokkr itself)
- Real-time streaming (polling/API access is sufficient)

## Use Cases **[REQUIRED]**

### UC-1: Debugging Failed Deployments
- **Actor**: Platform operator
- **Scenario**: Deployments to agent X are failing. Operator queries `/agents/X/operational-status` and sees cluster is at 95% memory.
- **Outcome**: Root cause identified as infrastructure capacity, not application issue.

### UC-2: Fleet Capacity Planning
- **Actor**: Orchestration platform
- **Scenario**: Before scheduling new workloads, orchestrator queries `/fleet/summary` to see aggregate resource availability across agents.
- **Outcome**: Informed decision about where capacity exists.

### UC-3: Identifying Stuck Reconciliations
- **Actor**: Platform operator
- **Scenario**: Operator queries deployment objects and sees one with `retry_count: 47` and `failure_reason: "timeout"`.
- **Outcome**: Can investigate specific failure vs. wondering "why isn't this applying?"

## Detailed Design **[REQUIRED]**

### Agent-Reported Data (Heartbeat Enrichment)

Optional fields agents can include in heartbeat payload:

| Field | Type | Description |
|-------|------|-------------|
| `cluster_cpu_capacity` | int | Total CPU millicores in cluster |
| `cluster_cpu_available` | int | Available CPU millicores |
| `cluster_memory_capacity` | int | Total memory bytes in cluster |
| `cluster_memory_available` | int | Available memory bytes |
| `node_count` | int | Number of nodes in cluster |
| `node_ready_count` | int | Number of Ready nodes |
| `k8s_api_healthy` | bool | Can agent reach K8s API |
| `k8s_api_latency_ms` | int | Last observed API latency |
| `agent_uptime_seconds` | int | Agent process uptime |
| `reconciliation_errors_total` | int | Cumulative error count |
| `last_successful_reconcile` | timestamp | When last object succeeded |

All fields optional - agents report what they can. Broker stores latest snapshot per agent.

### Broker-Computed Data (API Exposed)

Broker computes and exposes via API:

| Metric | Description |
|--------|-------------|
| Queue depth per agent | Pending deployment objects awaiting pickup |
| Work order backlog | Pending/in-progress work orders per agent |
| Heartbeat staleness | Time since last heartbeat |
| Delivery latency | Time from object creation to agent acknowledgment |

### Deployment Object Status Enrichment

Extend object status with:

| Field | Description |
|-------|-------------|
| `retry_count` | Number of reconciliation attempts |
| `last_attempt_at` | Timestamp of last attempt |
| `failure_reason` | Categorized error (if failed) |
| `reconciliation_duration_ms` | How long last attempt took |

### API Endpoints

- `GET /agents/{id}/operational-status` - Agent-reported health data
- `GET /agents/{id}/queue-status` - Broker-computed queue metrics
- `GET /fleet/summary` - Aggregate fleet health overview

## Alternatives Considered **[REQUIRED]**

### Push all data to OTEL
Rejected: OTEL is for system telemetry consumed by external monitoring. This is platform data consumed by Brokkr API clients. Different audiences, different access patterns.

### Store full time-series history
Rejected: Broker stores latest snapshot only. Historical trending belongs in external time-series DBs via OTEL export. Keeps Brokkr simple.

### Make resource reporting mandatory
Rejected: Agents in constrained environments may not have access to cluster metrics. Optional fields with graceful degradation is more practical.

## Implementation Plan **[REQUIRED]**

### Phase 1: Broker-Computed Metrics
- Add queue depth computation per agent
- Add work order backlog queries
- Expose via new API endpoints
- No agent changes required

### Phase 2: Heartbeat Enrichment
- Define optional heartbeat payload schema
- Agent-side: collect and report resource metrics
- Broker-side: store latest snapshot per agent
- New `/agents/{id}/operational-status` endpoint

### Phase 3: Reconciliation Status Enrichment
- Extend deployment object status fields
- Track retry counts, failure reasons, durations
- Update object status on each reconciliation attempt

### Phase 4: Fleet Summary
- Aggregate endpoint for fleet-wide view
- Summary statistics across all agents