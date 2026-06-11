---
id: brokkr-cli-apply-f-folder-of
level: task
title: "brokkr CLI: apply -f folder of manifests"
short_code: "BROKKR-T-0198"
created_at: 2026-06-11T02:19:34.293955+00:00
updated_at: 2026-06-11T02:19:34.293955+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] `brokkr apply -f ./manifests/ --stack test` creates/updates the stack and submits the bundle; re-run with no changes reports unchanged
- [ ] `--target-label k=v` sets stack labels (fan-out targeting) in the same call
- [ ] Config file + flag/env overrides for broker URL + PAK
- [ ] Integration/e2e test of the apply loop (incl. edit→re-apply→prune)
- [ ] CLI reference doc + a "submit a folder" how-to
- [ ] Release packaging decided and wired

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Depends on T-0195 (Rust SDK helper).
