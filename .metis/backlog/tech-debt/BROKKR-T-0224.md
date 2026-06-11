---
id: ci-docker-hub-auth-upload-artifact
level: task
title: "CI: Docker Hub auth + upload-artifact retry + cache efficiency (deferred from T-0218)"
short_code: "BROKKR-T-0224"
created_at: 2026-06-11T20:16:07.862693+00:00
updated_at: 2026-06-11T20:16:07.862693+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#task"
  - "#phase/backlog"


exit_criteria_met: false
---

# CI: Docker Hub auth + upload-artifact retry + cache efficiency (deferred from T-0218)

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

The contained flake fixes from [[BROKKR-T-0218]] (imagetools retries, node 22, paths filter) shipped. These remaining items either need a secret/decision from the maintainer or are efficiency work.

## Backlog Item Details

### Type
- [x] Tech Debt

## Acceptance Criteria

- [ ] **Docker Hub auth (NEEDS SECRET)**: the highest-value item — the repeated 6-suite compose failures were Docker-Hub rate-limited pulls. Add `docker/login-action` with `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` (create the repo secrets first) before the compose-based suites (sdk_contract_tests, integration_tests, e2e, helm) and the image builds (build-and-test, release, nightly).
- [ ] **upload-artifact ETIMEDOUT mitigation**: `continue-on-error: true` + `id:` on the digest uploads (release.yml, build-and-test.yml) with a guarded re-upload step on failure.
- [ ] **Cache efficiency**: setup.yml warms a `…-cargo-` cache that downstream jobs (`…-rust-build-`) never read — unify the keys/paths or drop the setup build; build-and-test.yml `no-cache: true` forces full base-image re-pulls each PR (use gha cache); docs.yml `curl` needs `--retry`, and `cargo install plissken` should be cached/binstalled.
- [ ] **DECISION**: `publish-cli-binaries` carries `environment: release` (a second approval round after charts) — keep or drop?

## Status Updates

*To be added during implementation*
