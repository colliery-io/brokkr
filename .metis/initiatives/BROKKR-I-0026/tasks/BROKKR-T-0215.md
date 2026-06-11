---
id: docs-staleness-sweep-binary-count
level: task
title: "Docs: staleness sweep (binary count, versions, container tags, rustdoc tags)"
short_code: "BROKKR-T-0215"
created_at: 2026-06-11T11:02:08.276101+00:00
updated_at: 2026-06-11T11:02:08.276101+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# Docs: staleness sweep (binary count, versions, container tags, rustdoc tags)

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

Fix every stale-claim finding from the docs sweep (excluding submission-example modernization — that is T-0216).

## Acceptance Criteria

- [ ] `reference/cli.md`: intro (line 3) says "two command-line binaries" — now three; scope the `## Configuration` (line 166: "Both binaries…", line 172: "no command-line flag for loading a configuration file" — false for `brokkr --config`) and Exit Codes sections explicitly to brokkr-broker/brokkr-agent.
- [ ] `how-to/sdks/rust.md:15`: `brokkr-client = "0.5"` → `"0.6"` (the page's own apply examples need 0.6.x to compile).
- [ ] `reference/health-endpoints.md:163,182,199`: example `"version": "0.5.0"` → 0.6.0.
- [ ] `how-to/install-operations.md`: dev-build pattern `0.0.0-develop.<ts>` → `0.0.0-main.<ts>` / `0.0.0-pr<N>.<ts>` (develop branch dropped in fffc39c; see build-and-test.yml:291,340); `--version 1.0.0/1.1.0` examples → 0.6.0 (lockstep — no 1.x exists).
- [ ] `reference/container-images.md`: drop/caveat the unpublished `brokkr-ui` image row (no workflow builds one); `develop-abc1234`/`develop` tag examples → `main`/`main-<sha>`/`pr-<N>`; add the `:nightly` tag row (nightly.yml pushes it).
- [ ] `explanation/publishing-strategy.md:58,92`: same develop-* fixes.
- [ ] Rustdoc unclosed-HTML warnings fixed at source: backtick `Api<DynamicObject>` at `crates/brokkr-agent/src/k8s/api.rs:273` and `Vec<String>` at `crates/brokkr-utils/src/config.rs:73`, regenerate API docs (verified: browsers swallow the type names in rendered output today).
- [ ] `how-to/sdks/README.md`: add a bullet for the shared folder-submission helpers.
- [ ] `explanation/architecture.md` C4 L1/L2: add the CLI/SDKs as the engineer/CI front door (C4 convention per project practice).
- [ ] `how-to/README.md` Deploy & manage row: add cli-apply.md (currently only in SUMMARY).
- [ ] `mdbook build` clean of the two known warnings; add a release-checklist note (or grep) for version strings.

## Status Updates

*To be added during implementation*
