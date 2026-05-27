---
id: r2-e-cut-v0-3-0-first-real-sdk
level: task
title: "R2-E: Cut v0.3.0 — first real SDK release"
short_code: "BROKKR-T-0149"
created_at: 2026-05-15T22:30:00+00:00
updated_at: 2026-05-16T03:48:28.456361+00:00
parent: BROKKR-I-0018
blocked_by: [BROKKR-T-0148]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-E: Cut v0.3.0 — first real SDK release

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Push `v0.3.0`, watch both release workflows (containers via `release.yml`, SDKs via `release-sdks.yml`) run, approve them, and verify every published artifact is installable and usable from a clean machine. This is the smoke test for the whole initiative.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Tags pushed: `v0.3.0`, `v0.3.1`, `v0.3.2`. First full clean run on v0.3.2 after iterating through pending-publisher and npm-2FA blockers.
- [x] `release-sdks.yml` green on v0.3.2: all four publishes (cargo, both PyPI packages, npm) succeeded.
- [x] `release.yml` (containers + helm) — runs in parallel on each tag; tracked separately.
- [x] Registry verification (HTTP API): all four packages report `0.3.2` as their latest published version.
- [ ] **Validation on a clean machine** — deferred to a follow-up. Cheap to spot-check before announcing externally.
- [ ] Release runbook documenting what to do for partial failures — captured in this task's Status Updates for now; will lift into `docs/release-workflow.md` as part of R2-F or follow-up.

## Implementation Notes

### Technical Approach

1. Pick the tag commit. Should be the head of `develop` post-R2-D merge, fully CI-green.
2. Push tag, approve both environment gates.
3. Run the install validation on a clean Docker container (`docker run --rm -it python:3.12 bash`, etc.). Don't trust your laptop — caches lie.
4. Note in the runbook: cargo's first publish locks the crate name to whatever identity owns the token. If something goes sideways here, fix it now rather than discovering it on v0.3.1.
5. If any single publish fails after others succeed, do **not** delete the tag and retry — version numbers are permanent on every registry. Cut v0.3.1 instead, fix the broken step, and document what went wrong.

### Dependencies

- Hard: [[BROKKR-T-0148]].

### Risk Considerations

- This is the highest-blast-radius task in the initiative. Treat it as a planned activity, not a side quest. Schedule it for a time when the approvers are reachable.
- Publishing identity is forever. Double-check `CARGO_REGISTRY_TOKEN` and the PyPI Trusted Publisher owner are what we want on the public registry pages, *before* hitting approve.

## Status Updates

- 2026-05-16: Tagged + pushed v0.3.0 and v0.3.1 (the latter to recover from a partial failure in v0.3.0). Current registry state:

  | Registry | Package | Latest published | Notes |
  |----------|---------|------------------|-------|
  | crates.io | `brokkr-client` | 0.3.1 | ✅ first publish on v0.3.0 succeeded; 0.3.1 followed |
  | PyPI | `brokkr-client` | 0.3.1 | ✅ |
  | PyPI | `brokkr-client-generated` | 0.3.1 | ✅ (0.3.0 attempt failed — pending publisher record was missing; user added it before v0.3.1) |
  | npm | `@colliery-io/brokkr-client` | — | ❌ blocked: `403 Two-factor authentication or granular access token with bypass 2fa enabled is required to publish packages` |

  **Outstanding blocker**: npm token lacks the "bypass 2FA" / CI flag. User will regenerate the token tomorrow (granular access token with `Allow this token to bypass 2FA` enabled, scoped to `@colliery-io` org). Once `NPM_TOKEN` repo secret is updated, cut `v0.3.2` to retry. cargo + PyPI publishes are idempotent (PyPI uses `skip-existing: true`, cargo publishes a fresh 0.3.2) so no side effects.

  **CI side-fixes landed during the release**:
  - `.github/workflows/release-sdks.yml` partial-failure handling: each publish step is now `continue-on-error: true` with explicit step ids; a final `Summarize publish results` step writes a markdown table to `$GITHUB_STEP_SUMMARY` and fails the job if any step's `outcome == failure`. (User confirmed keeping strict failure semantics.)
  - Cargo version stamping switched from `cargo set-version` (cargo-edit) to a Python regex, sidestepping cargo-edit's version-ordering check that rejected pre-release dispatch defaults.
  - Workflow_dispatch default version: `0.0.0-test` → `99.99.99` (valid under both semver and PEP 440).

  **Validation on clean machines still pending**: not yet run for cargo / PyPI on v0.3.1; can't run for npm until the token is fixed. Hold until npm lands so we validate everything in one pass.