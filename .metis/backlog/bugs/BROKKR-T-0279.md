---
id: readme-quick-start-is-broken-stack
level: task
title: "README quick start is broken: stack example omits required generator_id; ui-slim mislabeled as 'the admin UI'"
short_code: "BROKKR-T-0279"
created_at: 2026-07-27T14:19:50.709845+00:00
updated_at: 2026-07-27T14:19:50.709845+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# README quick start is broken: stack example omits required generator_id; ui-slim mislabeled as 'the admin UI'

## Objective

Fix the repo front door. Two defects found in the 2026-07-27 auth-drift doc sweep (code-verified):

1. **Quick-start curl fails as written (blocker).** The "Creating Your First Deployment" stack-creation example omits `generator_id`, which is non-optional (`crates/brokkr-models/src/models/stacks.rs:77-83`, `NewStack.generator_id: Uuid`); the request 422s. `installation.md` and `evaluate.md` already show the correct admin-generator lookup pattern — the README predates it.
2. **ui-slim conflated with the product console (minor).** "Running Locally" calls the :3001 demo "the admin UI"; the supported Operator Console is now served by the broker itself at :3000 (`crates/brokkr-broker/src/api/assets.rs`), while `examples/ui-slim` is explicitly "a demonstration ... not a supported product" (`evaluate.md:128`).

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (first-contact experience; the very first command a prospective consumer copies fails)

### Impact Assessment
- **Affected Users**: Everyone evaluating Brokkr from the README.
- **Reproduction Steps**: Follow README Quick Start → POST the stack-creation example → 422 unprocessable entity.
- **Expected vs Actual**: Expected a created stack; actual validation failure on missing `generator_id`.

## Acceptance Criteria

- [ ] README stack example includes the admin-generator lookup and `"generator_id"` field and works verbatim against a fresh install.
- [ ] README distinguishes the broker-served Operator Console (:3000) from the ui-slim demo (:3001, unsupported).

## Status Updates

*To be added during implementation*
