---
id: broker-enforce-generator
level: task
title: "Broker: enforce generator registration in authorize_target_mutation"
short_code: "BROKKR-T-0246"
created_at: 2026-06-26T13:33:39.478105+00:00
updated_at: 2026-06-26T17:48:45.062403+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Broker: enforce generator registration in authorize_target_mutation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `authorize_target_mutation` in `crates/brokkr-broker/src/api/v1/agents.rs` checks `dal.agent_generator_registrations().is_registered(agent_id, stack.generator_id)` after the existing ownership check
- [ ] Unregistered agent returns `403` with error code `agent_not_registered` — applies to all callers including admin
- [ ] Registered agent passes through to existing logic unchanged
- [ ] System generator stacks pass naturally (auto-registration in T-0242 ensures every agent is always registered with the system generator — no special-case code needed)
- [ ] Integration test: admin targeting an unregistered agent → `403 agent_not_registered`
- [ ] Integration test: admin targeting a registered agent → succeeds
- [ ] All existing targeting integration tests in `tests/integration/api/agents.rs` continue to pass without modification (existing agents will be auto-registered with the system generator via back-fill in T-0248)

## Implementation Notes

### Technical Approach
Edit `authorize_target_mutation` at `crates/brokkr-broker/src/api/v1/agents.rs` lines ~785-816. After the existing stack ownership check, look up the agent record to get its ID (it's already available from `new_target.agent_id`), then call `dal.agent_generator_registrations().is_registered(agent_id, stack.generator_id)`. If `false`, return `ApiError::forbidden("agent_not_registered", "agent must be registered with this generator before stacks can be targeted at it")`. No change to the function signature.

This is the enforcement checkpoint for the entire initiative. Everything else is infrastructure; this is the gate.

### Dependencies
Blocked by T-0241 (`is_registered` DAL method) and T-0242 (system generator auto-registration must be in place so the check doesn't break all existing stacks).

### Risk Considerations
**This task is the breaking change.** Once merged, any agent not registered with a generator cannot be targeted — including by admin. The back-fill in T-0248 must run before or alongside this task in production. Do not merge T-0246 before T-0248 is ready for the environment being upgraded.

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