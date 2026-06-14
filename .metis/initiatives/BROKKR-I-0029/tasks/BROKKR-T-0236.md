---
id: file-code-tickets-for-openapi-auth
level: task
title: "File code tickets for OpenAPI auth-vs-enforcement mismatches"
short_code: "BROKKR-T-0236"
created_at: 2026-06-14T16:08:33.763320+00:00
updated_at: 2026-06-14T16:31:31.190155+00:00
parent: BROKKR-I-0029
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0029
---

# File code tickets for OpenAPI auth-vs-enforcement mismatches

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0029]]

## Objective **[REQUIRED]**

Record the **code** discrepancies found during the I-0029 docs audit so they get fixed in the source (separate from the docs PR). The docs now describe the **actual enforced behavior**; these are spec/code bugs where the OpenAPI annotations (or a runtime path) don't match enforcement. These are NOT part of the docs PR.

### Findings (verified against source by the accuracy review)
1. **agent_events list/get** — OpenAPI `security` declares admin+agent+generator, but code enforces **admin-only** (`crates/brokkr-broker/src/api/v1/agent_events.rs:51,89`). Fix the OpenAPI annotation to match (admin-only).
2. **agents `search_agent`** (`GET /agents/`) — OpenAPI declares admin-only, but code also permits the matching agent (`agents.rs:282`). Reconcile annotation with enforcement.
3. **agents `add_target`/`remove_target`** — OpenAPI declares admin+generator, code is admin OR the **owning** generator (`agents.rs:785`). Tighten the annotation to owning-generator semantics.
4. **`POST /admin/config/reload`** — if hot-reload is disabled, the missing `Extension<ReloadableConfig>` makes the handler **500 before** the admin check runs (`v1/mod.rs:72-74`). Should fail gracefully (e.g. 404/501) and after auth.
5. **`complete_work_order`** — returns **202** `{"status":"retry_scheduled"}` at runtime in addition to 200 (`work_orders.rs:645`); OpenAPI documents only 200. Add the 202 response.
6. **Stale module doc** — `crates/brokkr-models/src/models/agents.rs` header comment (lines ~14-25) omits the three k8s connectivity columns; refresh to match the struct (agents.rs:60-91) + migration 20.

These are likely best handled as a small "OpenAPI annotation drift + minor API correctness" code initiative/backlog, scheduled independently of the docs work.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

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

## Acceptance Criteria

## Acceptance Criteria

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