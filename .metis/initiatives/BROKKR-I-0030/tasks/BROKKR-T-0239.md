---
id: broker-provision-system-generator
level: task
title: "Broker: provision system generator at startup"
short_code: "BROKKR-T-0239"
created_at: 2026-06-26T13:33:19.611613+00:00
updated_at: 2026-06-26T17:27:29.923336+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Broker: provision system generator at startup

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] System generator is created idempotently at broker startup — running multiple times leaves exactly one system generator
- [ ] System generator UUID is persisted and stable across restarts (`is_system` column on `generators` table, or `system_config` key/value table)
- [ ] `DELETE /generators/:id` returns `403 cannot_delete_system_generator` when targeting the system generator
- [ ] System generator is excluded from `GET /generators` listing by default (or clearly marked `is_system=true`)
- [ ] Broker startup log emits the system generator UUID at `INFO` level
- [ ] `angreal models migrations` passes (up + redo) for whatever schema change is chosen

## Implementation Notes

### Technical Approach
Add `is_system BOOLEAN NOT NULL DEFAULT false` to the `generators` table via a migration. This is simpler than a separate `system_config` table and keeps the system generator queryable through the existing DAL. Add `provision_system_generator()` to the generators DAL — checks for an existing `is_system=true` row, creates one if absent. Call this from broker startup in `bin.rs` before the HTTP server begins accepting connections. Reserve the name `"__system__"` and make it non-editable. Guard `delete_generator` in `crates/brokkr-broker/src/api/v1/generators.rs` to reject requests targeting any generator where `is_system=true`.

### Dependencies
Independent — can start immediately. T-0242 and T-0244 block on this.

### Risk Considerations
Migration adding `is_system` defaults to `false` for all existing generators — fully backward compatible. Idempotent provisioning is critical: if the broker crashes mid-startup, the next restart must not create a second system generator.

## Status Updates

*To be added during implementation*

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

2026-06-26: Implementation complete pending migration verification.
- Migration `21_system_generator`: `ALTER TABLE generators ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT false`
- `Generator` model: added `is_system: bool` field
- `GeneratorsDAL`: `provision_system_generator()` (idempotent), `get_system_generator_id()`, `list()` now excludes system generators
- `commands.rs serve()`: calls `provision_system_generator()` after migrations, logs UUID at INFO
- `delete_generator` API: guards `is_system=true` with `403 cannot_delete_system_generator`
- `cargo check` clean; `angreal tests unit brokkr-models` 132/132 pass
- `angreal models migrations` running in background (Docker build)