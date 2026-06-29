---
id: broker-database-query-metrics
level: task
title: "Broker: database query metrics (brokkr_database_queries_total)"
short_code: "BROKKR-T-0265"
created_at: 2026-06-29T10:45:50.781507+00:00
updated_at: 2026-06-29T10:45:50.781507+00:00
parent: brokkr-operator-console
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0031
---

# Broker: database query metrics (brokkr_database_queries_total)

## Parent Initiative

[[BROKKR-I-0031]]

## Objective

Instrument the broker's Postgres/diesel layer with Prometheus metrics (query count +
latency) so the Operator Console Broker-health view can show a "DB queries / min" card
(from the Brokkr Monitor handoff) and operators can spot N+1 storms / pool saturation.

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: The data layer is currently un-instrumented — the broker exports HTTP,
  agent, WS, and entity-count metrics but **nothing** for the database. A query-rate +
  latency signal catches N+1 query storms and connection-pool saturation under load, which
  the existing metrics can't see.
- **Effort Estimate**: S–M (diesel instrumentation hook + 2 metrics; no API/SDK surface).

## Context / current state

`crates/brokkr-broker/src/metrics.rs` has no DB metric, and no diesel `Instrumentation`
is installed anywhere. The handoff's Broker-health view drew a "DB queries / min" card
that was omitted in I-0031 because no `brokkr_database_queries_total` exists.

## Acceptance Criteria

- [ ] `brokkr_database_queries_total` (counter) + `brokkr_database_query_duration_seconds`
      (histogram) registered in `metrics.rs` and exported on `/metrics`.
- [ ] A diesel [`Instrumentation`](https://docs.rs/diesel/latest/diesel/connection/trait.Instrumentation.html)
      hook attached to pooled connections (in `db.rs` pool builder) increments/observes on
      query finish; negligible overhead.
- [ ] (Optional) a pool-saturation gauge (in-use vs idle connections).
- [ ] Operator Console Broker-health view restores the "DB queries / min" card (rate) and
      adds a p95 query-latency card; pixel-verified via the web-e2e harness.
- [ ] No OpenAPI/SDK change (metrics are not part of the spec).

## Implementation Notes

### Technical Approach
diesel 2.1+ exposes the `Instrumentation` trait emitting events on query start/finish.
Implement a lightweight instrumentation that bumps the counter and observes elapsed time,
and install it on each pooled connection via the r2d2 pool customizer. Compute the
per-minute rate in the UI from the counter delta (the console already polls /metrics).

### Dependencies
None. Pure broker-internal observability; complements the existing `/metrics` cards.

## Status Updates

*To be added during implementation.*
