---
id: agent-broker-deregistration
level: task
title: "Agent/Broker: deregistration reconciliation to empty cluster state"
short_code: "BROKKR-T-0247"
created_at: 2026-06-26T13:33:43.943683+00:00
updated_at: 2026-06-26T18:00:07.591680+00:00
parent: BROKKR-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0030
---

# Agent/Broker: deregistration reconciliation to empty cluster state

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0030]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Design decision recorded at task start: which reconciliation mechanism is used and why
- [ ] Deregistration of an agent from a generator leaves no orphaned cluster resources for that agent
- [ ] Other agents targeting the same stacks are completely unaffected
- [ ] The departing agent's cluster state converges to empty for those stacks within a bounded time
- [ ] Mechanism does not require work orders (ruled out — they are imperative one-shots, not state reconciliation)

## Implementation Notes

### Technical Approach

Begin this task with a spike: read `crates/brokkr-agent/src/work_orders/` and the agent's reconciliation loop to understand whether "stack disappeared from my target list" already triggers cleanup, or whether that path needs to be added.

**Three mechanisms to evaluate — pick one:**

1. **Reconciliation loop gap detection (preferred starting point).** The `DELETE /generators/:id/register` cascade (T-0244) already removes the agent's `agent_targets` rows for that generator's stacks. The agent's next reconciliation tick fetches its target state and finds those stacks gone. If the agent already applies deletion logic for resources it applied from stacks that are no longer in its target list, this is zero additional broker work. Verify this path exists in the agent codebase; add it if not.

2. **WS push on deregistration.** After the cascade in T-0244, broker calls `push_target_changed` (or equivalent) to the departing agent via WebSocket. The agent reacts immediately rather than waiting for its next polling interval. Adds latency predictability but requires the agent to be connected — graceful degradation if it is not.

3. **Scoped deletion record.** New table `agent_stack_removals (agent_id, stack_id, created_at)`. On deregistration, broker inserts one row per removed stack. Agent checks this on its next reconciliation tick, applies deletion for those stacks, then deletes the rows. Most explicit lifecycle but most new surface area.

Work orders are ruled out — they are imperative one-shot commands, not a desired-state signal.

### Dependencies
Blocked by T-0244 (deregistration endpoint and cascade must exist). Requires reading the agent reconciliation loop in `crates/brokkr-agent/src/work_orders/` before the mechanism can be chosen.

### Risk Considerations
Highest-uncertainty task in the initiative. The risk of getting this wrong is orphaned Kubernetes resources in a cluster. Plan for a spike before committing to an implementation. Validate end-to-end in the local environment (`angreal local up`) before closing the task.

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