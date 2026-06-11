---
id: ci-add-brokkr-wire-brokkr-cli
level: task
title: "CI: add brokkr-wire, brokkr-cli, brokkr-client to the unit-test matrix"
short_code: "BROKKR-T-0217"
created_at: 2026-06-11T11:02:08.378231+00:00
updated_at: 2026-06-11T19:48:29.492021+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# CI: add brokkr-wire, brokkr-cli, brokkr-client to the unit-test matrix

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

`unit_tests.yml:9` matrix is `[brokkr-agent, brokkr-broker, brokkr-models, brokkr-utils]`, but `.angreal/task_tests.py:14` also lists `brokkr-wire` as a unit-test crate, and `brokkr-cli` (16 tests) and `brokkr-client` (14+ tests) have suites that run nowhere in CI — PR, nightly, or release gates.

## Acceptance Criteria

## Acceptance Criteria

- [ ] brokkr-wire, brokkr-cli, brokkr-client unit tests run on every PR/main/release (extend the matrix or switch the job to the angreal task so the list lives in one place).
- [ ] `.angreal/task_tests.py` crate list and the CI matrix agree (single source of truth preferred).
- [ ] All suites green on CI.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0026-docs-ci-hygiene). The unit_tests.yml matrix `[agent, broker, models, utils]` omitted brokkr-wire/client/cli, and diverged from `.angreal/task_tests.py` CRATES["unit_tests"] (which had wire but not client/cli). Made the angreal list the single source of truth and switched the job from a 4-way matrix (which recompiled the workspace 4× in parallel) to one job running `angreal tests unit all` — now covering agent/broker/models/utils/wire/**client** via `cargo test --lib`. brokkr-cli is bin-only (no lib target — `cargo test --lib` errors), so it runs in a dedicated `cargo test -p brokkr-cli` step (its config unit tests + the tests/cli.rs black-box tests, which need no broker). Verified locally: client --lib 14 pass, cli 16 pass (9+7), wire 0 (type-only). YAML valid. One compile instead of four, and every testable crate now runs in CI.
