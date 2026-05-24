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
  - "#phase/todo"


exit_criteria_met: false
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

*To be added during implementation*
