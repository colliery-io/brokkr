---
id: c3-generated-sdk-regeneration-ci
level: task
title: "C3: Generated SDK regeneration CI check"
short_code: "BROKKR-T-0139"
created_at: 2026-05-14T18:26:27.006830+00:00
updated_at: 2026-05-15T01:44:05.828389+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0135, BROKKR-T-0136]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# C3: Generated SDK regeneration CI check

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Apply the same drift-detection pattern from B1 (spec drift) to the generated SDK source: regenerate Rust + Python clients in CI, diff against committed output, fail the build on any unexpected drift. Without this, the SDKs decouple from the spec the moment someone forgets to regenerate.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] **Rust SDK drift.** Progenitor regenerates the client inline at every `cargo build` via the proc macro — no committed generated source to diff. The drift check therefore is "does the crate still build against the committed spec." Wired as `cargo build -p brokkr-client --tests` in the CI workflow. If the spec drifts in a way the generator can't handle, this step fails with a real type-checker error message.
- [x] **Python SDK drift.** New `angreal openapi check-python` regenerates into a temp dir using the pinned `openapi-python-client@0.28.4` and walks the candidate tree against `sdks/python/brokkr-client/`. One-sided comparison: every generator-produced file must exist identically under the committed tree. Extras under the committed tree (hand-added tests, dev caches) are ignored, as are `__pycache__`, `*.pyc`, and `.ruff_cache`. Verified by injecting drift into `client.py` (correctly detected: `differs: brokkr_broker_client/client.py`) and restoring it (clean again).
- [x] **Failure messages.** Drift output names every changed file plus `Regenerate locally with: angreal openapi gen-python`. Rust-side failures surface as standard `cargo build` errors and are self-explanatory.
- [x] **Triggers.** Workflow `paths:` filter expanded to include `crates/brokkr-client/**` and `sdks/python/brokkr-client/**` alongside the existing spec/broker triggers.
- [x] **Angreal task.** Two new commands: `angreal openapi check-python` (drift check) and the existing `angreal openapi gen-python` (regenerate). Both discoverable via `angreal tree`.

## Implementation Notes

### Technical Approach

1. Likely a single workflow file or extension of B1's. Run the generators against the committed spec, diff, fail on drift.
2. If `progenitor` is invoked via `build.rs` macro (no committed generated file), then drift detection collapses to "the crate builds" — sufficient. Adjust criteria accordingly.
3. For Python, the generated source is checked in, so it's a real diff check.

### Dependencies

- Hard: [[BROKKR-T-0135]], [[BROKKR-T-0136]] (need the generators wired before the check exists).

### Risk Considerations

- Generator non-determinism (timestamps, file ordering). Normalize or pin tool versions tightly.

## Status Updates

### 2026-05-15 — Completed

**Files changed:**

- `.angreal/task_openapi.py`:
  - New `_run_python_gen(target_dir)` helper shared by `gen-python` and `check-python`. Pinned generator version `OPENAPI_PYTHON_CLIENT_VERSION = "0.28.4"`.
  - New `_generated_drifts(fresh, committed)` walker: enumerates every file the generator produced, checks each exists byte-identical under the committed tree. Ignores `__pycache__`, `*.pyc`, `.ruff_cache`. One-sided so hand-added test files and dev caches don't trigger.
  - New `angreal openapi check-python` task. Regenerates into a tempdir, runs the walker, prints up to 50 drifted entries plus the remediation command.
- `.github/workflows/openapi.yml`:
  - Expanded path triggers to include `crates/brokkr-client/**` and `sdks/python/brokkr-client/**`.
  - Added uv to the python deps install step (needed by `uvx`).
  - Added "Verify Rust SDK regenerates from spec" step running `cargo build -p brokkr-client --tests`.
  - Added "Check Python SDK drift" step running `angreal openapi check-python`.
  - Step ordering: spec drift → spec lint → Rust SDK build → Python SDK drift. Each step's failure is self-explanatory.

**Verification:**

- `angreal tree` shows `check-python` + `gen-python`.
- Clean tree: `OK: sdks/python/brokkr-client matches the spec`.
- Drift injection (`echo "# tampered" >> brokkr_broker_client/client.py`): `FAIL: sdks/python/brokkr-client is stale (1 drifted entries). differs: brokkr_broker_client/client.py`.
- Restore: clean again.
- Workflow YAML parses (`js-yaml`).
- `cargo build -p brokkr-client --tests` clean (Rust drift check passes today).

**Decisions:**

- **One-sided diff for Python.** The committed tree intentionally contains files the generator doesn't produce (hand-written `tests/test_surface.py`, possibly more in the future). A symmetric diff would require maintaining an ignore list per file; the one-sided "every generated file must exist identically under committed" is robust and self-maintaining.
- **No separate Rust diff step.** The proc macro regenerates the client every build, so build success *is* drift detection. Adding a separate "diff" step would duplicate work and confuse failure modes. The CI step's name makes the contract obvious: "Verify Rust SDK regenerates from spec".
- **Generator version pinned in code, not CI.** `OPENAPI_PYTHON_CLIENT_VERSION = "0.28.4"` lives in the angreal task so local and CI runs are guaranteed to match. Bumping is a deliberate code change with a reviewable diff.
- **Skip set is conservative.** Currently `__pycache__`, `*.pyc`, `.ruff_cache`. If the generator starts emitting other transient artifacts, the skip set is the right place to extend.

**Carry-overs:**

- The Python drift check requires `uvx` (uv tool runner). Workflow installs `uv` via pip. If we ever drop uv from the project's toolchain, swap to `pipx run openapi-python-client@<version>` — the rest of the angreal logic is unchanged.
- **T-D1 (agent migration)** and **T-D2 (docs)** are the last two tasks in the initiative; both unblocked.