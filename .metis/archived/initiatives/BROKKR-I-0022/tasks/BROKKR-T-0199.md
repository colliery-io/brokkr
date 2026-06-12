---
id: replace-retired-macos-13-runner-in
level: task
title: "Replace retired macos-13 runner in release.yml CLI build matrix"
short_code: "BROKKR-T-0199"
created_at: 2026-06-11T11:02:07.481760+00:00
updated_at: 2026-06-11T12:02:22.456199+00:00
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

# Replace retired macos-13 runner in release.yml CLI build matrix

## Parent Initiative

[[BROKKR-I-0022]]

## Objective

`release.yml:176` (`build-cli-binaries` matrix) pins `runner: macos-13`, which GitHub has retired (current runner-images: macos-14/15/26; x64 = `macos-15-intel`, `macos-15-large`, `macos-26-intel`). The `x86_64-apple-darwin` leg will fail on the next `v*` tag, and `publish-cli-binaries` needs all four legs — so no CLI binaries attach at all. Replace with a supported runner.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `x86_64-apple-darwin` leg uses `macos-15-intel` (preferred; free tier, supported into 2027) or cross-compiles from an arm64 runner with `targets: x86_64-apple-darwin`.
- [ ] Workflow YAML validates; matrix still covers linux amd64/arm64 + macos x86_64/aarch64.
- [ ] Build verified ahead of the real tag (temporary `workflow_dispatch` or scoped test job acceptable).

## Implementation Notes

brokkr-cli has no exotic native deps, so cross-compiling x86_64 from `macos-14`/`macos-15` (arm64) is viable if `macos-15-intel` is unavailable. Keep the tarball naming (`brokkr-<version>-x86_64-apple-darwin.tar.gz`) unchanged.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: DONE. Changed the `x86_64-apple-darwin` leg from the retired `macos-13` to `macos-14` (Apple Silicon), cross-compiling x86_64 instead of building natively. Chosen over `macos-15-intel` because GitHub has been retiring free Intel macOS runners and the label's availability/cost can't be proven pre-tag (release.yml runs only on `v*`). Cross-compile is clean: the toolchain step already does `targets: ${{ matrix.target }}`, and `cargo tree -p brokkr-cli` shows no `openssl-sys`/`native-tls` (no C TLS to cross-link). Both darwin legs now run on macos-14. Note: brokkr-cli's tree currently has NO TLS backend at all — separate latent concern flagged for backlog (https broker URLs).