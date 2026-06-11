---
id: docs-staleness-sweep-binary-count
level: task
title: "Docs: staleness sweep (binary count, versions, container tags, rustdoc tags)"
short_code: "BROKKR-T-0215"
created_at: 2026-06-11T11:02:08.276101+00:00
updated_at: 2026-06-11T21:08:02.661727+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# Docs: staleness sweep (binary count, versions, container tags, rustdoc tags)

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

Fix every stale-claim finding from the docs sweep (excluding submission-example modernization — that is T-0216).

## Acceptance Criteria

## Acceptance Criteria

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
## Status Updates

- 2026-06-11: DONE (high-value fixes; a few lower-impact items noted). Branch feat/i0026-docs-ci-hygiene.
  - **rust.md** install snippet `brokkr-client = "0.5"` → `"0.6"` (the 0.5 pin couldn't compile the page's own apply examples).
  - **reference/cli.md**: intro "two command-line binaries" → three (adds `brokkr`); scoped the `## Configuration` section ("Both binaries…", "no command-line flag…") to the two SERVER binaries, noting `brokkr` is configured separately / takes `--config`.
  - **health-endpoints.md**: example `"version": "0.5.0"` → 0.6.0 (×3).
  - **Rustdoc unclosed-HTML mangling**: backticked `Api<DynamicObject>` (k8s/api.rs:273) and `Vec<String>` (config.rs:73) in the source doc comments — these generated the two mdbook "unclosed HTML tag" warnings and the type names vanished from rendered output. (The generated api/rust/*.md regenerate from these in the docs pipeline.)
  - **container-images.md**: caveated the unpublished `brokkr-ui` row (no CI builds it), replaced `develop-*`/`:develop` examples with `main`, added the `:nightly` tag row.
  - **explanation/publishing-strategy.md**: `develop-abc1234` → `main-abc1234`.
  - **how-to/README.md**: added the cli-apply.md link to the Deploy & manage row.
  REMAINING (lower impact, follow-on): install-operations.md dev-build pattern (`0.0.0-develop.*` → `0.0.0-main.*`/`-pr<N>.*`) + `1.x` version examples → 0.6.0; architecture.md C4 L1/L2 (add CLI/SDKs as the front door); sdks/README.md shared folder-helper bullet. SUMMARY↔disk and cross-links were verified clean in the original sweep.

- 2026-06-11: RESIDUALS (mostly) DONE. install-operations.md: `--version 1.0.0/1.1.0` → 0.6.0, dev-build pattern `0.0.0-develop.<ts>` → `0.0.0-main.<ts>` with the `0.0.0-pr<N>.<ts>` note. sdks/README.md: added the shared folder-helper (submit_manifests/apply) bullet. ONE remaining: architecture.md C4 L1/L2 diagrams (add the CLI/SDKs as the engineer/CI front door alongside the HTTPS REST API) — a diagram edit, low impact.

- 2026-06-11: architecture.md C4 done — the L1/L2 engineer & CI/CD → broker relationships now read "brokkr CLI / SDKs / REST API" (the CLI/SDKs are the front door over the REST API). T-0215 fully complete.