---
id: r2-c-release-sdks-workflow
level: task
title: "R2-C: release-sdks.yml workflow skeleton with publish-sdks gate"
short_code: "BROKKR-T-0147"
created_at: 2026-05-15T22:30:00.000000+00:00
updated_at: 2026-05-15T22:30:00.000000+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0145, BROKKR-T-0146]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-C: release-sdks.yml workflow skeleton with publish-sdks gate

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Stand up `.github/workflows/release-sdks.yml`. Tag-triggered, builds all four SDK artifacts (Rust crate, two Python sdists/wheels, npm tarball) with version-stamped manifests, and gates publishing behind the `publish-sdks` GitHub environment. **No publish steps yet** — just artifact upload. Publish steps land in R2-D.

## Acceptance Criteria

- [ ] `.github/workflows/release-sdks.yml` exists; triggers on `push.tags: ['v*']`.
- [ ] Workflow rejects tags that don't match `v\d+\.\d+\.\d+` (no pre-release tags in v1; reject with a clear error).
- [ ] Fast-fails with `angreal openapi check` + `check-python` + `check-typescript` + the new spec parity check from R2-A before any build step runs.
- [ ] One `version-stamp-and-build` job that:
  - Derives version from `${{ github.ref_name }}` stripping the `v` prefix.
  - Stamps the version into `crates/brokkr-client/Cargo.toml` (via `cargo set-version` or `sed`), the two `pyproject.toml`s, and `sdks/typescript/brokkr-client/package.json`.
  - Builds: `cargo package -p brokkr-client`, `uv build` for both Python packages, `npm pack` for TS.
  - Uploads all artifacts to the workflow run.
- [ ] A `publish-sdks` GitHub environment exists (Settings → Environments) with the same required reviewers as the existing `release` environment; deployment branches restricted to tags matching `v*`.
- [ ] A `publish` job depends on `version-stamp-and-build`, declares `environment: publish-sdks`, but does nothing yet (placeholder echo). This is what R2-D fills in.
- [ ] A workflow_dispatch input is added so we can test the skeleton without pushing a tag (uses a synthetic version like `v0.0.0-test`).
- [ ] Running it against a test tag produces uploaded artifacts and pauses at the approval gate.

## Implementation Notes

### Technical Approach

1. Crib structure from existing `.github/workflows/release.yml` — same trigger style, same approval-gate pattern. Don't fold into that workflow; concerns are different and approval gates are independent.
2. Version stamping. For Cargo: `cargo install cargo-edit && cargo set-version -p brokkr-client $VERSION`. For Python: `uv version $VERSION` per package, or `sed -i "s/^version = \".*\"/version = \"$VERSION\"/"`. For npm: `npm version $VERSION --no-git-tag-version`.
3. Do not commit the stamped manifests back to the branch. Stamping is workflow-local.
4. Drift check before build. The point: if a tag is cut from a commit where someone forgot to regenerate, we don't ship inconsistent SDKs. Same `angreal openapi check*` commands the PR drift workflow already runs.
5. Manual `workflow_dispatch` with a version input is the testability hook. Avoids needing to push throwaway tags to validate the workflow.

### Dependencies

- Hard: [[BROKKR-T-0145]] (vendored spec path required for `cargo package`).
- Hard: [[BROKKR-T-0146]] (Python distribution names must be final before stamping).

### Risk Considerations

- The `publish-sdks` environment is referenced exactly by PyPI's pending publishers. A typo here means PyPI's Trusted Publishing fails — and the error message is unhelpful. Double-check the environment name matches: `publish-sdks`.
- `cargo set-version` only ships with `cargo-edit`. Install it in the workflow, don't assume it's preinstalled on the runner.

## Status Updates

*To be added during implementation*
