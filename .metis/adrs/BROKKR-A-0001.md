---
id: 001-generic-work-system-for-transient
level: adr
title: "Generic Work System for Transient Operations"
number: 1
short_code: "BROKKR-A-0001"
created_at: 2025-10-17T09:48:19.059489+00:00
updated_at: 2025-10-17T09:48:19.059489+00:00
decision_date: 2025-10-17
decision_maker: Dylan Storey
parent:
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Generic Work System for Transient Operations

## Context **[REQUIRED]**

Brokkr's agent/broker architecture currently handles persistent deployment state management through the stacks and deployment_objects system. However, there are operational needs for managing transient, one-time work that doesn't fit the persistent state model:

- **Container image builds**: Execute once, produce artifact, complete
- **Test execution**: Run tests, report results, clean up
- **Backup operations**: Perform backup, verify, finish
- **Database migrations**: Execute migration, validate, complete

These operations share common characteristics:
- Ephemeral: Execute once and complete (not continuously reconciled)
- Status-based: Progress through states (pending → running → completed/failed)
- Time-bounded: Have defined start and end points
- Result-oriented: Produce outcomes rather than maintain desired state
- Retry-capable: May need retry logic for transient failures

Without a generic system, each new transient operation type would require:
- Custom broker tables and DAL logic
- New API endpoints and patterns
- Duplicated retry/failure handling
- Different approaches to work assignment and targeting

## Decision **[REQUIRED]**

Implement a generic work system in the broker for managing transient operations, separate from the persistent deployment state system. This system will:

1. Store work items as opaque CRD specifications (text) in broker database
2. Use the same targeting patterns as stacks (work_order_targets table)
3. Provide unified retry logic (max retries, stale claim detection, exponential backoff)
4. Support multiple work types through a type discriminator field
5. Maintain consistent broker API patterns across all work types

The first implementation will be container image builds (BuildRequest), establishing patterns for future work types.

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **Generic work system** (chosen) | Extensible to new work types; consistent patterns; single retry logic; reusable targeting | More upfront design; additional abstraction layer | Low | 2-3 weeks |
| **Build-specific tables only** | Simpler initial implementation; no abstraction | Duplicate logic for each new work type; inconsistent patterns; technical debt | Medium | 1-2 weeks (per type) |
| **Extend deployment_objects** | Reuse existing tables; no new patterns | Semantic confusion (builds aren't deployments); conflates persistent and transient state | High | 1 week |
| **Separate service** | Complete isolation; independent scaling | Duplicates agent communication; adds operational complexity; more moving parts | High | 6-8 weeks |

## Rationale **[REQUIRED]**

The generic work system was chosen because:

1. **Extensibility**: Clear path for adding test execution, backups, migrations without schema changes
2. **Consistency**: All ephemeral work uses same broker APIs, targeting logic, and retry mechanisms
3. **Separation of concerns**: Clear distinction between persistent state (deployments) and transient operations (work)
4. **Proven patterns**: Leverages existing stack targeting approach (work_order_targets mirrors agent_targets)
5. **Future-proof**: Small upfront investment prevents accumulating technical debt from work-type-specific implementations

The upfront design cost (2-3 weeks) is offset by avoiding repeated implementation for each new work type and establishing consistent patterns across the platform.

## Consequences **[REQUIRED]**

### Positive
- New ephemeral work types can be added with minimal broker changes (just new work_type values)
- Consistent API patterns across all transient operations
- Single implementation of retry logic, failure handling, and work targeting
- Clear semantic boundary between persistent state and transient operations
- CRD specifications stored as opaque text (broker doesn't need to understand work details)
- Enables future work types: test runs, backup operations, database migrations, etc.

### Negative
- Additional abstraction layer adds initial complexity
- More tables than build-specific approach (but shared across work types)
- Requires careful interface design to accommodate different work type needs
- Some broker logic must handle polymorphic work types

### Neutral
- Database migration 08 adds work_orders and work_order_targets tables
- Work assignment follows same agent targeting patterns as stacks
- CRD lifecycle managed by specialized operators (not broker responsibility)
