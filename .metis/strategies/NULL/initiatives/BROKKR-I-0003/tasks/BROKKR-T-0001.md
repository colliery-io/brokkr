---
id: update-dockerfiles-for-non-root
level: task
title: "Update Dockerfiles for non-root execution"
short_code: "BROKKR-T-0001"
created_at: 2025-10-18T14:47:35.679703+00:00
updated_at: 2025-10-18T19:23:03.505867+00:00
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

# Update Dockerfiles for non-root execution

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Update both broker and agent Dockerfiles to run as non-root user (UID/GID 10001:10001) to follow security best practices and meet production deployment requirements. Ensure all integration tests pass with the new configuration.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Both Dockerfiles (broker and agent) create non-root user with UID/GID 10001:10001
- [x] Both Dockerfiles use USER directive to switch to non-root user before ENTRYPOINT
- [x] File ownership and permissions correctly set for binaries (brokkr-broker, brokkr-agent, kubectl)
- [x] All integration tests pass (29 agent tests + 194 broker tests = 223 total)
- [x] Init containers in docker-compose override to run as root where needed for volume permissions
- [x] No file write permission errors in containers



## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Dockerfile Changes (broker and agent):**
1. Added `groupadd` and `useradd` commands to create non-root user with UID/GID 10001:10001
2. Updated file ownership using `chown brokkr:brokkr` for all binaries
3. Added `USER brokkr` directive before ENTRYPOINT to switch from root to non-root user
4. Agent Dockerfile also chowns kubectl since it needs to be executable by non-root user

**Files Modified:**
- `docker/Dockerfile.broker`: Added non-root user configuration
- `docker/Dockerfile.agent`: Added non-root user configuration + kubectl permissions
- `.angreal/files/docker-compose.yaml`: Added `user: "0:0"` override for init-agent service

**Integration Test Compatibility:**
The init-agent service in docker-compose failed initially because it runs as a one-time setup task that needs to write to shared volumes (created by k3s as root). Solution: Added `user: "0:0"` override specifically for the init-agent service, which is acceptable for init containers that need elevated permissions for setup tasks.

### Dependencies
None - this task is foundational and doesn't depend on other Phase 1 tasks.

### Risk Considerations

**Risk: Volume permission issues in production deployments**
- Mitigation: Helm charts (BROKKR-T-0006, BROKKR-T-0007) will need to configure fsGroup in securityContext to ensure volumes are accessible by UID 10001

**Risk: Applications attempting file writes to read-only locations**
- Mitigation: Verified through integration tests that applications don't write to filesystem (stateless containers)

**Risk: Init containers needing special permissions**
- Mitigation: Documented pattern for overriding user in docker-compose; Helm charts will handle init containers appropriately

## Status Updates **[REQUIRED]**

### 2025-10-18 - Task Completed

**Changes Made:**
1. Updated `docker/Dockerfile.broker` to add non-root user (UID/GID 10001:10001)
2. Updated `docker/Dockerfile.agent` to add non-root user and set kubectl permissions
3. Fixed `.angreal/files/docker-compose.yaml` init-agent service with `user: "0:0"` override for volume write permissions

**Testing Results:**
- All 223 integration tests passed (29 agent tests + 194 broker tests)
- No permission errors in running containers
- Containers verified running as UID 10001 (non-root)

**Key Findings:**
- Init containers in docker-compose need explicit root override when writing to shared volumes
- This pattern will inform Helm chart implementation in BROKKR-T-0006 and BROKKR-T-0007
- fsGroup securityContext will be needed in Helm charts for volume permissions
