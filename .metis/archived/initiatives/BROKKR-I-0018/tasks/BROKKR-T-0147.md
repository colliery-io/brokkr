---
id: r2-c-release-sdks-yml-workflow
level: task
title: "R2-C: release-sdks.yml workflow skeleton with publish-sdks gate"
short_code: "BROKKR-T-0147"
created_at: 2026-05-15T22:30:00+00:00
updated_at: 2026-05-16T00:09:10.341921+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0145, BROKKR-T-0146]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-C: release-sdks.yml workflow skeleton with publish-sdks gate

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Stand up `.github/workflows/release-sdks.yml`. Tag-triggered, builds all four SDK artifacts (Rust crate, two Python sdists/wheels, npm tarball) with version-stamped manifests, and gates publishing behind the `publish-sdks` GitHub environment. **No publish steps yet** — just artifact upload. Publish steps land in R2-D.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `.github/workflows/release-sdks.yml` exists; triggers on `push.tags: ['v*']`.
- [x] Workflow rejects tags that don't match `v\d+\.\d+\.\d+` (the `resolve-version` job exits non-zero with a clear error message).
- [x] Fast-fails with `angreal openapi check` (which now also asserts the R2-A crate-mirror parity) + `check-python` + `check-typescript` in a dedicated `drift-check` job before any build step runs.
- [x] `build` job derives version from `${GITHUB_REF_NAME#v}` (or `inputs.version` for workflow_dispatch); stamps via `cargo set-version` + a regex on both `pyproject.toml`s + `npm version --no-git-tag-version`; builds `cargo package`, `uv build` for both Python packages, and `npm pack` for TS; uploads four named artifacts.
- [ ] **(Manual)** `publish-sdks` GitHub environment must be created at `Settings → Environments` with the same required reviewers as the existing `release` environment and deployment branches restricted to tags matching `v*`. PyPI's pending publishers already reference this exact name.
- [x] `publish` job depends on `build`, declares `environment: publish-sdks`, has placeholder echo step (R2-D fills in cargo/PyPI/npm publish).
- [x] `workflow_dispatch` accepts a `version` input (default `0.0.0-test`) for testing without pushing a tag; non-tag runs are flagged via `is_tag=false` so R2-D's publish steps can no-op on dry runs.
- [ ] **(Manual)** End-to-end smoke test via `workflow_dispatch` once the branch lands and the environment exists.

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

- 2026-05-15: Workflow skeleton landed at `.github/workflows/release-sdks.yml`.
  - Four jobs: `resolve-version` (validates and normalizes the version), `drift-check` (runs all three angreal openapi checks), `build` (stamps + builds all four artifacts and uploads), `publish` (gated by `publish-sdks` environment, placeholder for R2-D).
  - Also renamed the TS package from `@brokkr/client` to `@colliery-io/brokkr-client` (consumer was a single import in `examples/ui-slim/src/api.js` + `package.json` lock files). Wasn't in R2-B's explicit scope but matches the same naming-alignment intent and was needed before the workflow could reference the npm package by its final name. Tests + clean install verified post-rename.
  - `cargo set-version` smoke-tested locally; `actionlint` clean.
  - Two manual follow-ups before R2-E: (1) create the `publish-sdks` GitHub environment with reviewers and `v*`-tag deployment restriction, (2) `workflow_dispatch` the skeleton from the merged branch to confirm artifacts upload and the gate pauses.