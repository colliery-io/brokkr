---
id: ci-add-brokkr-wire-brokkr-cli
level: task
title: "CI: add brokkr-wire, brokkr-cli, brokkr-client to the unit-test matrix"
short_code: "BROKKR-T-0217"
created_at: 2026-06-11T11:02:08.378231+00:00
updated_at: 2026-06-11T11:02:08.378231+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# CI: add brokkr-wire, brokkr-cli, brokkr-client to the unit-test matrix

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

`unit_tests.yml:9` matrix is `[brokkr-agent, brokkr-broker, brokkr-models, brokkr-utils]`, but `.angreal/task_tests.py:14` also lists `brokkr-wire` as a unit-test crate, and `brokkr-cli` (16 tests) and `brokkr-client` (14+ tests) have suites that run nowhere in CI — PR, nightly, or release gates.

## Acceptance Criteria

- [ ] brokkr-wire, brokkr-cli, brokkr-client unit tests run on every PR/main/release (extend the matrix or switch the job to the angreal task so the list lives in one place).
- [ ] `.angreal/task_tests.py` crate list and the CI matrix agree (single source of truth preferred).
- [ ] All suites green on CI.

## Status Updates

*To be added during implementation*
