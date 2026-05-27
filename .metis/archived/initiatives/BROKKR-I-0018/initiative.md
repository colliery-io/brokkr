---
id: sdk-publishing-rust-python
level: initiative
title: "SDK Publishing: Rust / Python / TypeScript"
short_code: "BROKKR-I-0018"
created_at: 2026-05-15T22:08:56.351163+00:00
updated_at: 2026-05-16T03:53:57.318038+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: sdk-publishing-rust-python
---

# SDK Publishing: Rust / Python / TypeScript

## Context

BROKKR-I-0017 shipped three generated SDKs (`brokkr-client` Rust crate, `brokkr` Python package + `brokkr-broker-client` low-level, `brokkr-client` TypeScript package). They are in the workspace, tested, documented, and used by the demo UI — but nothing is published to a public registry. Every consumer is forced into a path / git dep, which blocks adoption by anyone outside this repo.

This initiative wires up the publishing pipeline. Versioning is **lockstep**: a single `v*` git tag drives broker images, helm charts, and all three SDKs. Decided 2026-05-15.

Accounts and credentials already in place:
- **PyPI**: pending publishers for `brokkr-client` and `brokkr-client-generated` (Trusted Publishing, no token; targets `release-sdks.yml` + `publish-sdks` environment).
- **npm**: `@colliery-io` org reserved; `NPM_TOKEN` in repo secrets.
- **crates.io**: `CARGO_REGISTRY_TOKEN` in repo secrets; crate name `brokkr-client` claimed on first publish.

## Goals & Non-Goals

**Goals:**
- Publish `brokkr-client` to crates.io on every `v*` tag, in lockstep with the broker version.
- Publish `brokkr-client` + `brokkr-client-generated` to PyPI on every `v*` tag, in lockstep.
- Publish `@colliery-io/brokkr-client` to npm on every `v*` tag, in lockstep.
- Stamp the git-tag version into all four packages at publish time (source stays `0.0.0`).
- A manual-approval gate (`publish-sdks` GitHub environment) sits between build and publish, parallel to the existing container/helm `release` environment.
- Cut a first real release (`v0.3.0`) end-to-end across all four artifacts.

**Non-Goals:**
- Pre-release / RC publishing (alpha, beta, rc tags). Out of scope for v1 of this pipeline; the existing release workflow already handles them for containers, but SDK publishing is start-with-stable-only.
- Publishing `brokkr-models`, `brokkr-utils`, `brokkr-agent`, or `brokkr-broker` as separate crates. They stay workspace-internal.
- Renaming Python packages from `brokkr` → `brokkr-client` in-tree. We'll do the rename as part of this work since it's coupled to the PyPI claim, but no broader Python module refactor.
- Unscoped npm `brokkr-client` reservation. Scoped `@colliery-io/brokkr-client` only.
- Coordinated cross-language versioning *policy* (e.g. handling SDK-breaking changes that don't touch the broker). Deferred — lockstep handles the v1 case where SDKs are byte-mechanically derived from the spec.

## Detailed Design

### Architecture overview

```
                           v0.3.0 git tag pushed
                                   |
                                   v
                        +----------+----------+
                        |    release-sdks.yml |
                        +----------+----------+
                                   |
                        +----------+----------+
                        |  build all packages |
                        |  (version-stamp)    |
                        +----------+----------+
                                   |
                                   v
                        +----------+----------+
                        | publish-sdks env    |
                        |  (manual approval)  |
                        +----+----+----+------+
                             |    |    |
                +------------+    |    +-------------+
                v                 v                  v
        +-------+--------+  +-----+-------+  +-------+-------+
        | cargo publish  |  | PyPI upload |  | npm publish   |
        | brokkr-client  |  | brokkr-     |  | @colliery-io/ |
        |                |  |  client +   |  |  brokkr-      |
        |                |  |  -generated |  |  client       |
        +----------------+  +-------------+  +---------------+
```

The existing `release.yml` (containers + helm) stays untouched; it triggers off the same tag and runs in parallel.

### The version-stamping problem

All four packages currently read `version = "0.0.0"` in their manifests. We do **not** want to commit version bumps to the source tree on every release — that creates a chicken-and-egg situation and pollutes the diff. Instead, the publish workflow rewrites the version in-place from the tag:

- **Rust**: `cargo set-version --workspace <version>` (from `cargo-edit`) before `cargo publish -p brokkr-client`.
- **Python**: `uv version <version>` or a `sed` against `pyproject.toml`; the build picks it up.
- **TypeScript**: `npm version <version> --no-git-tag-version` before `npm publish`.

The rewritten manifests are not committed back — they exist only in the workflow's checkout.

### The progenitor spec-path problem (Rust)

`crates/brokkr-client/src/lib.rs` does:

```rust
progenitor::generate_api!(spec = "../../openapi/brokkr-v1.json", ...);
```

`cargo publish` packages only the crate directory, so the relative path breaks for downstream consumers. Fix: **vendor the spec into the crate at publish time**.

- The workflow `cp openapi/brokkr-v1.json crates/brokkr-client/spec/brokkr-v1.json` before `cargo publish`.
- The macro path becomes `spec = "spec/brokkr-v1.json"` always — the same file ships in-tree for workspace builds (gitignored copy or a symlink; or just commit it and add a CI check that it matches `openapi/brokkr-v1.json`).
- Simplest variant: commit the spec copy at `crates/brokkr-client/spec/brokkr-v1.json` and have `angreal openapi export` write to both locations. Drift check enforces parity. The cost is a duplicate file in git; the upside is one canonical macro path that works identically in-tree and in published builds.

### The Python wrapper rename

Today: `sdks/python/brokkr/` ships as `brokkr` on PyPI, depends on `brokkr-broker-client`. We rename:
- Distribution name `brokkr` → `brokkr-client`. Import name stays `brokkr` (PEP 8 / community pattern: distribution and import names can diverge).
- Distribution name `brokkr-broker-client` → `brokkr-client-generated`. Import name stays `brokkr_broker_client` (changing the import name is a much bigger refactor for zero ergonomic gain).

Affected: `sdks/python/brokkr/pyproject.toml`, `sdks/python/brokkr-client/pyproject.toml`, the `tool.uv.sources` path-dep names, docs that mention install commands.

### Manual approval gate

Create `publish-sdks` GitHub environment with the same required-reviewer set as the existing `release` environment. The PyPI Trusted Publisher config already references this environment name. Lets approvers green-light containers without auto-publishing SDKs (or vice versa) during the shakedown.

### Lockstep enforcement

The workflow refuses to run if the tag version doesn't match a semver pattern (`v\d+\.\d+\.\d+`). All four publishes share the same `${{ github.ref_name }}` derived version string. There is no per-package version input.

## Alternatives Considered

- **Independent SDK versions** — each SDK semver'd separately. Rejected: SDKs are mechanically derived from the spec, so an independent version carries no extra information; it just adds three changelogs and a compatibility matrix. Revisit if SDK wrappers grow non-trivial hand-written surface.
- **Fold SDK publishing into existing `release.yml`** — one workflow, one approval gate for everything. Rejected: containers and SDKs have different failure modes (registry availability, build steps, audit trails). A second workflow keeps blast radius small and lets us approve them independently.
- **Pre-generate Rust client into the crate at commit time** — vendor the generated code so `cargo publish` doesn't need `progenitor`. Rejected for now: doubles the size of the in-tree crate, fights with the auto-regen workflow, and the vendor-at-publish-time approach gives us the same end-state on crates.io with less churn.
- **Publish only the high-level Python wrapper, vendor the generated client inside it** — single package on PyPI. Rejected: bigger refactor (generation paths, import structure), and the two-package shape mirrors the Rust workspace boundary cleanly.

## Implementation Plan

Decomposes into tasks (see decompose phase). Rough order:

1. **R2-A: Spec-path fix for `brokkr-client` Rust crate**. Move/duplicate `openapi/brokkr-v1.json` into `crates/brokkr-client/spec/`, update the macro path, teach `angreal openapi export` to write both copies, and add a CI drift check between them.
2. **R2-B: Python rename**. Update `pyproject.toml` distribution names (wrapper to `brokkr-client`, generated to `brokkr-client-generated`), update `tool.uv.sources`, update docs and `examples/ui-slim` references if any.
3. **R2-C: `release-sdks.yml` workflow**. Triggered by `v*` tags. Builds all four artifacts with version-stamped manifests. Gated by `publish-sdks` environment.
4. **R2-D: Publish jobs**. Three matrix entries (cargo, twine via PyPI Trusted Publishing, npm). Each can fail independently without poisoning the others.
5. **R2-E: First release**. Cut `v0.3.0` after R2-A through R2-D land. Validate every package is installable from a clean machine.
6. **R2-F: Docs update**. Flip the SDK docs (`docs/src/how-to/sdks/`) from "not yet published" to install instructions referencing the registries.

## Risks

- **First crates.io publish locks the name to the publishing identity**. The token in `CARGO_REGISTRY_TOKEN` becomes the permanent owner. Verify it's the org-owned account, not a personal one. Mitigation: do a `--dry-run` from the workflow first, confirm the identity, then real publish.
- **PyPI Trusted Publisher matching is exact** on owner + repo + workflow filename + environment name. Any rename of `release-sdks.yml` or `publish-sdks` after-the-fact breaks publish until pending publishers are updated.
- **Lockstep + spec drift** — if a `v*` tag is cut from a commit where the spec is stale, all SDKs ship a stale shape. Existing CI drift check protects against this on PR; we should also assert it in `release-sdks.yml` as a fast-fail before any publish runs.
- **The `_generated` suffix on the Python package name is unusual** but defensible. Watch user feedback after the first release.

## Status

Discovery — populated 2026-05-15. Awaiting decomposition.