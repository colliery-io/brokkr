---
id: broker-auto-register-agents-with
level: task
title: "Broker: auto-register agents with system generator; accept generator_ids on POST /agents"
short_code: "BROKKR-T-0242"
created_at: 2026-06-26T13:33:32.721997+00:00
updated_at: 2026-06-26T17:40:59.936670+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Broker: auto-register agents with system generator; accept generator_ids on POST /agents

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every agent created via `POST /agents` is automatically registered with the system generator — no extra call required by the caller
- [ ] `NewAgent` / `CreateAgentRequest` gains an optional `generator_ids: Vec<Uuid>` field (defaults to empty)
- [ ] Any `generator_ids` supplied are validated (generator must exist) and registered alongside the system generator registration
- [ ] Unknown generator UUIDs in `generator_ids` return `400 invalid_generator_id`; no partial registration is created
- [ ] Audit log entries emitted for each registration created at agent creation time
- [ ] Existing `POST /agents` callers with no `generator_ids` field are unaffected (field is optional, backward compatible)

## Implementation Notes

### Technical Approach
Edit `create_agent` in `crates/brokkr-broker/src/api/v1/agents.rs`. After the agent row is committed, resolve the system generator ID (available from startup state — store it in `DAL` or pass via app state), and call `dal.agent_generator_registrations().create(agent.id, system_id)`. For any `generator_ids` in the request body, validate each exists via `dal.generators().get(id)` — fail with 400 if any are missing — then insert registrations. Consider wrapping the agent insert + all registration inserts in a single transaction so no partial state is committed.

### Dependencies
Blocked by T-0239 (system generator ID) and T-0241 (registration DAL).

### Risk Considerations
Transaction boundary is important: if registration fails after the agent row is committed, the agent exists but has no system generator registration. Wrap in a transaction or use a compensating delete.

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