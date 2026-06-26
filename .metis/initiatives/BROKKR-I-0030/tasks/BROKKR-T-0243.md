---
id: agent-brokkr-generator-ids-env-var
level: task
title: "Agent: BROKKR_GENERATOR_IDS env var and startup self-registration"
short_code: "BROKKR-T-0243"
created_at: 2026-06-26T13:33:34.359210+00:00
updated_at: 2026-06-26T17:55:05.348709+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Agent: BROKKR_GENERATOR_IDS env var and startup self-registration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Agent reads `BROKKR_GENERATOR_IDS` on startup; if unset or empty, starts normally with no error
- [ ] Value parsed as comma-separated UUIDs; malformed entries log `WARN` and are skipped — not fatal
- [ ] Agent calls `POST /generators/:id/register` for each valid UUID using its own PAK
- [ ] A `409` response (already registered) is treated as success — fully idempotent
- [ ] A non-409 error response logs `ERROR` with the generator ID and HTTP status, but does not prevent agent startup
- [ ] Startup log at `INFO` level lists which generator IDs were registered and which (if any) failed

## Implementation Notes

### Technical Approach
Add parsing in the agent startup sequence — likely `crates/brokkr-agent/src/cli/commands.rs` or `bin.rs`, before the main reconciliation loop starts. Use `std::env::var("BROKKR_GENERATOR_IDS").unwrap_or_default()`, split on `','`, trim whitespace, parse each with `Uuid::parse_str()`. Call the register endpoint via the existing `broker_sdk` client (once T-0244 is merged and the SDK is updated to include the register method). Run registrations sequentially at startup — no need for concurrency here.

### Dependencies
Blocked by T-0244 (register endpoint must exist and broker SDK must expose it).

### Risk Considerations
If the broker is temporarily unreachable at agent startup, registration will fail. Log clearly and continue — the agent can retry on the next restart, or operators can register manually. Do not make startup registration blocking/fatal.

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

## Status Updates **[REQUIRED]**

*To be added during implementation*