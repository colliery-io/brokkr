---
id: broker-stop-panicking-on-db-pool
level: task
title: "Broker: stop panicking on DB pool exhaustion; add catch-panic layer"
short_code: "BROKKR-T-0209"
created_at: 2026-06-11T11:02:07.975727+00:00
updated_at: 2026-06-11T11:02:07.975727+00:00
parent: broker-api-correctness-error
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0024
---

# Broker: stop panicking on DB pool exhaustion; add catch-panic layer

## Parent Initiative

[[BROKKR-I-0024]]

## Objective

Every DAL method (225 sites, e.g. `dal/stacks.rs:70`) does `self.dal.pool.get().expect("Failed to get DB connection")`. Pool exhaustion or a DB outage panics inside the handler, and there is no `CatchPanicLayer` anywhere in the broker — the connection is dropped with no response, so under load the broker looks like it is hanging up on every client. The auth middleware (`middleware.rs:170-173`) already handles the same failure gracefully.

## Acceptance Criteria

- [ ] `pool.get()` returns an error from DAL methods (mapped to 500 `internal_error` at the API layer) — no `.expect` remains on pool acquisition in `dal/`.
- [ ] `tower_http::catch_panic::CatchPanicLayer` added to the router as belt-and-braces (any remaining panic → 500, logged).
- [ ] Test: with a pool of size 1 and a held connection, a request gets a 500 response (not a dropped connection).

## Implementation Notes

Mechanical but wide: changing the DAL signature ripples; consider a small `conn()?` helper on DAL returning `Result<PooledConnection, diesel::result::Error>` so call sites change minimally (`let conn = &mut self.conn()?;`). Coordinate with T-0207 so the two sweeps don't collide in the same files.

## Status Updates

*To be added during implementation*
