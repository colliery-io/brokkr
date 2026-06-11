---
id: crates-workspace-inherited-license
level: task
title: "Crates: workspace-inherited license/repository metadata + descriptions"
short_code: "BROKKR-T-0219"
created_at: 2026-06-11T11:02:08.479488+00:00
updated_at: 2026-06-11T19:46:02.656776+00:00
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

# Crates: workspace-inherited license/repository metadata + descriptions

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

`brokkr-agent`, `brokkr-broker`, `brokkr-models`, `brokkr-utils` have no `license`, `description`, or `repository` in Cargo.toml (brokkr-cli, brokkr-client, brokkr-wire have them). Add via workspace inheritance so future crates get them for free.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Root `Cargo.toml` `[workspace.package]` gains `license = "Elastic-2.0"`, `repository`, `homepage`.
- [ ] All seven crates inherit (`license.workspace = true`, etc.); existing per-crate values reconciled (brokkr-cli/client/wire already say Elastic-2.0).
- [ ] Each crate gets a one-line `description`.
- [ ] `cargo metadata --locked` clean; workspace builds.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE (branch feat/i0026-docs-ci-hygiene). Added `license = "Elastic-2.0"`, `repository`, `homepage` to root `[workspace.package]`. brokkr-agent/broker/models/utils (which had no license/description/repository) now inherit all three via `.workspace = true` and each got a one-line `description`. The 3 crates that already had literal metadata (brokkr-client/cli/wire) converted to `.workspace = true` (same values) so all 7 inherit consistently; brokkr-wire gained the missing homepage. Verified: `cargo metadata --no-deps --locked` resolves all 7 crates (manifests valid). Future crates get the metadata for free.
