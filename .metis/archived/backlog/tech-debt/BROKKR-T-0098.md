---
id: complete-prometheus-metrics
level: task
title: "Complete Prometheus metrics instrumentation"
short_code: "BROKKR-T-0098"
created_at: 2025-12-30T14:00:56.825238+00:00
updated_at: 2025-12-30T14:15:57.988511+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Complete Prometheus metrics instrumentation

## Objective **[REQUIRED]**

Wire up the existing Prometheus metrics definitions to actually record data during operations. Currently metrics are defined but never incremented - `/metrics` returns all zeros.

Related: ADR BROKKR-A-0003 (OpenTelemetry for Observability)

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: 
  - `/metrics` endpoint exists but returns empty/zero metrics
  - No visibility into HTTP request rates, latencies, error rates
  - No visibility into database query performance
  - No visibility into agent polling or K8s operations
  - Operators deploying Brokkr expect working Prometheus metrics
  
- **Benefits of Fixing**: 
  - Standard observability for Brokkr deployments
  - Grafana dashboards can show real data
  - Alerting on error rates, latency spikes
  - Capacity planning based on actual usage
  
- **Risk Assessment**: 
  - Low risk of not fixing immediately (system works without it)
  - Reputational risk if users expect metrics and get zeros

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Broker: HTTP request counter incremented on each request (endpoint, method, status)
- [ ] Broker: HTTP request duration histogram records latency
- [ ] Broker: Database query counter incremented per query type
- [ ] Broker: Database query duration histogram records latency  
- [ ] Broker: Active agents gauge reflects actual count
- [ ] Broker: Agent heartbeat age gauge updated per agent
- [ ] Agent: Poll request counter incremented (success/error)
- [ ] Agent: Poll duration histogram records latency
- [ ] Agent: K8s operation counter incremented per operation type
- [ ] Agent: K8s operation duration histogram records latency
- [ ] Agent: Heartbeat counter incremented on send
- [ ] `/metrics` returns non-zero values after activity

## Implementation Notes

### Technical Approach

**Broker HTTP metrics** - Add middleware layer that:
- Records request count with labels (endpoint, method, status)
- Times request duration and records to histogram
- Best approach: Axum middleware or Tower layer

**Broker DB metrics** - Instrument DAL methods:
- Wrap queries with timing
- Increment counters by query type (select, insert, update, delete)

**Broker state metrics** - Periodic background task or on-demand:
- Count active agents (heartbeat within threshold)
- Update heartbeat age gauges per agent

**Agent metrics** - Instrument existing code:
- Poll loop: time each poll, increment success/error counter
- K8s client: wrap apply/delete/get operations with timing
- Heartbeat: increment counter on send

### Files to Modify

**Broker:**
- `crates/brokkr-broker/src/api/mod.rs` - Add metrics middleware
- `crates/brokkr-broker/src/dal/*.rs` - Instrument DB operations
- `crates/brokkr-broker/src/api/v1/agents.rs` - Update agent gauges on heartbeat

**Agent:**
- `crates/brokkr-agent/src/lib.rs` - Instrument poll loop
- `crates/brokkr-agent/src/reconciler.rs` - Instrument K8s operations (if exists)

### Dependencies
- Metrics definitions already exist in `metrics.rs` files
- No new crate dependencies needed

## Status Updates **[REQUIRED]**

*To be added during implementation*