---
id: b1-spec-drift-ci-check-and-lint
level: task
title: "B1: Spec drift CI check and lint"
short_code: "BROKKR-T-0134"
created_at: 2026-05-14T18:26:20.734575+00:00
updated_at: 2026-05-14T21:41:55.582572+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0133]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# B1: Spec drift CI check and lint

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Wire the `openapi_export` example into CI as a required check: regenerate `openapi/brokkr-v1.json`, diff against the committed copy, fail the PR if they don't match. Additionally, run `redocly lint` (or equivalent) on the committed spec so structural regressions never reach main.

Without this, the spec hardening from A1–A3 will rot the moment someone adds a route without updating the committed JSON.

## Acceptance Criteria

## Acceptance Criteria

- [x] CI job runs the export and fails on drift. Implemented as `angreal openapi check` in `.github/workflows/openapi.yml`. On mismatch, prints `FAIL: openapi/brokkr-v1.json is stale.` followed by a unified diff and the remediation command.
- [x] CI runs `npx @redocly/cli@1.25.11 lint openapi/brokkr-v1.json` after the drift check. Currently passes with 7 unused-component warnings only.
- [x] Workflow triggers on PRs that touch `crates/brokkr-broker/src/api/**`, `crates/brokkr-broker/examples/openapi_export.rs`, `crates/brokkr-models/src/models/**`, `openapi/**`, `.angreal/task_openapi.py`, or `.github/workflows/openapi.yml`.
- [x] Failure messages include the exact local fix command (`angreal openapi export`).
- [x] Angreal task added: `.angreal/task_openapi.py` exposes `angreal openapi export` and `angreal openapi check`, discoverable via `angreal tree`.

## Implementation Notes

### Technical Approach

1. Add a GH Actions job (or extend an existing one) that:
   - Sets up Rust toolchain (matching CI's existing setup).
   - Runs the export example with `--release` (or `--locked`) for stability.
   - Pipes the generated JSON through a normalizer if needed (e.g. consistent key ordering) before `diff`.
   - On diff, prints a clear remediation hint.
2. Add `redocly lint` (npx or pinned npm install) as a parallel step. Configure a `redocly.yaml` with project conventions if needed — keep it minimal at first.
3. Update the angreal task list with a `docs openapi` (or similarly named) command that mirrors the CI export, for developer ergonomics.

### Dependencies

- Hard: [[BROKKR-T-0133]] — drift detection only makes sense once the spec is clean.

### Risk Considerations

- JSON key ordering: `utoipa`'s output may not be deterministic across runs. If diffs are noisy, sort keys via `jq -S` or a small canonicalization step before comparing.
- `redocly lint` may flag warnings worth ignoring on first pass — calibrate severity thresholds in `redocly.yaml`.

## Status Updates

### 2026-05-14 — Completed

**Files added:**

- `.angreal/task_openapi.py` — `openapi` command group with two tasks:
  - `angreal openapi export` — runs the cargo example, writes `openapi/brokkr-v1.json`.
  - `angreal openapi check` — regenerates into a tempfile (with backup-restore of the committed copy to avoid clobbering), compares byte-for-byte, prints a `diff -u` plus remediation command on mismatch.
- `.github/workflows/openapi.yml` — `OpenAPI Spec` workflow with one job (`drift_and_lint`) that runs `angreal openapi check` then `redocly lint`. Triggers on push to main/develop and on PRs touching the relevant paths.
- `redocly.yaml` — extends the `minimal` ruleset; disables doc-quality recommendations (`operation-summary`, `tag-description`, `info-license`, `info-contact`, `no-path-trailing-slash`) and downgrades `no-unused-components` to a warning. Structural correctness rules stay at error level. Rationale documented in the file.

**Verification:**

- `angreal tree` shows the new group.
- `angreal openapi check` on a clean tree: `OK: openapi/brokkr-v1.json is up to date`.
- `npx @redocly/cli@1.25.11 lint openapi/brokkr-v1.json`: `Your API description is valid. 🎉 You have 7 warnings.` (all `no-unused-components`).
- Workflow YAML parses cleanly (`js-yaml`).

**Decisions / notes:**

- Used `angreal openapi` rather than `angreal docs openapi` because the spec contract is independent of the docs build. Keeps the drift check's failure path obvious.
- Did not normalize JSON key ordering — utoipa's output is deterministic in practice, and any diff is the signal we want. If non-determinism shows up in CI we can pipe through `jq -S` later.
- Redocly version pinned (`@redocly/cli@1.25.11`). Bumping is a deliberate maintenance action — drift in lint rules across versions shouldn't silently change CI behavior.
- `redocly.yaml` opts to silence operation-summary errors rather than adding summaries to 85 operations. Adding summaries is real docs work owned by T-D2.

**Carry-overs:**

- The 7 unused-component warnings (`NewStackTemplate`, `NewTemplateAnnotation`, `NewTemplateLabel`, `NewStackLabel` in some paths, `WebhookSubscription`, plus a few `Vec<...>` envelope types) are leftovers from earlier schema registration that no operation references directly. Worth a sweep when convenient; not a blocker.