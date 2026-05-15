---
id: r2-a-vendor-spec-brokkr-client
level: task
title: "R2-A: Vendor OpenAPI spec into brokkr-client crate"
short_code: "BROKKR-T-0145"
created_at: 2026-05-15T22:30:00.000000+00:00
updated_at: 2026-05-15T22:30:00.000000+00:00
parent: BROKKR-I-0018
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-A: Vendor OpenAPI spec into brokkr-client crate

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Make `brokkr-client` `cargo publish`-able by ending the macro's reliance on a path outside the crate directory. Today `crates/brokkr-client/src/lib.rs` does `progenitor::generate_api!(spec = "../../openapi/brokkr-v1.json", ...)`; the relative path doesn't survive crate packaging.

## Acceptance Criteria

- [ ] `openapi/brokkr-v1.json` is mirrored into `crates/brokkr-client/spec/brokkr-v1.json` (committed; not gitignored).
- [ ] `crates/brokkr-client/src/lib.rs` macro path is `spec = "spec/brokkr-v1.json"` (no `../../`).
- [ ] `angreal openapi export` writes both copies and they stay byte-identical.
- [ ] `angreal openapi check` fails loudly if the two copies drift.
- [ ] `cargo package -p brokkr-client` produces a tarball that includes `spec/brokkr-v1.json` and compiles in isolation (verify with `cargo package --list` and a smoke-test against the unpacked tarball).
- [ ] CI's existing `drift_and_lint` job covers the new parity check (or a sibling job does).
- [ ] In-tree workspace build (`cargo build -p brokkr-client`) still works unchanged.

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

*To be added during implementation*
