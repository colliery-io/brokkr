---
id: dal-query-instrumentation-for
level: task
title: "DAL query instrumentation for Prometheus metrics"
short_code: "BROKKR-T-0088"
created_at: 2025-12-30T14:15:11.801469+00:00
updated_at: 2025-12-30T14:15:11.801469+00:00
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

# DAL query instrumentation for Prometheus metrics

## Objective **[REQUIRED]**

Add Prometheus metrics instrumentation to DAL (Data Access Layer) methods to track database query counts and durations by query type (select, insert, update, delete).

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P3 - Low (when time permits)

### Business Justification

**What this would provide:**
- Query counts by type (select/insert/update/delete)
- Query duration histograms per operation type
- Ability to break down "of this 500ms request, how much was DB time vs other processing"

**Why this is currently low priority:**

1. **HTTP request latency already captures end-to-end time** - The existing HTTP metrics middleware (BROKKR-T-0087) records total request duration, which includes DB time. For operational monitoring, this is usually sufficient.

2. **Postgres has richer native metrics** - `pg_stat_statements`, connection pool stats, and other Postgres-level metrics are more accurate and detailed for DB performance analysis than application-side instrumentation.

3. **Brokkr's broker is a thin API layer** - Most operations are simple CRUD queries, not complex multi-query transactions. There's limited value in breaking down individual query timing.

4. **High implementation cost** - Every DAL method would need to be wrapped with timing/counting logic. This is significant boilerplate across ~20+ DAL modules.

**When to prioritize this work:**

1. **Complex multi-query transactions** - If we add operations involving multiple dependent queries where we need to identify which specific query is slow

2. **Limited Postgres access** - If deployments exist where operators cannot access Postgres metrics directly and need application-side visibility into DB performance

Until one of these conditions is met, this remains a "nice to have" that doesn't justify the implementation effort.

**Effort Estimate**: M (touches many files, repetitive but straightforward)

## Acceptance Criteria **[REQUIRED]**

- [ ] All DAL methods instrumented with query timing
- [ ] `brokkr_database_queries_total` counter incremented per query (labels: query_type)
- [ ] `brokkr_database_query_duration_seconds` histogram records latency (labels: query_type)
- [ ] Metrics visible in `/metrics` output after DB operations

## Implementation Notes

### Technical Approach

Wrap DAL methods with timing:

```rust
pub fn get(&self, id: Uuid) -> Result<Option<Agent>, Error> {
    let start = Instant::now();
    let result = // ... existing query logic
    metrics::record_db_query("select", start.elapsed().as_secs_f64());
    result
}
```

Alternative: Create a macro or wrapper function to reduce boilerplate.

### Files to Modify
- `crates/brokkr-broker/src/dal/*.rs` - All DAL modules
- Metrics helpers already exist in `crates/brokkr-broker/src/metrics.rs`

## Status Updates **[REQUIRED]**

### 2025-12-30: Created as backlog item
Deferred during BROKKR-T-0087 implementation. Core metrics (HTTP, agent operations) completed; DAL instrumentation provides marginal additional value for current use cases.