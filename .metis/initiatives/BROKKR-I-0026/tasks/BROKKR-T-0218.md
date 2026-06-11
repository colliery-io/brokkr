---
id: ci-flake-hardening-and-efficiency
level: task
title: "CI: flake hardening and efficiency (retries, registry auth, cache keys)"
short_code: "BROKKR-T-0218"
created_at: 2026-06-11T11:02:08.427026+00:00
updated_at: 2026-06-11T19:53:03.546611+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# CI: flake hardening and efficiency (retries, registry auth, cache keys)

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

Harden the two flake classes that bit us this week (Docker Hub pull timeout during `imagetools create`; artifact-upload `Failed to CreateArtifact: ETIMEDOUT`) and stop paying for CI work nobody uses.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `docker buildx imagetools create`/`inspect` wrapped in retry (release.yml:154-158, build-and-test.yml:167-171) — `nick-fields/retry` or a 3-attempt shell loop.
- [ ] Authenticated Docker Hub pulls (rate limits): `docker/login-action` with Docker Hub creds in build-and-test.yml, release.yml, nightly.yml, and the three compose-based test workflows. NEEDS DECISION: add DOCKERHUB_USERNAME/DOCKERHUB_TOKEN secrets.
- [ ] Artifact-upload timeout mitigation: `continue-on-error: true` + `id:` on the digest uploads (release.yml:100, build-and-test.yml:110; release.yml:204), with a guarded duplicate upload step on failure (upload-artifact has no retry input; `nick-fields/retry` cannot wrap `uses:`).
- [ ] `docs.yml:34-41` curl gets `--retry 5 --retry-all-errors --connect-timeout 15`; `cargo install plissken` (:45) cached or binstalled.
- [ ] setup.yml cache key actually reused: setup builds the workspace into `…-cargo-…` (paths ~/.cargo + target) but every downstream job restores `…-rust-build-…` (target only) — unify key/paths or delete the setup build step (currently pure wasted compute on every run).
- [ ] openapi.yml push trigger (lines 5-6) gets the same `paths:` filter as its PR trigger (docs-only pushes currently pay the full drift suite).
- [ ] `node-version: '20'` pins (EOL April 2026) bumped to 22/24: openapi.yml:64, release-sdks.yml:83,115, sdk_contract_tests.yml:38.
- [ ] build-and-test.yml:100 `no-cache: true` replaced with gha cache (flake + cost amplifier on every PR).
- [ ] release.yml:199 `cp README.md LICENSE.txt … || true` loses the `|| true` (silent omission on rename).
- [ ] DECISION for Dylan: release.yml:266 `publish-cli-binaries` carries `environment: release` (second approval round after charts) — keep or drop.
- [ ] release-sdks.yml:10 stale comment ("Source manifests stay at 0.0.0" — they are lockstep 0.6.0 now) fixed.

## Status Updates

*To be added during implementation*
## Status Updates

- 2026-06-11: PARTIAL — contained flake fixes done; secret-dependent + efficiency items noted below. Branch feat/i0026-docs-ci-hygiene.
  DONE:
  - **imagetools create + inspect retries** (release.yml, build-and-test.yml): wrapped both in a 3-attempt shell loop with 15s backoff. This is the exact flake that failed `Create Multi-Arch Manifests` repeatedly this week (transient registry errors / read-after-write races).
  - **node-version 20 → 22** (EOL April 2026): openapi.yml, release-sdks.yml ×2, sdk_contract_tests.yml.
  - **openapi.yml push trigger** gained the same `paths:` filter as the PR trigger — a docs-only main push no longer pays the full drift suite (cargo build + SDK regen + npm ci).
  - **release.yml** `cp README.md LICENSE.txt … || true` → no `|| true` (a future rename now fails loudly instead of silently shipping a tarball without them).
  - **release-sdks.yml** stale "stay at 0.0.0" comment corrected to lockstep-versioned.
  DEFERRED / NEEDS DECISION (Dylan):
  - **Docker Hub auth (DOCKERHUB_USERNAME/DOCKERHUB_TOKEN secrets)** — the biggest lever: the 6-suite compose failures this week were Docker-Hub rate-limited pulls. Adding `docker/login-action` before the compose suites needs the secrets created first. Highest-value follow-up.
  - **upload-artifact ETIMEDOUT mitigation** (release.yml digests, build-and-test.yml): `continue-on-error` + guarded re-upload step.
  - **Efficiency**: setup.yml warms a `…-cargo-` cache that downstream jobs (`…-rust-build-`) don't read; build-and-test.yml `no-cache: true` forces full base-image re-pulls; docs.yml `curl` lacks `--retry` and `cargo install plissken` isn't cached.
  - **publish-cli-binaries `environment: release`** adds a second approval round after charts — keep or drop?
