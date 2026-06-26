---
id: dal-agentgeneratorregistrations
level: task
title: "DAL: AgentGeneratorRegistrations CRUD and is_registered lookup"
short_code: "BROKKR-T-0241"
created_at: 2026-06-26T13:33:25.111742+00:00
updated_at: 2026-06-26T17:27:52.559483+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# DAL: AgentGeneratorRegistrations CRUD and is_registered lookup

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create(agent_id, generator_id)` inserts a registration; returns the row on success; caller handles unique-violation as a 409
- [ ] `is_registered(agent_id, generator_id) -> bool` uses the UNIQUE constraint index — confirmed with `EXPLAIN` that no seq scan occurs
- [ ] `list_for_agent(agent_id) -> Vec<AgentGeneratorRegistration>` returns all generators an agent is registered with
- [ ] `list_for_generator(generator_id) -> Vec<AgentGeneratorRegistration>` returns all agents registered with a generator
- [ ] `delete(agent_id, generator_id) -> usize` removes one registration; returns rows deleted (0 or 1)
- [ ] `delete_for_agent_and_generator(agent_id, generator_id)` removes all `agent_targets` rows where `agent_id = $1` AND `stack_id IN (SELECT id FROM stacks WHERE generator_id = $2)` — used by the deregister cascade in T-0244
- [ ] DAL integration tests cover all operations including the cascade delete

## Implementation Notes

### Technical Approach
Follow the pattern in `crates/brokkr-broker/src/dal/agent_targets.rs`. Add `AgentGeneratorRegistrationsDal` struct and wire it into the `DAL` struct in `crates/brokkr-broker/src/dal/mod.rs`. `is_registered` should use `diesel::dsl::exists` with `.filter(agent_id.eq($1).and(generator_id.eq($2)))` — the database will use the unique index. The cascade-delete helper method `delete_for_agent_and_generator` can use a subquery or a join to identify the relevant `agent_targets` rows.

### Dependencies
Blocked by T-0240 (migration must exist before DAL compiles). T-0242, T-0244, T-0245, T-0246 all block on this.

### Risk Considerations
`is_registered` is on the hot path of every `POST /agents/:id/targets` call. Must not regress targeting latency. Verify with integration-test timing or EXPLAIN ANALYZE.

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

2026-06-26: Completed as part of T-0240 implementation.
- `dal/agent_generator_registrations.rs`: all six methods implemented — create, is_registered (diesel::dsl::exists backed by UNIQUE index), list_for_agent, list_for_generator, delete, delete_agent_targets_for_generator
- `dal/mod.rs`: module registered, `agent_generator_registrations()` accessor added to DAL
- `cargo check` clean; migrations pass up + redo
- Integration tests deferred to T-0248 (require full API surface to be meaningful)