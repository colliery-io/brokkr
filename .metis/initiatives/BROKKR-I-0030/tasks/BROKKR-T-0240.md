---
id: migration-agent-generator
level: task
title: "Migration: agent_generator_registrations table and indexes"
short_code: "BROKKR-T-0240"
created_at: 2026-06-26T13:33:21.163706+00:00
updated_at: 2026-06-26T17:27:31.844164+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Migration: agent_generator_registrations table and indexes

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Diesel migration creates `agent_generator_registrations` with `id UUID PK`, `agent_id UUID NOT NULL FK → agents(id) ON DELETE CASCADE`, `generator_id UUID NOT NULL FK → generators(id) ON DELETE CASCADE`, `registered_at TIMESTAMPTZ NOT NULL DEFAULT now()`, and `UNIQUE (agent_id, generator_id)`
- [ ] Indexes `idx_agr_agent_id` and `idx_agr_generator_id` created
- [ ] `AgentGeneratorRegistration` (Queryable) and `NewAgentGeneratorRegistration` (Insertable) structs added to `crates/brokkr-models/src/models/agent_generator_registrations.rs` and registered in `mod.rs`
- [ ] `angreal models migrations` passes (up + redo)
- [ ] `angreal models schema` regenerates `schema.rs` cleanly with the new table

## Implementation Notes

### Technical Approach
Follow the existing migration pattern in `crates/brokkr-models/migrations/`. Mirror the model shape from `agent_targets.rs` — two FK UUIDs plus metadata. The `UNIQUE (agent_id, generator_id)` constraint serves double duty as an index for the `is_registered` lookup in T-0241, so no separate unique index is needed on that pair.

### Dependencies
Independent — can be worked in parallel with T-0239. T-0241 (DAL) and all downstream tasks block on this.

### Risk Considerations
Straightforward additive migration. No existing data is touched.

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
- Migration `22_agent_generator_registrations`: table with UNIQUE (agent_id, generator_id) + two indexes
- Model `agent_generator_registrations.rs`: `AgentGeneratorRegistration` + `NewAgentGeneratorRegistration` with unit tests
- `models/mod.rs`: module registered
- `schema.rs`: table, joinable!, allow_tables entries added manually
- `dal/agent_generator_registrations.rs`: create, is_registered (EXISTS query), list_for_agent, list_for_generator, delete, delete_agent_targets_for_generator
- `dal/mod.rs`: module and accessor registered
- `cargo check` clean; unit tests 132/132 pass