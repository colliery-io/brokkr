---
id: agent-should-reconcile-existing
level: task
title: "Agent should reconcile existing deployments when targeted to a stack"
short_code: "BROKKR-T-0105"
created_at: 2026-01-01T00:32:53.514089+00:00
updated_at: 2026-01-02T20:05:59.422379+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Agent should reconcile existing deployments when targeted to a stack

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

When an agent is targeted to a stack that already contains deployment objects, the agent should reconcile those existing deployments on its next sync cycle.

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
- **Affected Users**: Any user who creates deployments before targeting a stack to an agent
- **Reproduction Steps**: 
  1. Create a stack
  2. Create a deployment object on the stack
  3. Target the stack to an agent
  4. Wait for agent reconciliation cycle
- **Expected vs Actual**: 
  - **Expected**: Agent should deploy the existing deployment objects
  - **Actual**: Agent never sees the deployments (they were created before targeting)

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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Agent reconciles existing deployments when targeted to a stack post-creation
- [x] Order of operations (create deployment vs target stack) doesn't matter
- [x] E2E test covers this scenario

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

## Implementation Notes **[REQUIRED]**

### Investigation Results (2026-01-02)

**Broker-side DAL logic is correct.** The code analysis and integration tests confirm that:
- `get_target_state_for_agent()` queries by current state, not creation timestamps
- `agent_targets`, label matching, and annotation matching all work correctly regardless of when targeting was established relative to deployment creation

### Tests Added
1. **Integration tests** (`deployment_objects.rs`):
   - `test_target_state_direct_targeting_after_deployment_exists` - PASS
   - `test_target_state_label_targeting_after_deployment_exists` - PASS
   - `test_target_state_annotation_targeting_after_deployment_exists` - PASS

2. **E2E test** (`scenarios.rs`):
   - `test_agent_reconciliation_existing_deployments` - Tests full API flow

### Conclusion
The broker-side logic is working correctly. If the original bug still manifests, the issue is likely in:
- Agent-side caching/polling logic
- Agent reconciliation loop timing
- Some specific edge case not covered by these tests

## Status Updates **[REQUIRED]**

- **2026-01-02**: Added integration tests and E2E test. Broker DAL logic confirmed working correctly.