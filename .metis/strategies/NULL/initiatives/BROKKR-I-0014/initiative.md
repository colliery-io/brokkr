---
id: documentation-quality-overhaul
level: initiative
title: "Documentation Quality Overhaul"
short_code: "BROKKR-I-0014"
created_at: 2026-01-04T14:34:26.165030+00:00
updated_at: 2026-01-04T15:05:45.433690+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: documentation-quality-overhaul
---

# Documentation Quality Overhaul Initiative

## Context

A comprehensive review of Brokkr documentation revealed three categories of issues requiring attention before the next release. The documentation has grown organically alongside feature development, resulting in inconsistent style, some accuracy gaps between documented behavior and actual implementation, and missing documentation for several implemented features.

## Goals & Non-Goals

**Goals:**
- Convert bullet-heavy documentation to flowing prosaic narrative that explains concepts in context
- Correct terminology discrepancies and add implementation status clarity to ADRs
- Create missing documentation for implemented features (generators, health monitoring, stacks, etc.)

**Non-Goals:**
- API reference auto-generation tooling
- Documentation site infrastructure changes
- Translations or internationalization

## Detailed Design

### Style Improvements

The following files require conversion from bullet-heavy format to prosaic paragraphs:

| File | Priority |
|------|----------|
| `README.md` (root) | High |
| `docs/content/getting-started/quick-start.md` | High |
| `docs/content/explanation/core-concepts.md` | High |
| `docs/content/explanation/components.md` | Medium |
| `charts/brokkr-broker/README.md` | Medium |
| `charts/brokkr-agent/README.md` | Medium |

Reference documentation with tables (configuration, API endpoints) should retain tabular format where appropriate.

How-to guides should retain numbered procedural steps (recipe format) but improve the explanatory prose between steps. The prosaic style conversion primarily targets explanation documents and conceptual overviews.

### Accuracy Fixes

**Terminology:**
- ADR-1 (BROKKR-A-0001): Update "ephemeral_work_targets" to "work_order_targets"
- work-orders.md: Clarify work_type values for Shipwright builds

**Implementation Status:**
- ADR-3 (OpenTelemetry): Add status indicating metrics implemented, tracing planned
- ADR-4 (Multi-Tenancy): Add status indicating this is planned, not implemented

**Missing Details:**
- work-orders.md: Document `last_error`, `last_error_at` fields and claim TTL recovery
- webhooks.md: Document AES-256-GCM encryption and 7-day retention policy

### Missing Documentation

**Critical:**
- `how-to/generators-and-ci-cd.md` - Generator lifecycle, PAK rotation, CI/CD integration
- `reference/generators-api.md` - Complete API documentation

**High Priority:**
- `how-to/deployment-health-monitoring.md` - Health status configuration and interpretation
- `how-to/managing-stacks.md` - Stack creation, labels, targeting
- `reference/soft-deletion.md` - Pattern explanation and unique constraint behavior

**Medium Priority:**
- `how-to/understanding-reconciliation.md` - Reconciliation loop and sequence IDs
- `reference/audit-logs.md` - Audit log schema and access patterns

## Implementation Plan

**Phase 1: Accuracy & Terminology** - Quick wins fixing existing content
**Phase 2: Critical Missing Docs** - Generators and health monitoring
**Phase 3: Style Improvements** - Prosaic rewrites of key files
**Phase 4: Remaining Documentation** - Stacks, soft-delete, reconciliation, audit logs