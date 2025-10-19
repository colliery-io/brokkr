---
id: implement-agent-health-check
level: task
title: "Implement agent health check endpoints"
short_code: "BROKKR-T-0003"
created_at: 2025-10-18T14:47:35.871478+00:00
updated_at: 2025-10-19T01:17:58.588493+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Implement agent health check endpoints

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Add HTTP server capability to the agent (currently CLI-only) and implement health check endpoints matching the broker's pattern for Kubernetes liveness and readiness probes.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] HTTP server added to agent (currently CLI-only application)
- [x] `/healthz` endpoint implemented (simple liveness check, returns 200 OK if process alive)
- [x] `/readyz` endpoint implemented with Kubernetes API connectivity validation
- [x] `/health` endpoint implemented with detailed JSON status including:
  - Kubernetes API connection status
  - Broker connection status
  - Service uptime
  - Application version
  - Timestamp
- [x] Endpoints match broker patterns for consistency
- [x] Proper HTTP status codes (200 healthy, 503 not ready/unhealthy)
- [x] All integration tests pass with new endpoints

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Current State:**
- Agent is CLI-only application (no HTTP server)
- Runs as continuous process polling broker for work
- No health check endpoints exist

**Architecture Decision:**
The agent needs dual operation modes:
1. **Main loop**: Existing agent functionality (polling broker, reconciling K8s)
2. **HTTP server**: Health check endpoints running concurrently

**Changes Required:**

1. **Add HTTP server using Axum** (matching broker's framework):
   - Create new module `crates/brokkr-agent/src/health.rs`
   - Implement lightweight Axum server on configurable port (default 8080)
   - Run HTTP server in separate tokio task alongside main agent loop

2. **Implement `/healthz` endpoint**:
   - Simple 200 OK response if process is running
   - No dependency checks (liveness probe)

3. **Implement `/readyz` endpoint**:
   - Test Kubernetes API connectivity using existing K8s client
   - Return 503 if K8s API unreachable
   - Return 200 OK if K8s API accessible

4. **Implement `/health` endpoint**:
   - JSON response with comprehensive status:
     ```json
     {
       "status": "healthy",
       "kubernetes": {
         "connected": true,
         "latency_ms": 10
       },
       "broker": {
         "connected": true,
         "last_heartbeat": "2025-10-18T19:30:00Z"
       },
       "uptime_seconds": 3600,
       "version": "0.1.0",
       "timestamp": "2025-10-18T19:30:00Z"
     }
     ```

5. **Update agent main.rs**:
   - Start HTTP server in background task
   - Share state between main loop and health endpoints (Arc<Mutex<AgentState>>)
   - Ensure graceful shutdown of both HTTP server and main loop

**Files to Create/Modify:**
- `crates/brokkr-agent/src/health.rs` - New health endpoint module
- `crates/brokkr-agent/src/bin.rs` or `main.rs` - Start HTTP server alongside main loop
- `crates/brokkr-agent/Cargo.toml` - Add axum dependency

### Dependencies

- Depends on BROKKR-T-0001 (non-root Dockerfiles) for container testing
- Depends on BROKKR-T-0002 (broker health endpoints) for consistent patterns

### Risk Considerations

**Risk: HTTP server port conflicts in constrained environments**
- Mitigation: Make port configurable via environment variable
- Document port requirements for Helm chart configuration

**Risk: Shared state between HTTP server and main loop causing deadlocks**
- Mitigation: Use tokio RwLock or minimal Arc<Mutex> with very short critical sections
- Consider message passing if state sharing becomes complex

**Risk: Health checks impacting agent performance**
- Mitigation: Health endpoints should be read-only operations
- Use existing clients, don't create new connections for each check

## Status Updates **[REQUIRED]**

### 2025-10-18: Implementation Complete

**What was done:**
- Created new `crates/brokkr-agent/src/health.rs` module with three health check endpoints:
  - `/healthz`: Simple liveness check (200 OK if process alive)
  - `/readyz`: Readiness check with Kubernetes API connectivity validation
  - `/health`: Detailed JSON status with K8s, broker, uptime, version, and timestamp
- Updated `crates/brokkr-agent/src/lib.rs` to expose health module
- Modified `crates/brokkr-agent/src/cli/commands.rs` to:
  - Start HTTP server on configurable port (default 8080) alongside main agent loop
  - Share broker status between main loop and health endpoints using Arc<RwLock>
  - Update broker status on heartbeat success/failure for health endpoint reporting
- Added `health_port` configuration field to agent config in `crates/brokkr-utils/src/config.rs`
- Added default health_port (8080) to `crates/brokkr-utils/default.toml`
- Updated `docker/Dockerfile.agent` to expose port 8080
- Added healthcheck to brokkr-agent service in `.angreal/files/docker-compose.yaml`
- Fixed imports to use `axum::http::StatusCode` instead of `hyper::StatusCode`
- All 223+ integration tests passed

**Technical Notes:**
- HTTP server runs as background tokio task spawned alongside main agent loop
- Health state shared via `HealthState` struct containing K8s client and broker status
- Kubernetes API health checked using `k8s_client.apiserver_version()` (lightweight call)
- Broker status updated in real-time during heartbeat interval
- Matches broker health endpoint patterns for consistency

**Acceptance Criteria Status:**
- [x] HTTP server added to agent (currently CLI-only application)
- [x] `/healthz` endpoint implemented (simple liveness check, returns 200 OK if process alive)
- [x] `/readyz` endpoint implemented with Kubernetes API connectivity validation
- [x] `/health` endpoint implemented with detailed JSON status
- [x] Endpoints match broker patterns for consistency
- [x] Proper HTTP status codes (200 healthy, 503 not ready/unhealthy)
- [x] All integration tests pass with new endpoints
