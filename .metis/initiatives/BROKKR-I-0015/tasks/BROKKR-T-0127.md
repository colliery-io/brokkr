---
id: domain-8-adr-accuracy-verification
level: task
title: "Domain 8: ADR Accuracy Verification"
short_code: "BROKKR-T-0127"
created_at: 2026-03-13T14:01:21.542897+00:00
updated_at: 2026-03-13T14:12:47.643983+00:00
parent: BROKKR-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0015
---

# Domain 8: ADR Accuracy Verification

## Parent Initiative

[[BROKKR-I-0015]] — Documentation Validation Against Implementation

## Objective

Verify that each Architecture Decision Record (ADR) accurately reflects the current implementation state. ADRs are particularly prone to drift — a decision may have been made and recorded, but the implementation may have diverged, been partially completed, or evolved beyond what was decided. Each ADR's status annotations, technical claims, and described approach must be checked against what was actually built.

## Documents in Scope

- `BROKKR-A-0001` — Generic Work System for Transient Operations (decided)
- `BROKKR-A-0002` — Operator Sidecar Pattern for Agent Capabilities (decided)
- `BROKKR-A-0003` — OpenTelemetry for Vendor-Agnostic Observability (decided)
- `BROKKR-A-0004` — Schema-Per-Tenant Multi-Tenancy Architecture (decided)
- `BROKKR-A-0005` — Shipwright Build Integration for Container Image Builds (decided)

## Source of Truth

- The actual implementation in the Rust codebase for each decision area
- Cargo.toml dependencies (do the dependencies match the decided technology choices?)
- Database migrations (do schema decisions match what was migrated?)
- Feature flags or conditional compilation (any decisions partially implemented behind flags?)

## Verification Checklist

### Per-ADR Verification

**BROKKR-A-0001: Generic Work System**
- [ ] The work order/work system described matches the actual implementation in code
- [ ] Entity names and terminology used in the ADR match the codebase
- [ ] The "transient operations" pattern described is how work orders actually function
- [ ] Any noted limitations or future work items are still accurate

**BROKKR-A-0002: Operator Sidecar Pattern**
- [ ] The sidecar pattern described is actually implemented (or accurately noted as planned)
- [ ] If implemented, the sidecar communication mechanism matches the ADR description
- [ ] Operator capabilities described match what the sidecar actually does
- [ ] Any dependencies on external operators (e.g., Shipwright) match reality

**BROKKR-A-0003: OpenTelemetry**
- [ ] Status annotation ("metrics implemented, tracing planned") is still accurate
- [ ] Metrics implementation uses OpenTelemetry or Prometheus as described
- [ ] If tracing has been implemented since the annotation, update the status
- [ ] Vendor-agnostic claim is accurate — no vendor lock-in in observability code
- [ ] Dependencies (`opentelemetry`, `prometheus`, `tracing` crates) match what's described

**BROKKR-A-0004: Schema-Per-Tenant Multi-Tenancy**
- [ ] Status annotation ("planned, not implemented") is still accurate
- [ ] If any multi-tenancy work has been done, update the status
- [ ] The described approach still aligns with the current schema and migration structure
- [ ] Any prerequisite work mentioned is accurately characterized

**BROKKR-A-0005: Shipwright Build Integration**
- [ ] The Shipwright integration described matches the actual build work order implementation
- [ ] Tekton/Shipwright dependencies are present in the codebase as described
- [ ] Build pipeline flow described matches the actual code path
- [ ] Any limitations or caveats noted are still accurate

### Cross-ADR Consistency
- [ ] No two ADRs contradict each other
- [ ] ADR references to each other are accurate
- [ ] The collective set of ADRs accurately represents the project's architectural decisions
- [ ] No significant architectural decisions exist in the code that lack an ADR

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every ADR read and every technical claim verified against the codebase
- [ ] Status annotations verified as current or updated
- [ ] Implementation state of each decision accurately reflected
- [ ] Any undocumented architectural decisions identified (gap inventory)
- [ ] All findings recorded using verdict taxonomy
- [ ] All non-CORRECT findings fixed in the ADR documents

## Findings Report

### BROKKR-A-0001: Generic Work System for Transient Operations

| Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| Decision | Work items stored as opaque text in broker database | CORRECT | `crates/brokkr-models/migrations/08_work_orders/up.sql` | Column is `yaml_content TEXT NOT NULL` |
| Decision | Uses same targeting patterns as stacks (work_order_targets table) | CORRECT | `crates/brokkr-models/migrations/08_work_orders/up.sql` | work_order_targets mirrors agent_targets |
| Decision | Unified retry logic (max retries, stale claim detection, exponential backoff) | CORRECT | `crates/brokkr-models/src/models/work_orders.rs` | Fields: max_retries, retry_count, backoff_seconds, claim_timeout_seconds |
| Decision | Type discriminator field for multiple work types | CORRECT | `crates/brokkr-models/src/models/work_orders.rs` | `work_type VARCHAR(50)`, currently "build" and "custom" supported |
| Decision | First implementation is container image builds (BuildRequest) | DIVERGED | `crates/brokkr-agent/src/work_orders/mod.rs` | Uses Shipwright Build/BuildRun, not custom BuildRequest. Terminology superseded by ADR-5 |
| Consequences | "Database migration 08 adds work_orders and work_order_targets tables" | INACCURATE | Migrations 08 + 09 | **FIXED**: Migration 08 has work_orders + work_order_log + work_order_targets; Migration 09 adds labels/annotations |

Implementation State: Fully Implemented
Status Annotation Accurate: Yes (no status annotation present)

### BROKKR-A-0002: Operator Sidecar Pattern for Agent Capabilities

| Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| Status | "Superseded for Builds" by ADR-5 | CORRECT | N/A | No buildah-operator crate exists; agent uses Shipwright directly |
| Status | Sidecar pattern remains valid for future non-build capabilities | CORRECT | N/A | Architectural pattern not invalidated |
| Decision | Agent container: poll broker, claim work, apply CRDs, report status | CORRECT | `crates/brokkr-agent/src/work_orders/mod.rs` | Agent does exactly this |
| Consequences | "Each capability becomes a separate Rust crate (e.g., `brokkr-buildah-operator`)" | NOT APPLICABLE | `crates/` | Only 4 crates exist: agent, broker, models, utils. No operator crate built (superseded) |

Implementation State: Superseded for Builds (as documented); Pattern available for future use
Status Annotation Accurate: Yes

### BROKKR-A-0003: OpenTelemetry for Vendor-Agnostic Observability

| Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| Status | "Phase 1 (Prometheus metrics): Complete" | CORRECT | `crates/brokkr-broker/src/metrics.rs`, `crates/brokkr-agent/src/metrics.rs` | Full Prometheus metrics in both broker and agent |
| Status | "Phase 2 (OTLP export): Not implemented" | **STALE** | `crates/brokkr-utils/src/telemetry.rs` | **FIXED**: OTLP export IS implemented via opentelemetry-otlp |
| Status | "Phase 3 (Distributed tracing): Not implemented" | **STALE** | `crates/brokkr-utils/src/telemetry.rs` | **FIXED**: Tracing infrastructure complete (subscriber + OTel layer); trace context propagation not yet implemented |
| Context | "Broker has stub /metrics endpoint" | **STALE** | `crates/brokkr-broker/src/metrics.rs` | **FIXED**: Full Prometheus metrics implementation |
| Context | "Agent has no observability endpoints" | **STALE** | `crates/brokkr-agent/src/health.rs` | **FIXED**: Agent has /metrics endpoint |
| Decision | "Add opentelemetry-prometheus crate" | DIVERGED | `Cargo.toml` | Metrics use `prometheus` crate directly, not `opentelemetry-prometheus`. Not a problem, just different approach |
| Decision | Vendor-agnostic claim | CORRECT | All telemetry code | Prometheus for metrics (pull), OTLP for traces (push). No vendor lock-in |
| Decision | Dependencies match | PARTIALLY | `Cargo.toml` | Has: opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing, tracing-opentelemetry, prometheus. Missing: opentelemetry-prometheus (using native prometheus instead) |

Implementation State: Substantially Implemented (Phases 1-2 complete, Phase 3 infrastructure ready)
Status Annotation Accurate: No — **FIXED** to reflect current state

### BROKKR-A-0004: Schema-Per-Tenant Multi-Tenancy Architecture

| Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| Status | "Not Implemented" | **STALE** | `crates/brokkr-broker/src/db.rs` | **FIXED**: Core infrastructure IS implemented |
| Decision | ConnectionPool gains schema awareness | CORRECT | `crates/brokkr-broker/src/db.rs` | `pub schema: Option<String>` field present |
| Decision | Automatic schema routing via SET search_path | CORRECT | `crates/brokkr-broker/src/db.rs` | `ConnectionPool::get()` sets search_path |
| Decision | Schema from configuration (BROKKR__DATABASE__SCHEMA) | CORRECT | `crates/brokkr-utils/src/config.rs` | `Database.schema: Option<String>` with env var |
| Decision | Backward compatible (schema: None uses public) | CORRECT | `crates/brokkr-broker/tests/integration/db/multi_tenant.rs` | Test `test_backward_compatibility_no_schema` verifies this |
| Consequences | Schema name validation to prevent SQL injection | CORRECT | `crates/brokkr-broker/src/db.rs` | `validate_schema_name()` function exists |

Implementation State: Core Infrastructure Implemented (connection pool, schema routing, configuration, validation, integration tests)
Status Annotation Accurate: No — **FIXED** from "Not Implemented" to "Core Infrastructure Implemented"

### BROKKR-A-0005: Shipwright Build Integration for Container Image Builds

| Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| Decision | Agent creates Shipwright Build/BuildRun CRDs | CORRECT | `crates/brokkr-agent/src/work_orders/build.rs` | Uses kube-rs dynamic API to create Build and BuildRun |
| Decision | Agent watches BuildRun status and reports to broker | CORRECT | `crates/brokkr-agent/src/work_orders/build.rs` | `watch_buildrun_completion()` polls BuildRun status |
| Decision | Agent remains single-container (no sidecar) | CORRECT | N/A | No operator sidecar crate or code exists |
| Decision | work_type: 'shipwright-build' | INACCURATE | `crates/brokkr-models/src/models/work_orders.rs` | **FIXED**: Actual work_type is 'build', not 'shipwright-build' |
| Decision | Shipwright v1beta1 API | CORRECT | `crates/brokkr-agent/src/work_orders/build.rs` | `SHIPWRIGHT_API_VERSION: &str = "v1beta1"` |
| Decision | Agent uses kube-rs with Shipwright CRDs | CORRECT | `crates/brokkr-agent/Cargo.toml` | kube = 0.95.0, dynamic API discovery used |
| Consequences | "Work type identifier: 'shipwright-build'" | INACCURATE | `crates/brokkr-agent/src/work_orders/mod.rs` | **FIXED**: Work type is 'build' |
| Consequences | "BuildRun status conditions map to broker ephemeral work status" | CORRECT | `crates/brokkr-agent/src/work_orders/build.rs` | BuildRunStatus mapped to success/failure for broker completion |

Implementation State: Implemented (Shipwright Build/BuildRun integration working via kube-rs dynamic API)
Status Annotation Accurate: Yes (no explicit status annotation; decision reflects reality)

### Cross-ADR Consistency

| Check | Verdict | Notes |
|---|---|---|
| No contradictions between ADRs | CORRECT | ADR-2 correctly notes supersession by ADR-5; ADR-1 and ADR-5 complementary |
| ADR references accurate | CORRECT | ADR-5 references ADR-2 supersession correctly; ADR-3 references BROKKR-T-0019 |
| Collective set represents architectural decisions | MOSTLY CORRECT | Webhook system (migration 13), audit logging (migration 16), deployment health (migration 10), and diagnostics (migration 11) lack ADRs but may not warrant them |
| Undocumented decisions | NOTED | Config hot-reload (notify crate, config_watcher.rs), soft-delete pattern, encryption (aes-gcm), and event bus system are implemented without ADRs |

## Status Updates

### 2026-03-13: Verification Complete

**Summary**: All 5 ADRs reviewed against codebase. Found significant status annotation drift in ADRs 3 and 4, and a minor terminology mismatch in ADR 5.

**Corrections Made:**
1. **BROKKR-A-0001**: Fixed migration description (08 includes 3 tables, 09 adds labels/annotations)
2. **BROKKR-A-0003**: Updated status from "Partially Implemented" to "Substantially Implemented"; corrected all 3 phase statuses; updated stale "Current state" section; corrected Phase 1-3 implementation details
3. **BROKKR-A-0004**: Updated status from "Not Implemented" to "Core Infrastructure Implemented" with detailed implementation inventory
4. **BROKKR-A-0005**: Corrected work_type from 'shipwright-build' to 'build' in two locations (Decision and Consequences sections)

**Key Finding**: ADRs 3 and 4 had the most significant drift — implementation substantially outpaced the status annotations. ADR-3's status was 2 phases behind reality, and ADR-4 was marked as "Not Implemented" despite having full infrastructure including integration tests.