---
id: ci-docker-hub-auth-upload-artifact
level: task
title: "CI: Docker Hub auth + upload-artifact retry + cache efficiency (deferred from T-0218)"
short_code: "BROKKR-T-0224"
created_at: 2026-06-11T20:16:07.862693+00:00
updated_at: 2026-06-12T03:02:43.744469+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
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

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Docker Hub auth (NEEDS SECRET)**: the highest-value item — the repeated 6-suite compose failures were Docker-Hub rate-limited pulls. Add `docker/login-action` with `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` (create the repo secrets first) before the compose-based suites (sdk_contract_tests, integration_tests, e2e, helm) and the image builds (build-and-test, release, nightly).
- [ ] **upload-artifact ETIMEDOUT mitigation**: `continue-on-error: true` + `id:` on the digest uploads (release.yml, build-and-test.yml) with a guarded re-upload step on failure.
- [ ] **Cache efficiency**: setup.yml warms a `…-cargo-` cache that downstream jobs (`…-rust-build-`) never read — unify the keys/paths or drop the setup build; build-and-test.yml `no-cache: true` forces full base-image re-pulls each PR (use gha cache); docs.yml `curl` needs `--retry`, and `cargo install plissken` should be cached/binstalled.
- [ ] **DECISION**: `publish-cli-binaries` carries `environment: release` (a second approval round after charts) — keep or drop?

## Status Updates

*To be added during implementation*

## Status Updates

- 2026-06-11: ACTIVE. Maintainer created the DOCKERHUB_USERNAME + DOCKERHUB_TOKEN repo secrets. Wired authenticated Docker Hub pulls (the real fix for the shared-runner-IP anonymous 100/6h rate-limit flake class) across every job that pulls from docker.io:
  - 3 compose suites (integration_tests, sdk_contract_tests, e2e_tests): job-level `env.DOCKERHUB_USERNAME` from the secret + a guarded `docker/login-action` (no registry = Docker Hub) before the hoverkraft compose-action.
  - 3 buildx build jobs (build-and-test build-images, release build-release-images, nightly nightly-images): same login before the build, alongside the existing GHCR login (which is unchanged — publishing still uses GHCR_TOKEN).
  - Secret propagation: the 3 compose suites are reusable (`workflow_call`); added `secrets: inherit` to all 6 call sites (main.yml ×2, release.yml ×2, nightly.yml ×2) so the secret reaches them. (`secrets` context isn't available in workflow-level `env` or in `if:`, hence the job-level env + `if: env.DOCKERHUB_USERNAME != ''` guard — which also keeps fork PRs / secret-absent runs working anonymously, exactly as before.)
  - `actionlint` clean (exit 0) on all 7 edited workflows.
  Remaining (still open in this task): upload-artifact ETIMEDOUT retry, the setup/cache-key efficiency items, and the publish-cli-binaries `environment: release` keep/drop DECISION.
- 2026-06-12: DECISION (maintainer): keep the `publish-cli-binaries` `environment: release` gate. No workflow change — the gate stays as-is (it's the approval used at the v0.7.0 release). This closes the last open sub-item of T-0218/T-0224; the only remaining deferrals (upload-artifact ETIMEDOUT retry, setup/cache-key efficiency) are pure CI-efficiency nice-to-haves with no outstanding decision.