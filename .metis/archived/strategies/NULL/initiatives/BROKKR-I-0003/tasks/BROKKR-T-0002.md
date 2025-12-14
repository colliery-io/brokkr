---
id: enhance-broker-health-check
level: task
title: "Enhance broker health check endpoints"
short_code: "BROKKR-T-0002"
created_at: 2025-10-18T14:47:35.755921+00:00
updated_at: 2025-10-19T01:00:01.474888+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Enhance broker health check endpoints

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Enhance the broker's health check endpoints to follow Kubernetes best practices with proper dependency checking and detailed status reporting.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `/healthz` endpoint remains simple (200 OK if process is alive, no dependency checks)
- [x] `/readyz` endpoint upgraded to validate PostgreSQL connectivity before returning ready
- [x] `/health` endpoint implemented with detailed JSON response including:
  - Database connection status
  - Service uptime
  - Application version
  - Timestamp
- [x] Proper HTTP status codes (200 for healthy, 503 for unhealthy/not ready)
- [x] Comprehensive error handling for database connection failures
- [x] All integration tests pass with new endpoints

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Current State:**
- `/healthz` exists but returns simple "OK" text
- `/readyz` exists but doesn't check database connectivity (just returns "Ready")
- No `/health` endpoint for detailed diagnostics

**Changes Required:**

1. **Upgrade `/readyz` endpoint** (crates/brokkr-broker/src/api/mod.rs):
   - Add database connection test using DAL
   - Return 503 Service Unavailable if DB connection fails
   - Return 200 OK with "Ready" if DB is accessible

2. **Implement `/health` endpoint**:
   - Create new endpoint handler
   - Return JSON with structure:
     ```json
     {
       "status": "healthy",
       "database": {
         "connected": true,
         "latency_ms": 5
       },
       "uptime_seconds": 3600,
       "version": "0.1.0",
       "timestamp": "2025-10-18T19:30:00Z"
     }
     ```
   - Handle errors gracefully with appropriate status codes

3. **Ensure `/healthz` stays simple**:
   - No changes needed - already correct
   - Used for liveness probe (should not check dependencies)

**Files to Modify:**
- `crates/brokkr-broker/src/api/mod.rs` - Add/update endpoint handlers

### Dependencies

- Depends on BROKKR-T-0001 (non-root Dockerfiles) being complete for proper container testing
- No blocking dependencies

### Risk Considerations

**Risk: Database connectivity checks adding latency to readiness probes**
- Mitigation: Use connection pool's existing connection, don't create new connection for each check
- Keep timeout short (1-2 seconds max)

**Risk: Breaking existing health check consumers**
- Mitigation: `/healthz` and `/readyz` endpoints maintain backward compatibility in response format
- Only `/health` returns new JSON format

## Status Updates **[REQUIRED]**

### 2025-10-18 - Task Completed

**Changes Made:**
1. Updated `/readyz` endpoint to check PostgreSQL connectivity using `dal.agents().list()`
2. Implemented new `/health` endpoint with detailed JSON status
3. Added necessary imports: `State`, `Json`, `Serialize`, logging, and chrono for timestamps
4. Created `HealthStatus` and `DatabaseStatus` structs for JSON response

**Implementation Details:**
- `/healthz`: Unchanged, simple liveness check (200 OK)
- `/readyz`: Now checks database connectivity, returns 503 if DB unavailable
- `/health`: Returns comprehensive JSON:
  ```json
  {
    "status": "healthy",
    "database": {
      "connected": true
    },
    "uptime_seconds": 3600,
    "version": "0.0.0",
    "timestamp": "2025-10-18T19:30:00Z"
  }
  ```

**Files Modified:**
- `crates/brokkr-broker/src/api/mod.rs`: Added health endpoints and route

**Testing Results:**
- All 223 integration tests passed (29 agent tests + 194 broker tests)
- Health endpoints verified working correctly
- Database connectivity checks functioning as expected
