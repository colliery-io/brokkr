---
id: angreal-task-exit-status
level: task
title: "Angreal Task Exit Status"
short_code: "BROKKR-T-0027"
created_at: 2025-10-23T01:46:25.523842+00:00
updated_at: 2025-10-23T01:46:25.523842+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


  - "#bug"
exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Angreal Task Exit Status

*Currently, even when things fail in the angreal test runner - they always exit code 0. This is normal, we need to update angreal tasks to force a sys.exit(1) on failure paths to ensure that things like CI/CD pipelines*\
*appropriately*

## Objective **\[REQUIRED\]**

Fix angreal exit status codes to ensure that CI/CD pipelines appropriately “go red” on failure.

##

### Type

- \[ x \] Bug - Production issue that needs fixing
- \[ \] Feature - New functionality or enhancement
- \[ \] Tech Debt - Code improvement or refactoring
- \[ \] Chore - Maintenance or setup work

### Priority

- \[ \] P0 - Critical (blocks users/revenue)
- \[ x \] P1 - High (important for user experience)
- \[ \] P2 - Medium (nice to have)
- \[ \] P3 - Low (when time permits)

### Impact Assessment **\[CONDITIONAL: Bug\]**

- **Affected Users**: All users running angreal tasks in CI/CD pipelines
- **Reproduction Steps**:
  1. Run any angreal task that encounters a failure (e.g., `angreal models test` with failing tests)
  2. Check the exit code with `echo $?`
  3. Observe that exit code is 0 even though the task failed
- **Expected vs Actual**:
  - Expected: Exit code should be non-zero (typically 1) on failure
  - Actual: Exit code is 0 even when tasks fail, causing CI/CD pipelines to incorrectly pass

### Business Justification **\[CONDITIONAL: Feature\]**

- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **\[CONDITIONAL: Tech Debt\]**

- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria **\[REQUIRED\]**

- [ ] All angreal tasks return non-zero exit codes on failure
- [ ] task_models.py: `schema()` returns subprocess exit code
- [ ] task_models.py: `test()` returns proper exit code on failure
- [ ] task_local.py: `docs()` returns subprocess exit code
- [ ] task_local.py: `rebuild()` returns subprocess exit code
- [ ] task_tests.py: `unit_tests()` always returns a valid exit code (never None)
- [ ] task_tests.py: `integration_tests()` properly returns exit code from finally block
- [ ] CI/CD pipelines correctly fail when angreal tasks encounter errors



## Implementation Notes **\[CONDITIONAL: Technical Task\]**

### Technical Approach

Review each angreal task file and ensure all functions properly propagate exit codes. The pattern should be:

1. For subprocess calls: Capture and return the `.returncode`
2. For functions that can fail: Return appropriate non-zero exit code
3. For try/finally blocks: Ensure return statements are in finally or properly propagated
4. Never return `None` - always return an explicit exit code (0 for success, non-zero for failure)

### Files Requiring Updates

#### 1. `.angreal/task_models.py`

**Issues Found:**
- `schema()` (lines 22-30): Runs two subprocess calls but doesn't capture or return exit codes
- `test()` (lines 221-281): No return statement - function returns None implicitly

**Required Changes:**
- `schema()`: Add `return` statements or use `check=True` and handle exceptions with `sys.exit(1)`
- `test()`: Add proper exit code handling for subprocess failures

#### 2. `.angreal/task_local.py`

**Issues Found:**
- `docs()` (lines 47-54): Runs subprocess but doesn't return exit code
- `rebuild()` (lines 58-76): Runs subprocess but doesn't return exit code, also has early return on validation error without exit code

**Required Changes:**
- Both functions should capture subprocess result and return `.returncode`
- `rebuild()`: Return non-zero exit code when service validation fails

#### 3. `.angreal/task_tests.py`

**Issues Found:**
- `unit_tests()` (lines 48-62): Returns `None` when `crate_name != "all"` and no failures occur
- `integration_tests()` (lines 70-97): Return statement is inside `finally` block but `rc` might be `None`

**Required Changes:**
- `unit_tests()`: Ensure `rc` is always set before return (initialize to 0 or ensure all code paths set it)
- `integration_tests()`: Ensure `rc` is always initialized and returned properly

#### 4. `.angreal/task_docs.py`

**Already Correct:** Functions properly return subprocess return codes or use early returns with error codes

#### 5. `.angreal/task_build.py`

**Already Correct:** Properly returns exit codes throughout

#### 6. `.angreal/task_helm.py`

**Already Correct:** Uses `sys.exit()` for failures, which is appropriate for this use case

### Dependencies

None - this is a standalone bug fix

### Risk Considerations

**Low Risk:**
- Changes are isolated to return statements
- Improves reliability of CI/CD pipelines
- May cause currently "passing" but broken pipelines to fail (this is desired behavior)

**Testing Strategy:**
- Test each angreal command manually
- Verify exit codes with `echo $?` after running commands
- Intentionally trigger failures to verify non-zero exit codes

## Status Updates **\[REQUIRED\]**

*To be added during implementation*
