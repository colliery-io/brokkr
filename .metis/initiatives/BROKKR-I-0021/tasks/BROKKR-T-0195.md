---
id: rust-sdk-submit-manifests-apply
level: task
title: "Rust SDK: submit_manifests/apply folder helper"
short_code: "BROKKR-T-0195"
created_at: 2026-06-11T02:19:30.770706+00:00
updated_at: 2026-06-11T03:26:24.386307+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0021
---

# Rust SDK: submit_manifests/apply folder helper

## Parent Initiative

[[BROKKR-I-0021]]

## Objective

Add wrapper-layer helpers to the Rust SDK (`crates/brokkr-client`) so a folder of manifests can be submitted as a stack's desired state in one idempotent call. No OpenAPI/codegen change — these live in `src/wrapper.rs` alongside `BrokkrClient`.

## Design

- `submit_manifests(stack_id, paths_or_dir)` — accept a dir, file, or globs; walk for `*.yaml`/`*.yml`; concatenate documents with `---`; validate each parses and carries `apiVersion`+`kind`; POST the stream (raw-YAML endpoint from T-0194 when available, else the JSON envelope).
- `apply(stack_name, dir, targeting)` — idempotent: resolve stack by name; create if missing (owner = the client's generator identity); compute the bundle checksum and submit a new deployment object only when it differs from the stack's latest; set targeting labels. Return an enum/struct indicating Created / Updated / Unchanged.
- Ordering is forgiving (agent front-loads Namespace/CRD); document that deleting a file and re-applying prunes the removed object.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `submit_manifests` and `apply` on `BrokkrClient`, reading a directory of YAML files
- [ ] `apply` is idempotent — re-running with an unchanged folder reports Unchanged and creates no new revision
- [ ] Per-doc validation (apiVersion+kind) with clear errors
- [ ] Unit/integration tests (folder fixture; idempotency; prune-on-delete behavior)
- [ ] Rust SDK how-to updated with the folder workflow

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Feeds the CLI (T-0198).
- 2026-06-11: IMPLEMENTED (branch feat/i0021-raw-yaml-submission). Added `submit_manifests(stack_id, path)`, idempotent `apply(stack_name, path, targeting)` returning `ApplyOutcome::{Created,Updated,Unchanged}`, and pure helpers `read_manifests` (folder walk + concatenate `---` + per-doc apiVersion/kind validation) and `sha256_hex` (matches broker checksum for idempotency) in `crates/brokkr-client/src/wrapper.rs`. `apply` resolves the generator via verify_pak, find-or-creates the stack by name, applies targeting labels (ignoring 409s), and submits only when the bundle checksum differs from the latest. Added serde_yaml/sha2 deps; ApplyOutcome re-exported. Unit tests: 6 (sorted folder concat, single-file/multidoc, missing apiVersion/kind reject, malformed reject, empty-dir/missing-path errors, sha256 known-vector). Functional: live scenario in tests/sdk-contract/rust (Created→Unchanged→Updated + label + submit_manifests) — runs on CI. Docs: rust SDK how-to. Remaining: contract test runs on CI.
