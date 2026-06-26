---
id: api-post-and-delete-generators-id
level: task
title: "API: POST and DELETE /generators/:id/register endpoints"
short_code: "BROKKR-T-0244"
created_at: 2026-06-26T13:33:36.138973+00:00
updated_at: 2026-06-26T17:44:13.302303+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# API: POST and DELETE /generators/:id/register endpoints

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /generators/:id/register` with agent PAK registers the calling agent; returns `201` with the registration record; `409` if already registered
- [ ] `POST /generators/:id/register` with admin PAK requires an `agent_id` UUID body field; registers that agent; same response codes
- [ ] `DELETE /generators/:id/register` with agent PAK deregisters the calling agent; cascades all `agent_targets` rows for that generator's stacks for this agent; returns `204`
- [ ] `DELETE /generators/:id/register` with admin PAK requires an `agent_id` body field; same cascade behaviour
- [ ] Both endpoints return `404` if generator `:id` does not exist
- [ ] Both endpoints return `403` if a generator PAK tries to call them (generators cannot register/deregister agents)
- [ ] All events written to audit log with actor type `agent` or `admin` as appropriate
- [ ] Routes wired into `generators.rs` router and added to the OpenAPI spec with utoipa annotations

## Implementation Notes

### Technical Approach
Add `register_agent` and `deregister_agent` handlers in `crates/brokkr-broker/src/api/v1/generators.rs`. Auth logic: if `auth.agent.is_some()` → agent registers/deregisters itself; if `auth.admin` → parse `agent_id` from request body. For `DELETE`, after removing the registration row via `dal.agent_generator_registrations().delete(agent_id, generator_id)`, call `dal.agent_generator_registrations().delete_for_agent_and_generator(agent_id, generator_id)` to cascade the `agent_targets` cleanup (implemented in T-0241). Wrap the two deletes in a transaction.

### Dependencies
Blocked by T-0241 (registration DAL and cascade-delete helper). T-0243 and T-0246 block on this.

### Risk Considerations
The `DELETE` cascade must be transactional — if the `agent_targets` cleanup fails, the registration row should not be removed either, or the agent will be in a state where it has no registration but still has active targets.

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