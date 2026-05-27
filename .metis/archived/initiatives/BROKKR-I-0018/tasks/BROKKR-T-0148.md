---
id: r2-d-wire-sdk-publish-steps-cargo
level: task
title: "R2-D: Wire SDK publish steps (cargo, PyPI, npm)"
short_code: "BROKKR-T-0148"
created_at: 2026-05-15T22:30:00+00:00
updated_at: 2026-05-16T00:21:09.688061+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0147]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-D: Wire SDK publish steps (cargo, PyPI, npm)

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Fill in the `publish` job in `release-sdks.yml` with three publish steps — one per registry. Each step is independent: a registry outage or auth failure in one shouldn't poison the others' publish state (they can be retried via re-running just the failed step).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] **cargo**: `cargo publish -p brokkr-client` runs against the stamped manifest + vendored spec. `--dry-run` runs first (and runs on workflow_dispatch too); the real publish only fires when `is_tag=true`. Uses `CARGO_REGISTRY_TOKEN`.
- [x] **PyPI**: `pypa/gh-action-pypi-publish@release/v1` uploads `brokkr-client` and `brokkr-client-generated` from their respective artifact directories. No token; `permissions.id-token: write` is set on the job for Trusted Publishing. The two packages are independent steps so one failure doesn't block the other.
- [x] **npm**: `npm publish --access public` for `@colliery-io/brokkr-client`. Uses `NPM_TOKEN` (consumed via `NODE_AUTH_TOKEN` env). Dry-run validates the tarball + version before the real publish.
- [x] Each publish step fails the job by default (default `continue-on-error: false`), so partial failures surface in the run summary by step name.
- [x] Workflow-level `concurrency` block (`group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: false`) blocks racing tag runs without ever canceling a publish mid-flight.
- [ ] **(Manual)** Verified end-to-end against a real tag in R2-E. Cannot be checked from this task alone.

## Implementation Notes

### Technical Approach

1. Three sibling jobs (or three steps in one job) — sibling jobs give cleaner run-summary UX. Each `needs: version-stamp-and-build` and `environment: publish-sdks`.
2. The `cargo publish --dry-run` doesn't actually skip network — it validates the tarball and connects to the registry. That's the smoke check we want before committing.
3. For npm `--access public` is required for scoped packages; without it npm rejects with "402 Payment Required" (the default scope visibility is private and private scopes require a paid account).
4. PyPI Trusted Publishing requires `permissions: id-token: write` on the job, not at the workflow level. Don't grant it broadly.
5. Concurrency group: `group: release-sdks-${{ github.ref }}` with `cancel-in-progress: false`. Block double-runs, never cancel a publish mid-flight.

### Dependencies

- Hard: [[BROKKR-T-0147]].

### Risk Considerations

- First-publish identity lock-in (crates.io and PyPI). The `CARGO_REGISTRY_TOKEN` and the GitHub identity that triggers PyPI Trusted Publishing become the permanent owners. The token under `colliery-io/brokkr` repo secrets should be from the org-owned account; if it's a personal account, fix that *before* this task ships its first real publish.
- A failed cargo publish followed by a retry will fail with "crate version already taken" because version numbers are permanent. The retry path is: revert to a development tag, fix, cut a new patch tag. Document this in R2-E's runbook.
- PyPI's pending publisher record matches exactly on workflow filename and environment name. If `release-sdks.yml` is later renamed, the pending publishers must be updated *first* or publishes break.

## Status Updates

- 2026-05-15: Replaced the placeholder `publish` job with real publish steps for all three registries.
  - **cargo**: re-stamps the version (artifacts come from the build job but cargo publish rebuilds from source), runs `--dry-run` unconditionally, then real publish gated on `is_tag=true`.
  - **PyPI**: two `pypa/gh-action-pypi-publish@release/v1` calls — one each for `dist/python-wrapper` and `dist/python-generated`. Trusted Publishing via `permissions.id-token: write` on the job. Independent steps so partial failure is visible and recoverable.
  - **npm**: `npm publish --access public` for the scoped `@colliery-io/brokkr-client` package. Dry-run first; real publish gated on `is_tag=true`. Uses `NPM_TOKEN` via `NODE_AUTH_TOKEN`.
  - `actionlint` clean. End-to-end verification happens in R2-E once the tag is cut.