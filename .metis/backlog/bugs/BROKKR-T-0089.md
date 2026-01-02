---
id: shipwright-build-work-orders-have
level: task
title: "Shipwright Build work orders have zero test coverage"
short_code: "BROKKR-T-0089"
created_at: 2025-12-31T02:30:39.536371+00:00
updated_at: 2025-12-31T02:30:39.536371+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Shipwright Build work orders have zero test coverage

## Objective

Add comprehensive test coverage for the Shipwright Build work order feature, which currently has zero tests despite being a documented feature.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment

- **Affected Users**: All users attempting to use `build` type work orders
- **Severity**: The feature is completely untested and may not work at all

**Root Cause Analysis:**

1. The bundled Tekton helm chart (OpenFunction v0.37.2) uses `gcr.io` images which now return 403 Forbidden (Tekton migrated to `ghcr.io/tektoncd` in April 2025)
2. The bundled Shipwright helm chart (v0.10.0) is significantly outdated (current: v0.18.1)
3. E2E tests only exercise `"custom"` type work orders with basic Kubernetes Jobs
4. No unit tests exist for `crates/brokkr-agent/src/work_orders/build.rs`

**Evidence (before fix):**

| Component | Test Coverage |
|-----------|---------------|
| `build.rs` (329 lines) | 0 tests |
| E2E `test_work_orders` | Only tests `"custom"` type |
| Integration tests | None for Shipwright |

**After fix (current state):**

| Component | Test Coverage |
|-----------|---------------|
| `build.rs` | 22 unit tests |
| E2E `test_build_work_orders` | Tests `"build"` type work orders |
| Integration tests | Pending (requires cluster restart) |

**Files with zero test coverage:**
- `crates/brokkr-agent/src/work_orders/build.rs` - entire module untested

**Untested code paths:**
- `execute_build()` - main entry point
- `apply_shipwright_resource()` - applies Build CRs to cluster
- `create_buildrun()` - creates BuildRun from Build
- `watch_buildrun_completion()` - polls for build completion
- Error handling for build failures
- Image digest extraction

## Acceptance Criteria

- [x] Unit tests for `build.rs` with mocked K8s client covering:
  - [x] YAML parsing for Build resources (8 tests)
  - [x] BuildRun creation (2 tests for name generation)
  - [x] Status polling logic (deserialization tests)
  - [x] Success path (digest extraction) - 2 tests
  - [x] Failure path (error extraction) - 4 tests
  - [ ] ~~Timeout handling~~ (requires async mocking, deferred)
- [x] E2E test that creates a `build` type work order
- [ ] Integration test with actual Shipwright in K3s dev environment
- [x] Update bundled helm chart dependencies to working versions:
  - [x] Tekton: Using official manifests v0.68.1 (ghcr.io images)
  - [x] Shipwright: Using official manifests v0.18.1
- [ ] CI pipeline includes build work order tests

## Implementation Notes

### Technical Approach

1. **Unit Tests**: Use `kube` crate's mock client or `k8s-openapi` test utilities to mock Shipwright CRD responses
2. **E2E Tests**: Add `test_build_work_orders()` scenario that:
   - Creates a Build resource via work order
   - Verifies BuildRun is created
   - Waits for completion (use ttl.sh for ephemeral images)
3. **Infrastructure**: Update docker-compose to install Tekton/Shipwright from official sources

### Dependencies

- Tekton Pipelines v0.59+ (required by Shipwright v0.18.1)
- Shipwright Build v0.18.1
- ClusterBuildStrategies (sample strategies)

### Risk Considerations

- Build tests are slow (container builds take 1-5 minutes)
- May need to skip in CI or use mock strategies
- ttl.sh has rate limits for ephemeral image storage

## Status Updates

- 2025-12-31: Bug identified during demo testing. Bundled helm charts completely broken due to gcr.io migration.
- 2025-12-31: Added 22 unit tests to `build.rs` covering YAML parsing, status deserialization, and status interpretation logic.
- 2025-12-31: Added `test_build_work_orders()` E2E scenario (enabled via `SHIPWRIGHT_ENABLED=true`).
- 2025-12-31: Updated docker-compose to use official Tekton v0.68.1 and Shipwright v0.18.1 manifests.