---
id: bump-node-20-16-github-actions-to
level: task
title: "Bump Node-20/16 GitHub Actions to Node-24 majors across workflows"
short_code: "BROKKR-T-0200"
created_at: 2026-06-11T11:02:07.533160+00:00
updated_at: 2026-06-11T11:02:07.533160+00:00
parent: release-pipeline-blockers-retired
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0022
---

# Bump Node-20/16 GitHub Actions to Node-24 majors across workflows

## Parent Initiative

[[BROKKR-I-0022]]

## Objective

GitHub forces Node-20 actions to run on Node 24 starting 2026-06-16. Upgrade every affected action to its verified Node-24 major and let the changes soak on main before the date. All target versions were verified node24 via each tag's `action.yml`.

## Acceptance Criteria

- [ ] `hoverkraft-tech/compose-action@v2.6.0` → `v3.0.0` — e2e_tests.yml:43, integration_tests.yml:46, sdk_contract_tests.yml:57
- [ ] `azure/setup-helm@v3` (node16!) → `v5.0.0` — build-and-test.yml:186,237,271,321; release.yml:225 (keep `version: v3.12.0` input for the same Helm binary)
- [ ] `docker/build-push-action@v5` → `v7.2.0` — build-and-test.yml:94, release.yml:81, nightly.yml:69
- [ ] `docker/setup-buildx-action@v3` → `v4.1.0`; `docker/login-action@v3` → `v4.2.0`; `docker/setup-qemu-action@v3` → `v4.1.0`; `docker/metadata-action@v5` → `v6.1.0` (build-and-test.yml, release.yml, nightly.yml occurrences)
- [ ] `softprops/action-gh-release@v2` → `v3.0.0` — release.yml:255,280
- [ ] `dorny/paths-filter@v3` → `v4.0.1` — main.yml:32
- [ ] `astral-sh/setup-uv@v3` → `v8.2.0` — release-sdks.yml:78,110
- [ ] Full CI green on main after the bumps, before 2026-06-16.

## Implementation Notes

`actions/delete-package-versions@v5` (nightly.yml:270) has NO node24 release — nothing to bump; it is already `continue-on-error: true`. Watch the first nightly after 06-16. Already clean (verified): checkout@v6, setup-python@v6, cache@v5, upload-artifact@v7, download-artifact@v8, setup-node@v5/v6, github-script@v9, deploy-pages@v5, dtolnay/rust-toolchain, pypa/gh-action-pypi-publish. Major bumps can change inputs/defaults — read each action's release notes for breaking changes (esp. build-push-action v5→v7 and gh-release v2→v3).

## Status Updates

*To be added during implementation*
