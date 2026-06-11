---
id: python-sdk-submit-manifests-apply
level: task
title: "Python SDK: submit_manifests/apply folder helper"
short_code: "BROKKR-T-0196"
created_at: 2026-06-11T02:19:32.186237+00:00
updated_at: 2026-06-11T02:19:32.186237+00:00
parent: BROKKR-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0021
---

# Python SDK: submit_manifests/apply folder helper

## Parent Initiative

[[BROKKR-I-0021]]

## Objective

Mirror the Rust folder helpers in the Python SDK ergonomic wrapper (`sdks/python/brokkr/brokkr/client.py`) — `submit_manifests`/`apply` so a Python control plane can hand Brokkr a folder of manifests in one idempotent call.

## Design

- `submit_manifests(stack_id, path_or_paths)` — accept a dir / file / globs; read `*.yaml`/`*.yml`; concatenate with `---`; validate each doc parses (PyYAML safe_load_all) and has `apiVersion`+`kind`; POST the stream (raw-YAML body when available).
- `apply(stack_name, path, targeting=None)` — idempotent create-or-reuse stack by name + submit-on-change + set targeting; return a result indicating created/updated/unchanged.
- Async to match the wrapper's existing style; reuse the `retry`/error model.

## Acceptance Criteria

- [ ] `submit_manifests`/`apply` on `BrokkrClient`, reading a directory of YAML files
- [ ] Idempotent `apply` (unchanged folder → no new revision)
- [ ] Per-doc validation with clear `BrokkrError`s
- [ ] pytest coverage (folder fixture; idempotency)
- [ ] Python SDK how-to updated

## Status Updates

- 2026-06-11: Created under BROKKR-I-0021. Parallel with T-0195/T-0197.
