---
id: back-fill-migration-and
level: task
title: "Back-fill migration and integration test suite for generator registration"
short_code: "BROKKR-T-0248"
created_at: 2026-06-26T13:33:46.122053+00:00
updated_at: 2026-06-26T18:07:47.627639+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Back-fill migration and integration test suite for generator registration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Back-fill migration registers all pre-existing agents with the system generator — idempotent (`ON CONFLICT DO NOTHING`)
- [ ] Back-fill migration registers all pre-existing agents with each application generator they were already targeting (inferred from `agent_targets JOIN stacks ON stacks.id = agent_targets.stack_id` — if an agent has targets against a generator's stacks, register it with that generator)
- [ ] Back-fill migration is safe to run on a live database and produces no downtime
- [ ] Integration test: `POST /agents` → agent automatically has system generator registration
- [ ] Integration test: unregistered agent + `POST /agents/:id/targets` → `403 agent_not_registered`
- [ ] Integration test: registered agent + `POST /agents/:id/targets` → `201`
- [ ] Integration test: admin targets unregistered agent → `403` (admin cannot bypass)
- [ ] Integration test: two generators, two agents — Agent A registered with Gen A only; targeting Gen B's stack at Agent A → `403`; targeting Gen A's stack at Agent A → `201`
- [ ] Integration test: `DELETE /generators/:id/register` → `agent_targets` cascade verified (target rows gone)
- [ ] All pre-existing targeting integration tests in `tests/integration/api/agents.rs` pass without modification

## Implementation Notes

### Technical Approach
Back-fill as a Diesel migration (runs automatically on deploy). Core SQL:

```sql
-- 1. Register all existing agents with the system generator
INSERT INTO agent_generator_registrations (id, agent_id, generator_id)
SELECT gen_random_uuid(), a.id, (SELECT id FROM generators WHERE is_system = true)
FROM agents a
ON CONFLICT (agent_id, generator_id) DO NOTHING;

-- 2. Register agents with generators they are already targeting
INSERT INTO agent_generator_registrations (id, agent_id, generator_id)
SELECT DISTINCT gen_random_uuid(), at.agent_id, s.generator_id
FROM agent_targets at
JOIN stacks s ON s.id = at.stack_id
ON CONFLICT (agent_id, generator_id) DO NOTHING;
```

Integration tests live in `crates/brokkr-broker/tests/integration/api/` — add a new file `generator_registration.rs` following the pattern of `agents.rs` and `stacks.rs`.

### Dependencies
Blocked by all preceding tasks (T-0239 through T-0246). T-0247 (deregistration reconciliation) should be complete or in progress before this task is considered done, since the deregister cascade is part of the test suite.

### Risk Considerations
The back-fill is a standard Diesel migration — it runs automatically at broker startup before the HTTP server accepts requests, so no manual upgrade step is needed. The natural execution order is: migrations run → back-fill completes → enforcement code (T-0246) becomes active. As long as the back-fill migration file has an earlier timestamp than T-0248 itself (which it will, since T-0246 is a pure code change with no migration), the ordering is guaranteed by Diesel.

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

2026-06-26: Back-fill migration written as `23_backfill_generator_registrations`.
- Registers agents with generators they already target via agent_targets JOIN stacks — preserves current operational state
- System-generator registrations excluded from migration (system generator not yet created at migration time); handled by provision_system_generator() at startup which registers all existing agents when it first creates the system generator
- ON CONFLICT DO NOTHING — fully idempotent
- Integration tests pending (require T-0241 through T-0246 API/enforcement work to be complete)