---
id: r2-d-publish-steps
level: task
title: "R2-D: Wire SDK publish steps (cargo, PyPI, npm)"
short_code: "BROKKR-T-0148"
created_at: 2026-05-15T22:30:00.000000+00:00
updated_at: 2026-05-15T22:30:00.000000+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0147]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-D: Wire SDK publish steps (cargo, PyPI, npm)

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Fill in the `publish` job in `release-sdks.yml` with three publish steps — one per registry. Each step is independent: a registry outage or auth failure in one shouldn't poison the others' publish state (they can be retried via re-running just the failed step).

## Acceptance Criteria

- [ ] **cargo**: `cargo publish -p brokkr-client` runs against the stamped manifest + vendored spec. `--dry-run` runs first; the real publish gates on dry-run success. Uses `CARGO_REGISTRY_TOKEN` from repo secrets.
- [ ] **PyPI**: `pypa/gh-action-pypi-publish@release/v1` uploads both `brokkr-client` and `brokkr-client-generated` distributions. No token; uses Trusted Publishing via `id-token: write` permission. Each package gets its own step (or matrix entry) so one failure doesn't block the other.
- [ ] **npm**: `npm publish --access public` for `@colliery-io/brokkr-client`. Uses `NPM_TOKEN` from repo secrets. Asserts via `--dry-run` that the package version matches the stamped version before real publish.
- [ ] All three publish steps declare `continue-on-error: false` and report status individually so a partial failure is visible in the run's summary.
- [ ] Workflow concurrency group prevents two `v*` tag runs from racing.
- [ ] After successful run against a real tag, all four packages are installable from their public registries with `pip install brokkr-client==<v>`, `cargo add brokkr-client@<v>`, `npm install @colliery-io/brokkr-client@<v>`.

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

*To be added during implementation*
