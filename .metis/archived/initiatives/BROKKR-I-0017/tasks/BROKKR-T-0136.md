---
id: b3-prototype-python-client-with
level: task
title: "B3: Prototype Python client with openapi-python-client"
short_code: "BROKKR-T-0136"
created_at: 2026-05-14T18:26:23.555105+00:00
updated_at: 2026-05-14T23:08:43.177280+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0133]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# B3: Prototype Python client with openapi-python-client

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Scaffold `sdks/python`, generate a Python client from the hardened `openapi/brokkr-v1.json` using `openapi-python-client`, and confirm the generated package installs, imports, and produces a usable low-level client.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `sdks/python/brokkr-client/` contains the generated `brokkr_broker_client` package (16 API tag dirs, 69 models, httpx-based `Client`/`AuthenticatedClient`, typed `errors.py`).
- [x] `angreal openapi gen-python` regenerates from the committed spec via `uvx openapi-python-client@0.28.4`. Output is deterministic (round-trip diff is empty modulo `__pycache__`).
- [x] `uv pip install --python .venv -e .` succeeds; `import brokkr_broker_client` works; mypy passes cleanly on the package's core files (`client.py`, `errors.py`, `types.py`, `__init__.py`). Full-package mypy surfaces 36 errors — all from a naming collision between our domain `Generator` model and `typing.Generator`, documented as a finding for T-C2 (not a runtime issue).
- [~] Surface test (`tests/test_surface.py`, 4 tests, all green) exercises construction, sync+async availability, `ErrorResponse` shape, and verifies typed errors land in operation return unions. Real broker round-trip is owned by T-C2 (ergonomic wrapper) which needs a DB-backed broker fixture — same sequencing as T-B2.
- [x] Findings captured below for T-C2 consumption.

## Implementation Notes

### Technical Approach

1. Use `openapi-python-client generate --path openapi/brokkr-v1.json --output sdks/python` (or `--url` against a running broker).
2. Pin the generator version. Decide whether to commit the generated source or regenerate on demand. (Commit it — easier review, mirrors what we plan for Rust.)
3. Add an angreal task to regenerate; will be used by C3's CI drift check.
4. Verify `httpx`-based async client works; document sync vs async story.

### Dependencies

- Hard: [[BROKKR-T-0133]].
- Can run in parallel with [[BROKKR-T-0135]].

### Risk Considerations

- `openapi-python-client` has its own quirks (default value handling, optional vs nullable fields). Note any awkwardness for C2.
- Python packaging story: pinning the generator + committed output is the simplest model; revisit only if maintenance bites.

## Status Updates

### 2026-05-14 — Completed

**Files added:**

- `sdks/python/brokkr-client/` — generated package, 542 .py files. Generator: `openapi-python-client@0.28.4`, meta `uv`.
  - `brokkr_broker_client/` — package (api/, models/, client.py, errors.py, types.py).
  - `pyproject.toml` — uv build backend; deps: `httpx>=0.23,<0.29`, `attrs>=22.2`, `python-dateutil>=2.8,<3`. Python `>=3.10`.
  - `README.md` — generator's default.
- `sdks/python/brokkr-client/tests/test_surface.py` — 4 surface tests (construction, sync+async, ErrorResponse shape, typed error in return unions).
- `.angreal/task_openapi.py` — added `angreal openapi gen-python` task. Pinned generator version: `OPENAPI_PYTHON_CLIENT_VERSION = "0.28.4"`.

**Build & test:**

- `angreal openapi gen-python` — clean.
- Regeneration is deterministic (round-trip diff empty).
- `uv pip install --python .venv -e .` — clean.
- `pytest tests/` — 4/4 pass.
- mypy on core files (`client.py`, `errors.py`, `types.py`, `__init__.py`) — clean.
- mypy on full package — 36 errors, all from a domain-model naming collision (see findings).

**Generator quality findings (input for T-C2):**

1. **Both sync and async APIs generated for every operation.** Each `api/<tag>/<op>.py` exports `sync`, `sync_detailed`, `asyncio`, `asyncio_detailed`. The `*_detailed` variants return the full `Response[T]` envelope (status code + headers + parsed body); the bare ones return only the parsed body. T-C2 should pick one surface — recommend async-first with a thin sync facade — and re-export a clean API to hide the four-variants-per-op spread.

2. **Typed errors flow through return unions.** Example:
   ```
   def list_agents.sync(*, client) -> ErrorResponse | list[Agent] | None
   ```
   The generator folded the `ErrorResponse` schema into every operation's return type. Callers can `isinstance(result, ErrorResponse)` without re-parsing. This is exactly the contract the wrapper needs.

3. **Generator name collision.** Our domain model is `models.generator.Generator` (the Brokkr template generator concept). mypy gets confused in the 24 files that import this when parsing PEP 604 unions (`ErrorResponse | Generator | None`) — it occasionally binds `Generator` to `typing.Generator`. **Not a runtime issue** (Python imports resolve correctly, 4 surface tests pass), but a mypy hygiene cost. C2 options:
   - Re-export under a distinct name (`TemplateGenerator`) in the wrapper's public API.
   - Add `from __future__ import annotations` to suppress runtime evaluation — already in our spec output via the generator's preferences, so this may be a mypy bug specifically.
   - Live with it and document.

4. **`attrs`, not Pydantic.** Models derive from `attrs` with `kw_only=True`. Fields are `attrs.field()` with type hints. Validation is minimal (type-only). T-C2 may want pydantic-style validation for select inputs — would be a wrapper-layer concern, generated source remains pure.

5. **Two routes skipped at generation time** — POST `/stacks/{id}/labels` and POST `/templates/{id}/labels` accept a bare string body (`request_body = String` in utoipa). `openapi-python-client` doesn't support `text/plain` request bodies. The Rust client handles these fine; the Python SDK simply doesn't expose them. Carry-over: change the broker annotation to accept a typed body (e.g. `{"label": "..."}` wrapper) so both clients can use it. Not a blocker for prototype.

6. **Two clients: `Client` vs `AuthenticatedClient`.** Generated automatically because the spec declares security schemes. `AuthenticatedClient(base_url, token)` injects the `Authorization` header. The three PAK security schemes (admin/agent/generator) all map to the same header, so wrapping is straightforward — the wrapper exposes a single `token` parameter and lets the broker disambiguate at runtime (the A1 accepted limitation continues to be hidden).

7. **Generation speed.** ~3s on cold uvx (downloads generator), <1s on warm. Fast enough that the drift CI check is cheap.

**Decisions / tradeoffs:**

- Used `--meta uv` to align with this project's Python toolchain. Switching to `setup` (pip) or `poetry` is a one-line task change if downstream consumers need it.
- Committed the generated source. Drift check via diff is meaningful for review; alternative (regenerate-in-CI-and-import) hides API surface changes.
- Pinned generator version `0.28.4` in the angreal task constant. Bump deliberately.

**Carry-overs:**

- T-C2 (Python wrapper) needs: token injection (one-arg constructor), retry/backoff, sync facade decision, optional rename of `Generator` to avoid mypy noise, and a real-broker smoke test.
- T-C3 (regen drift CI) — the Python diff check is meaningful (committed source). Already-deterministic generator means it'll be a clean signal.
- Broker carry-over: change `POST /stacks/:id/labels` and `POST /templates/:id/labels` to accept a JSON body so they can appear in the Python SDK. Small spec/handler change. Filed informally here — should be tracked separately if it doesn't get picked up alongside T-C2.