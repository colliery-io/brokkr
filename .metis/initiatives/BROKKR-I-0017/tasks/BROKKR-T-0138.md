---
id: c2-python-sdk-ergonomic-wrapper
level: task
title: "C2: Python SDK ergonomic wrapper"
short_code: "BROKKR-T-0138"
created_at: 2026-05-14T18:26:25.670040+00:00
updated_at: 2026-05-15T01:04:08.252437+00:00
parent: BROKKR-I-0017
blocked_by: [BROKKR-T-0136, BROKKR-T-0137]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0017
---

# C2: Python SDK ergonomic wrapper

## Parent Initiative

[[BROKKR-I-0017]]

## Objective

Mirror C1's ergonomic surface for the Python SDK. Where Python idiom diverges from Rust, follow Python — but the shape (one credential, retry policy, typed errors, pagination iterators) should be recognizably parallel for developers crossing between the two.

C1 lands first so its API decisions inform this one — duplicate ergonomic choices across languages without re-litigating them.

## Acceptance Criteria

- [x] `BrokkrClient` shipped in a new `brokkr` package at `sdks/python/brokkr/`, depending on the generated `brokkr-broker-client` via a uv path source. Separate package isolates the wrapper from `angreal openapi gen-python` overwrites.
- [x] Single credential: `BrokkrClient(base_url, token="bk_...")`. With a token → generated `AuthenticatedClient`; without → unauthenticated `Client`. Three security schemes collapse to one wrapper parameter.
- [x] Retry/backoff: `await client.retry(op)` with exponential backoff (initial * 2^attempt, capped at 10s). Retryable set: transport errors + 408/429/502/503/504 (identical to C1). Configurable `max_retries` (default 3) and `initial_backoff` (default 0.2s). Opt-in per call.
- [x] Typed errors: `BrokkrError` dataclass-exception with `.code`, `.status`, `.response`, plus `is_retryable()`. `from_response` / `from_transport` constructors. `ErrorResponse` re-exported from `brokkr`.
- [~] Pagination — same no-op as C1. v1 endpoints return full collections.
- [x] Async-first wrapper. Generated client provides both sync + asyncio per op; `retry()` is async. Sync consumers use `client.api.<op>.sync(...)` or `asyncio.run(...)`.
- [x] **24 unit tests pass**, mypy clean. Real-broker smoke test deferred to T-D1.

## Implementation Notes

### Technical Approach

1. Decide package distribution: in-repo only (per initiative non-goal of public PyPI publishing).
2. Re-export generated models alongside the wrapper class for one-stop imports.
3. Use `pydantic` (if the generated client already uses it) for the `ErrorResponse` model to keep validation consistent.

### Dependencies

- Hard: [[BROKKR-T-0136]] (generated client to wrap).
- Hard: [[BROKKR-T-0137]] (C1's API decisions are the design template).

### Risk Considerations

- Sync vs async: if both surfaces, the wrapper doubles in size. Async-first with a sync facade is the standard answer.

## Status Updates

### 2026-05-15 — Completed

**Files added:**

- `sdks/python/brokkr/pyproject.toml` — wrapper package. uv build backend, runtime deps `brokkr-broker-client` (path source) + `httpx`. `[test]` extra: pytest + pytest-asyncio + respx.
- `sdks/python/brokkr/README.md` — short usage doc.
- `sdks/python/brokkr/brokkr/__init__.py` — re-exports `BrokkrClient`, `BrokkrError`, `ErrorResponse`, `TemplateGenerator` (alias for the generated `Generator` to side-step the mypy collision flagged in T-B3).
- `sdks/python/brokkr/brokkr/errors.py` (62 LOC) — `BrokkrError` dataclass-exception with `from_response` / `from_transport` constructors.
- `sdks/python/brokkr/brokkr/client.py` (109 LOC) — `BrokkrClient` with httpx timeouts and async `retry()` helper.
- `sdks/python/brokkr/tests/test_wrapper.py` — 24 pytest tests.

**Total wrapper LOC: 187.**

**Design parity with C1:**

| concern | C1 (Rust) | C2 (Python) |
|---|---|---|
| construction | `BrokkrClient::builder(url).token(pak).build()?` | `BrokkrClient(url, token="...")` |
| api access | `client.api()` | `client.api` |
| retry helper | `client.retry(\|c\| async {...})` | `await client.retry(op)` |
| retryable status | `{408, 429, 502, 503, 504}` | identical |
| backoff | `initial * 2^attempt`, cap 10s | identical |
| max retries default | 3 | 3 |
| typed error | `BrokkrError::Api(ErrorResponse, status)` | `BrokkrError(message, code, status, response)` |
| pagination | not implemented | not implemented |

**Verification:**

- `pytest tests/` → 24/24 pass.
- `mypy brokkr` → clean (3 files).

**Decisions / tradeoffs:**

- **Separate `brokkr` package** rather than co-locating with the generated source. `--overwrite` regen would clobber hand-written code otherwise.
- **`Generator → TemplateGenerator` re-export** — addresses the T-B3 mypy noise without touching generated source. Original still reachable.
- **Async-only retry helper.** Wrapping both sync + asyncio doubles surface area for marginal value. Sync consumers fall back to the generated `client.api.<op>.sync(...)` or `asyncio.run(...)`.
- **`ErrorResponse` returns vs raises split.** `retry()` converts return-union `ErrorResponse` to a raise; direct `client.api` calls preserve the generator's return-union typing.
- **No tenacity dep.** Hand-rolled retry mirrors C1.

**Carry-overs:**

- **`status=500` placeholder for return-union errors.** The generator's `asyncio()` variant strips status code; the wrapper treats return-union `ErrorResponse` as 500 for retryability. Status-aware callers should use `asyncio_detailed()` and unwrap. Documented inline.
- **T-C3 (regen drift CI)** — Python diff check is meaningful (committed source) and generator is deterministic per T-B3.
- **T-D2 (docs)** — lift getting-started examples from C1 + C2 READMEs.
- **T-D1 (agent migration)** — fully unblocked; both wrappers ready.