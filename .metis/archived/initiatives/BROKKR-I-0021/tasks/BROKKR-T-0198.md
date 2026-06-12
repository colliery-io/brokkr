---
id: brokkr-cli-apply-f-folder-of
level: task
title: "brokkr CLI: apply -f folder of manifests"
short_code: "BROKKR-T-0198"
created_at: 2026-06-11T02:19:34.293955+00:00
updated_at: 2026-06-11T05:47:28.389491+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0021
---

# brokkr CLI: apply -f folder of manifests

## Parent Initiative

[[BROKKR-I-0021]]

## Objective

A new user-facing `brokkr` CLI (distinct from the broker/agent admin binaries) that makes submitting a folder feel like `kubectl apply -f ./dir`, backed by the Rust SDK `apply` helper (T-0195).

## Design

- New crate (e.g. `crates/brokkr-cli`) over `brokkr-client`; binary `brokkr`.
- v1 verb: `brokkr apply -f <dir|file> --stack <name> [--target-label k=v]...` — read the folder via the SDK `apply`, idempotent create-or-reuse stack, submit-on-change, set targeting labels; pruning is automatic via the engine. Print created/updated/unchanged.
- Config: `~/.brokkr/config` (kubeconfig-shaped — broker URL + PAK), overridable by `--broker-url`/`--pak` flags and `BROKKR_*` env.
- Out of v1 (follow-ons): `brokkr diff`, `brokkr get -o yaml`, `brokkr stack {create,delete,prune}`, shell completion.
- Decide packaging/release: the CLI is a user artifact — fold into the lockstep release (a `brokkr` binary alongside broker/agent images, or a published crate/brew/install script). Capture in the release workflow.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `brokkr apply -f ./manifests/ --stack test` creates/updates the stack and submits the bundle; re-run with no changes reports unchanged
- [ ] `--target-label k=v` sets stack labels (fan-out targeting) in the same call
- [ ] Config file + flag/env overrides for broker URL + PAK
- [ ] Integration/e2e test of the apply loop (incl. edit→re-apply→prune)
- [ ] CLI reference doc + a "submit a folder" how-to
- [ ] Release packaging decided and wired

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Depends on T-0195 (Rust SDK helper).
- 2026-06-11: IMPLEMENTED (branch feat/i0021-raw-yaml-submission). New `brokkr-cli` crate (`crates/brokkr-cli`) producing the `brokkr` binary. Command `brokkr apply -f <path> --stack <name> [--target-label LABEL]...` — a thin shell over the contract-tested `BrokkrClient::apply`; prints created/updated/unchanged, exit 0 on success (incl. unchanged), `error: …`/exit 1 on failure. Connection config resolved flag > env (BROKKR_BROKER_URL/BROKKR_PAK) > `~/.brokkr/config` (TOML), with per-layer blank-is-unset and `/api/v1` normalization (src/config.rs). Tests: 9 unit (config precedence/normalize/parse) + 7 black-box binary integration tests in tests/cli.rs (help, version, missing-args, missing/malformed-config errors, config-accepted path) — all 16 pass; clippy clean. Live apply path covered transitively by the Rust SDK contract suite. PACKAGING DECISION: lockstep release binaries — added `build-cli-binaries` (matrix: linux amd64/arm64 native, macos x86_64/aarch64) + `publish-cli-binaries` jobs to .github/workflows/release.yml; cross-compiled tarballs `brokkr-<version>-<target>.tar.gz` attached to the GitHub Release via softprops/action-gh-release. Crate licensed Elastic-2.0 with source headers. Docs: how-to/cli-apply.md + reference/cli.md `brokkr` section + SUMMARY entry; mdbook builds. crates/* glob auto-includes the crate in all workspace CI; Cargo.lock updated. Remaining: CI green on PR #44.