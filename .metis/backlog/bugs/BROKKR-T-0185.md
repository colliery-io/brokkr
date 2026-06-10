---
id: fix-workspace-rust-version-typo-1
level: task
title: "Fix workspace rust-version typo (1.8 → 1.85)"
short_code: "BROKKR-T-0185"
created_at: 2026-06-10T03:03:52.637512+00:00
updated_at: 2026-06-10T04:57:51.127698+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Fix workspace rust-version typo (1.8 → 1.85)

## Objective

Correct the workspace MSRV declaration: `Cargo.toml:5` reads `rust-version = "1.8"`, but the workspace uses `edition = "2024"`, which requires rustc ≥ 1.85. `"1.8"` (i.e. 1.8.0) is meaningless for MSRV tooling and misleads contributors (the README and docs previously echoed "Rust 1.8+").

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Cargo.toml` `rust-version` set to `"1.85"` (or the actual verified MSRV)
- [ ] `cargo check` passes across the workspace
- [ ] Remaining doc references to "Rust 1.8" updated (README.md; docs already say 1.85+)

## Status Updates

- 2026-06-09: Found during /docs-diataxis accuracy review (BROKKR-I-0015 follow-up).
- 2026-06-09: IMPLEMENTED (uncommitted, unit tests green): `rust-version = "1.85"` in Cargo.toml; docs already updated.- 2026-06-10 (proper resolution): the original premise was WRONG — the crates were edition 2021, and `edition`/`rust-version` under `[workspace]` were dead keys (Cargo "unused manifest key"), so the 1.8→1.85 change had no effect. Fixed properly per user direction (option 3): moved both into `[workspace.package]` with edition 2024 + rust-version 1.90, members inherit via `.workspace = true`. cargo fix --edition applied (unsafe env::set_var, let-chains, 2024 fmt). Build + all units green; OpenAPI spec unchanged. Commit e53e8d8.
