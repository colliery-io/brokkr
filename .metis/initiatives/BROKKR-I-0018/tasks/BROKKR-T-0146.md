---
id: r2-b-python-rename
level: task
title: "R2-B: Rename Python SDK distributions to brokkr-client / brokkr-client-generated"
short_code: "BROKKR-T-0146"
created_at: 2026-05-15T22:30:00.000000+00:00
updated_at: 2026-05-15T22:30:00.000000+00:00
parent: BROKKR-I-0018
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0018
---

# R2-B: Rename Python SDK distributions to brokkr-client / brokkr-client-generated

## Parent Initiative

[[BROKKR-I-0018]]

## Objective

Bring the two Python packages' **distribution names** in line with what's reserved on PyPI: wrapper goes `brokkr` → `brokkr-client`, generated low-level goes `brokkr-broker-client` → `brokkr-client-generated`. Import names stay the same to avoid a churning refactor for zero ergonomic gain.

## Acceptance Criteria

- [ ] `sdks/python/brokkr/pyproject.toml` has `[project] name = "brokkr-client"`.
- [ ] `sdks/python/brokkr-client/pyproject.toml` has `[project] name = "brokkr-client-generated"`.
- [ ] `sdks/python/brokkr/pyproject.toml`'s `dependencies` and `tool.uv.sources` reference `brokkr-client-generated`, not `brokkr-broker-client`.
- [ ] Import names unchanged: `from brokkr import BrokkrClient` and `from brokkr_broker_client import ...` continue to work.
- [ ] `angreal openapi gen-python` still produces a build that satisfies the new dependency name (the generator emits a name we can override via its config).
- [ ] `uv pip install -e sdks/python/brokkr` succeeds against the renamed packages in a clean venv.
- [ ] `angreal openapi check-python` passes against the renamed sources.
- [ ] `docs/src/how-to/sdks/python.md` install command updated (`uv pip install brokkr-client` instead of `brokkr`).
- [ ] Any other docs / example references to the old distribution names swept (`grep -r brokkr-broker-client`, `grep -rE '"brokkr"' sdks/`).

## Implementation Notes

### Technical Approach

1. Rename in `pyproject.toml`s first. Keep the directory layout (`sdks/python/brokkr/` and `sdks/python/brokkr-client/`) — directory names are independent of distribution names and renaming them is unnecessary churn.
2. Update `tool.uv.sources` in the wrapper's `pyproject.toml` to point at the renamed generated package by its new distribution name.
3. `openapi-python-client` config (likely a `pyproject.toml` block or CLI flag in the angreal task) sets the package's distribution name on regeneration — update it so future `gen-python` runs produce the right name.
4. Sweep docs. The two install-line locations are `docs/src/how-to/sdks/python.md` and the SDK READMEs in `sdks/python/*/README.md`.

### Dependencies

- None (can run in parallel with R2-A).

### Risk Considerations

- The generator-emitted package name has to be settable. If `openapi-python-client` insists on a name derived from the OpenAPI `info.title`, we may need a post-generation `sed` step or a config file. Verify before assuming a flag exists.
- Import name `brokkr_broker_client` stays. This is awkward (mismatched distribution and import names) but the alternative — renaming every `from brokkr_broker_client...` site — is a larger refactor for purely cosmetic improvement.

## Status Updates

*To be added during implementation*
