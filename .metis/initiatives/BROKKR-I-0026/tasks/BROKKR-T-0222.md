---
id: broker-dal-stop-panicking-on-pool
level: task
title: "Broker DAL: stop panicking on pool.get() (225 sites) — deferred from T-0209"
short_code: "BROKKR-T-0222"
created_at: 2026-06-11T15:53:18.843524+00:00
updated_at: 2026-06-11T15:53:18.843524+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
---

# Broker DAL: stop panicking on pool.get() (225 sites) — deferred from T-0209

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Every DAL method does `self.dal.pool.get().expect("Failed to get DB connection")` (225 sites across 24 files in crates/brokkr-broker/src/dal/). Pool exhaustion or a DB outage panics inside the handler. [[BROKKR-T-0209]] added a `CatchPanicLayer` (outermost) so this now returns a clean 500 instead of dropping the connection — the operational symptom is fixed. This task removes the panic at the source so there is no unwind and the error is a normal 500 result.

## Backlog Item Details

### Type
- [x] Tech Debt

### Priority
- [x] P2 - Medium (CatchPanicLayer already covers the user-facing symptom)

## Acceptance Criteria

- [ ] No `pool.get().expect(...)` remains in `crates/brokkr-broker/src/dal/`.
- [ ] A DAL `conn()` helper returns `Result<PooledConnection, diesel::result::Error>` (or the DAL error type), mapping the r2d2 pool error — there is NO `From<r2d2::Error> for diesel::result::Error`, so the helper must construct an appropriate diesel error (e.g. `DatabaseError(UnableToSendCommand, ...)`) or the DAL error surface changes. Call sites become `let conn = &mut self.dal.conn()?;`.
- [ ] Integration test: with a pool of size 1 and a held connection, a request returns 500 (not a dropped connection).

## Implementation Notes

Coordinate with T-0207's files (same handlers/DAL). Mechanical but wide; the main design choice is the pool-error→diesel-error mapping. Reference middleware.rs:170-173 which already handles pool.get() gracefully.

## Status Updates

*To be added during implementation*
