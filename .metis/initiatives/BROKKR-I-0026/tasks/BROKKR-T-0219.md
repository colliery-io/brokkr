---
id: crates-workspace-inherited-license
level: task
title: "Crates: workspace-inherited license/repository metadata + descriptions"
short_code: "BROKKR-T-0219"
created_at: 2026-06-11T11:02:08.479488+00:00
updated_at: 2026-06-11T11:02:08.479488+00:00
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

# Crates: workspace-inherited license/repository metadata + descriptions

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

`brokkr-agent`, `brokkr-broker`, `brokkr-models`, `brokkr-utils` have no `license`, `description`, or `repository` in Cargo.toml (brokkr-cli, brokkr-client, brokkr-wire have them). Add via workspace inheritance so future crates get them for free.

## Acceptance Criteria

- [ ] Root `Cargo.toml` `[workspace.package]` gains `license = "Elastic-2.0"`, `repository`, `homepage`.
- [ ] All seven crates inherit (`license.workspace = true`, etc.); existing per-crate values reconciled (brokkr-cli/client/wire already say Elastic-2.0).
- [ ] Each crate gets a one-line `description`.
- [ ] `cargo metadata --locked` clean; workspace builds.

## Status Updates

*To be added during implementation*
