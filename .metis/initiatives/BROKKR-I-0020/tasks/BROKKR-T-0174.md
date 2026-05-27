---
id: b1-sdk-lockstep-version-bump-to-0
level: task
title: "B1: SDK lockstep version bump to 0.5.0 + CHANGELOG entries"
short_code: "BROKKR-T-0174"
created_at: 2026-05-24T12:56:43.000000+00:00
updated_at: 2026-05-24T12:56:43.000000+00:00
parent: BROKKR-I-0020
blocked_by:
  - BROKKR-T-0170
  - BROKKR-T-0171
  - BROKKR-T-0172
  - BROKKR-T-0173
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
initiative_id: BROKKR-I-0020
---

# B1: SDK lockstep version bump to 0.5.0 + CHANGELOG entries

## Parent Initiative

[[BROKKR-I-0020]]

## Objective

Apply the lockstep version bump required by `project_release_versioning`:
containers, helm charts, and all three SDKs share the git-tag version.
I-0019 added new endpoints + a new wire crate but no versions were bumped.
This task lands 0.5.0 across the board with CHANGELOG entries so the
eventual tag-and-publish is one motion.

## Acceptance Criteria

- [ ] `brokkr-wire` Cargo.toml bumped to 0.5.0
- [ ] `brokkr-client` (Rust SDK) Cargo.toml bumped to 0.5.0
- [ ] `brokkr-broker` and `brokkr-agent` Cargo.toml bumped to 0.5.0
- [ ] `sdks/python/brokkr-client/pyproject.toml` bumped to 0.5.0
- [ ] `sdks/typescript/brokkr-client/package.json` bumped to 0.5.0
- [ ] Helm chart version bumped to 0.5.0
- [ ] CHANGELOG entries added under each crate / SDK for 0.5.0 covering:
      WS channel (I-0019), telemetry retention (6h ceiling), new history
      endpoints, hardening (I-0020 deliverables)
- [ ] All `Cargo.lock` / `package-lock.json` / etc. regenerated and committed
- [ ] `angreal tests unit` + `angreal tests integration` + `angreal tests sdk-contract`
      all green after the bump
- [ ] **Do NOT** publish — the existing `release-sdks.yml` does that on tag

## Implementation Notes

### Technical Approach

- One commit per ecosystem (Rust / Python / TS / Helm) is easier to review
  than a single mega-commit
- CHANGELOG style should match existing project convention — check
  `CHANGELOG.md` in each crate before writing
- Cross-link to BROKKR-I-0019 and BROKKR-I-0020 in the CHANGELOG entries

### Dependencies

Blocked by A1–A4 — we shouldn't cut the version until the hardening tests
are green, since a bug found in A2 could force code changes that go in
the same release

### Risk Considerations

- Python and TS SDK contract tests pin a specific version; verify they
  still pass after the bump (they look up the installed dist, so the bump
  itself shouldn't break them — but worth confirming)
- The Rust SDK is a workspace crate; changing its version may cascade
  through `Cargo.lock`. Inspect the diff carefully

## Status Updates

### 2026-05-26 — Finding: releases are fully git-tag-driven; bump is cosmetic

Before bumping, discovered the release pipeline does not read the committed
version strings — the **git tag drives everything**:

- Containers (`release.yml`): image tags via `type=semver,pattern={{version}}`
  from the tag.
- Helm (`release.yml`): `helm package --version ${VERSION} --app-version
  ${VERSION}`, VERSION parsed from the tag — committed `Chart.yaml` is overwritten.
- SDKs (`release-sdks.yml`): each manifest regex-stamped from the tag at build.
  Its header even states *"Source manifests stay at 0.0.0; stamping happens in
  this workflow only"* — though they were actually committed at 0.4.2 (drift).

So the real "bump" is tagging `v0.5.0`. Editing manifests is cosmetic for the
release. Surfaced this to the human with options.

**Decisions (human-in-the-loop):**
1. *Bump everything to 0.5.0* — set all manifests + helm charts to 0.5.0 in
   source so the tree reads coherently, accepting that the tag overrides at
   publish. (Chose this over resetting SDK manifests to the 0.0.0 sentinel.)
2. *Skip CHANGELOGs* — no CHANGELOG convention exists in the repo; a
   version-bump task shouldn't invent one. The 0.5.0 release notes (WS channel
   I-0019, 6h telemetry retention, history endpoints, I-0020 hardening incl.
   the A3 pod-logs fix) go in the **git tag / GitHub release body** at tag time.

### 2026-05-26 — Bump applied

Bumped to 0.5.0:
- 6 workspace crates: brokkr-{broker,agent,utils,models,wire,client}
- 2 Python manifests: sdks/python/brokkr (wrapper) + sdks/python/brokkr-client
  (generated)
- TS: sdks/typescript/brokkr-client (via `npm version`, package-lock synced)
- Helm: charts/brokkr-{broker,agent} version + appVersion
- `Cargo.lock` regenerated (all six brokkr crates → 0.5.0)

`tests/e2e` and `tests/sdk-contract/rust` left at 0.0.0 (internal test
harnesses, never published). Helm `values.yaml` image tags untouched
(`latest` / unrelated subchart pins). Inter-crate deps are path-only (no
version pins) so nothing cascaded.

### 2026-05-26 — Test results

- `angreal tests unit all` → green (62/96/128/24/0 across crates)
- `angreal tests sdk-contract all` → green (Rust UAT passed; Python 6 passed;
  TypeScript 6 passed) — confirms the bump doesn't break the contract suites
  that look up the installed dist
- `angreal tests integration all` → green (38 + 437 = 475 passed, 0 failed)
  on a clean solo run.

**Process note:** a first integration run failed spuriously because I ran it
*concurrently* with `sdk-contract all` — both use docker-compose with the same
project name (`brokkr-dev`), so sdk-contract's teardown killed the Postgres the
integration run was mid-using ("connection refused" on :5433). Re-running
integration solo was fully green. Lesson: never run two docker-compose-based
angreal suites in parallel; they share the `brokkr-dev` project.

All B1 acceptance criteria met (version bump + lockfiles + all suites green;
CHANGELOG criterion intentionally dropped per the decision above; publish left
to the tag-driven `release-sdks.yml` / `release.yml`).
