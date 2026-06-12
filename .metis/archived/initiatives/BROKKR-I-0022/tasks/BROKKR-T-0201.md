---
id: add-tag-vs-crate-version-guard-to
level: task
title: "Add tag-vs-crate-version guard to release.yml"
short_code: "BROKKR-T-0201"
created_at: 2026-06-11T11:02:07.583856+00:00
updated_at: 2026-06-11T12:02:22.555894+00:00
parent: release-pipeline-blockers-retired
blocked_by: []
archived: true

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0022
---

# Add tag-vs-crate-version guard to release.yml

## Parent Initiative

[[BROKKR-I-0022]]

## Objective

`release.yml` names artifacts from the tag (`brokkr-${GITHUB_REF_NAME#v}-…`, line ~190) but the binary reports `CARGO_PKG_VERSION` (clap `version` attr). Tagging `v0.7.0` without the lockstep bump commit ships tarballs labeled 0.7.0 containing a 0.6.0 binary — and similarly mislabels images/charts. Add a fail-fast guard.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] An early job/step asserts `${GITHUB_REF_NAME#v}` equals the workspace crate version (e.g. `cargo metadata --no-deps`) and fails with a clear message on mismatch.
- [ ] Guard runs before any build/publish job (make the test-suite jobs or a new tiny job depend on it).
- [ ] Allows `-rc`/`-beta`/`-alpha` suffix handling consistent with the existing prerelease detection.

## Implementation Notes

Lockstep versioning is project policy (containers, charts, all SDKs share the git-tag version) — the guard should also check `sdks/python/brokkr/pyproject.toml` and `sdks/typescript/brokkr-client/package.json` while it is there; they are stamped from the same bump.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE. Added a `verify-version` job to release.yml that runs first; `setup` (and thus the whole test/build/publish chain) now `needs: verify-version`, so a mistag fails in seconds instead of after the full suite. Logic: workspace crates (via `cargo metadata --no-deps`) must EXACTLY equal the tag (`${GITHUB_REF_NAME#v}`) — the CLI binary reports its crate version, so this is the load-bearing check; SDK manifests (Python pyproject ×2, TS package.json) are compared on numeric base `X.Y.Z` to tolerate PEP440/npm-vs-semver prerelease spelling. Validated locally against the real repo: tag v0.6.0 → PASS, v0.7.0 (un-bumped) → FAIL on all 7 crates + both SDKs, v0.6.0-rc.1 → FAIL (crates aren't at the rc, as expected). YAML parses (10 jobs). cargo is preinstalled on ubuntu-latest; metadata needs no network/lock resolution.