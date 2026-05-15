---
id: r2-a-vendor-openapi-spec-into
level: task
title: "R2-A: Vendor OpenAPI spec into brokkr-client crate"
short_code: "BROKKR-T-0145"
created_at: 2026-05-15T22:30:00+00:00
updated_at: 2026-05-15T23:18:10.987859+00:00
parent: BROKKR-I-0018
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-A: Vendor OpenAPI spec into brokkr-client crate

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Make `brokkr-client` `cargo publish`-able by ending the macro's reliance on a path outside the crate directory. Today `crates/brokkr-client/src/lib.rs` does `progenitor::generate_api!(spec = "../../openapi/brokkr-v1.json", ...)`; the relative path doesn't survive crate packaging.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `openapi/brokkr-v1.json` is mirrored into `crates/brokkr-client/spec/brokkr-v1.json` (committed; not gitignored).
- [x] `crates/brokkr-client/src/lib.rs` macro path is `spec = "spec/brokkr-v1.json"` (no `../../`).
- [x] `angreal openapi export` writes both copies and they stay byte-identical.
- [x] `angreal openapi check` fails loudly if the two copies drift.
- [x] `cargo package -p brokkr-client` produces a tarball that includes `spec/brokkr-v1.json` and compiles in isolation (`cargo package` runs the unpack-and-verify build; passed).
- [x] CI's existing `drift_and_lint` job covers the new parity check (already invokes `angreal openapi check`).
- [x] In-tree workspace build (`cargo build -p brokkr-client`) still works unchanged.

## Implementation Notes

### Technical Approach

1. Add `spec/` dir under `crates/brokkr-client/` and copy current `openapi/brokkr-v1.json` content into it.
2. Update macro path. The two file copies coexist by design — `openapi/` remains the canonical broker-facing artifact (used by Python/TS generators and CI drift checks), `crates/brokkr-client/spec/` is the cargo-shipping copy.
3. Update the `.angreal/` task that drives `openapi export` to write both files. Cheapest: after writing `openapi/brokkr-v1.json`, `cp` it to `crates/brokkr-client/spec/brokkr-v1.json`.
4. Update `openapi check` to assert parity. A `cmp` or hash equality is enough.
5. Smoke-test the published shape: `cargo package -p brokkr-client && cd target/package/brokkr-client-* && cargo build`. This is the canonical "would this work for a downstream consumer" check.

### Dependencies

- None (can run in parallel with R2-B).

### Risk Considerations

- The duplicate spec file shows up in git diffs every time the API changes. The drift check is what keeps it honest; without it, the published crate will silently stale-bake.
- The `crates/brokkr-client/Cargo.toml` may already have an `include` / `exclude` field that affects what `cargo package` ships. Verify the spec directory is captured (default behavior is "include everything tracked by git," so this should Just Work, but check).

## Status Updates

- 2026-05-15: Implemented.
  - Mirrored `openapi/brokkr-v1.json` → `crates/brokkr-client/spec/brokkr-v1.json`.
  - Updated `crates/brokkr-client/src/lib.rs` macro path from `../../openapi/brokkr-v1.json` to `spec/brokkr-v1.json`.
  - Extended `.angreal/task_openapi.py`: `export` now mirrors to the crate-local copy via `_mirror_crate_spec()`; `check` asserts both copies exist and are byte-identical before running its temp-export drift comparison.
  - Verified: `cargo build -p brokkr-client` clean; `angreal openapi check` passes; deliberate drift triggers the new FAIL path; `cargo package -p brokkr-client --list` includes `spec/brokkr-v1.json`; `cargo package`'s built-in verification compile of the unpacked tarball succeeds.
  - No `include`/`exclude` rules in `crates/brokkr-client/Cargo.toml`; spec file is not gitignored. No CI workflow changes needed — existing `drift_and_lint` job runs `angreal openapi check`.