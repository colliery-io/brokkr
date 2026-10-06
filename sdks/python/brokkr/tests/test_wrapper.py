"""Unit tests for the BrokkrClient wrapper."""

from __future__ import annotations

import asyncio
from types import SimpleNamespace
from uuid import UUID

import httpx
import pytest

from brokkr import BrokkrClient, BrokkrError, ErrorResponse, TemplateGenerator
from brokkr.client import normalize_base_url
from brokkr_broker_client import AuthenticatedClient, Client
from brokkr_broker_client import models as generated_models


def _resp(status: int, parsed: object) -> SimpleNamespace:
    """Stand in for a generated ``*_detailed`` Response (``.status_code`` +
    ``.parsed``), which is what ``retry``'s ``op`` must now return."""
    return SimpleNamespace(status_code=status, parsed=parsed)


def test_constructs_authenticated_when_token_supplied() -> None:
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_admin_test")
    assert isinstance(c.api, AuthenticatedClient)


def test_constructs_unauthenticated_when_token_omitted() -> None:
    c = BrokkrClient("http://localhost:3000/api/v1")
    assert isinstance(c.api, Client)


@pytest.mark.parametrize(
    "given",
    [
        "http://localhost:3000",
        "http://localhost:3000/",
        "http://localhost:3000/api/v1",
        "http://localhost:3000/api/v1/",
        "  http://localhost:3000  ",
    ],
)
def test_base_url_gets_api_prefix_once(given: str) -> None:
    assert normalize_base_url(given) == "http://localhost:3000/api/v1"
    c = BrokkrClient(given, token="bk_admin_test")
    assert c.api._base_url == "http://localhost:3000/api/v1"


def test_rejects_invalid_max_retries() -> None:
    with pytest.raises(ValueError):
        BrokkrClient("http://localhost", max_retries=-1)


def test_rejects_invalid_initial_backoff() -> None:
    with pytest.raises(ValueError):
        BrokkrClient("http://localhost", initial_backoff=0)


def test_error_code_and_status_round_trip() -> None:
    body = ErrorResponse(code="agent_not_found", message="agent not found")
    err = BrokkrError.from_response(body, status=404)
    assert err.code == "agent_not_found"
    assert err.status == 404
    assert err.response is body
    assert not err.is_retryable()


@pytest.mark.parametrize("status", [408, 429, 502, 503, 504])
def test_retryable_classification_retryable(status: int) -> None:
    err = BrokkrError(message="x", code="transient", status=status)
    assert err.is_retryable()


@pytest.mark.parametrize("status", [400, 401, 403, 404, 409, 422, 500, 501])
def test_retryable_classification_non_retryable(status: int) -> None:
    err = BrokkrError(message="x", code="non_transient", status=status)
    assert not err.is_retryable()


def test_transport_error_default_retryable() -> None:
    err = BrokkrError.from_transport(httpx.ConnectError("connection refused"))
    assert err.is_retryable()
    assert err.status is None


async def test_retry_returns_on_first_success() -> None:
    c = BrokkrClient("http://localhost", max_retries=5, initial_backoff=0.001)
    calls = 0

    async def op(_api: object) -> object:
        nonlocal calls
        calls += 1
        return _resp(200, "ok")

    result = await c.retry(op)
    assert result == "ok"
    assert calls == 1


async def test_retry_retries_retryable_status_then_succeeds() -> None:
    c = BrokkrClient("http://localhost", max_retries=5, initial_backoff=0.001)
    calls = 0

    async def op(_api: object) -> object:
        nonlocal calls
        calls += 1
        # 503 twice (retryable, parsed is None for an undocumented status),
        # then a 200 — the old code returned None on the first 503.
        return _resp(503, None) if calls < 3 else _resp(200, "ok")

    result = await c.retry(op)
    assert result == "ok"
    assert calls == 3


async def test_retry_raises_with_real_status_not_fabricated() -> None:
    c = BrokkrClient("http://localhost", max_retries=5, initial_backoff=0.001)
    calls = 0

    async def op(_api: object) -> object:
        nonlocal calls
        calls += 1
        return _resp(
            404, ErrorResponse(code="agent_not_found", message="agent not found")
        )

    with pytest.raises(BrokkrError) as exc_info:
        await c.retry(op)
    # Real wire status, not the old hardcoded 400/500.
    assert exc_info.value.status == 404
    assert exc_info.value.code == "agent_not_found"
    assert calls == 1


async def test_retry_stops_after_max_attempts_on_transport_error() -> None:
    c = BrokkrClient("http://localhost", max_retries=2, initial_backoff=0.001)
    calls = 0

    async def op(_api: object) -> str:
        nonlocal calls
        calls += 1
        raise httpx.ConnectError("nope")

    with pytest.raises(BrokkrError):
        await c.retry(op)
    # Initial attempt + 2 retries = 3 calls.
    assert calls == 3


async def test_retry_short_circuits_on_non_retryable_status() -> None:
    c = BrokkrClient("http://localhost", max_retries=5, initial_backoff=0.001)
    calls = 0

    async def op(_api: object) -> object:
        nonlocal calls
        calls += 1
        # A 404 detailed Response — non-retryable, so raise on first attempt.
        return _resp(
            404, ErrorResponse(code="agent_not_found", message="agent not found")
        )

    with pytest.raises(BrokkrError) as exc_info:
        await c.retry(op)
    assert exc_info.value.code == "agent_not_found"
    assert calls == 1


async def test_retry_backoff_doubles(monkeypatch: pytest.MonkeyPatch) -> None:
    sleeps: list[float] = []

    async def fake_sleep(seconds: float) -> None:
        sleeps.append(seconds)

    monkeypatch.setattr(asyncio, "sleep", fake_sleep)

    c = BrokkrClient("http://localhost", max_retries=4, initial_backoff=0.1)
    calls = 0

    async def op(_api: object) -> str:
        nonlocal calls
        calls += 1
        raise httpx.ConnectError("nope")

    with pytest.raises(BrokkrError):
        await c.retry(op)

    assert calls == 5  # initial + 4 retries
    # Backoffs are scheduled between attempts: 0.1, 0.2, 0.4, 0.8.
    assert sleeps == pytest.approx([0.1, 0.2, 0.4, 0.8])


def test_template_generator_reexport_resolves_to_generated_type() -> None:
    # The wrapper re-exports the generated `Generator` model under a less
    # confusable name. Both should point at the same class.
    assert TemplateGenerator is generated_models.Generator


# --- BROKKR-T-0196: manifest folder helpers ---

from pathlib import Path

from brokkr.client import _read_manifests, _sha256_hex


def _write(d: Path, name: str, content: str) -> None:
    (d / name).write_text(content)


def test_read_manifests_concatenates_folder_sorted(tmp_path: Path) -> None:
    _write(tmp_path, "02-cm.yaml", "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: c\n")
    _write(tmp_path, "01-ns.yaml", "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: n\n")
    _write(tmp_path, "notes.txt", "ignored")
    stream = _read_manifests(tmp_path)
    assert stream.index("kind: Namespace") < stream.index("kind: ConfigMap")
    assert "\n---\n" in stream
    assert "ignored" not in stream


def test_read_manifests_single_file_multidoc(tmp_path: Path) -> None:
    _write(
        tmp_path,
        "all.yaml",
        "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: a\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: b\n",
    )
    stream = _read_manifests(tmp_path / "all.yaml")
    assert "kind: Namespace" in stream and "kind: ConfigMap" in stream


def test_read_manifests_rejects_missing_apiversion_or_kind(tmp_path: Path) -> None:
    _write(tmp_path, "bad.yaml", "kind: ConfigMap\nmetadata:\n  name: x\n")
    with pytest.raises(BrokkrError):
        _read_manifests(tmp_path)


def test_read_manifests_rejects_malformed_yaml(tmp_path: Path) -> None:
    _write(tmp_path, "bad.yaml", "kind: : : [unbalanced")
    with pytest.raises(BrokkrError):
        _read_manifests(tmp_path)


def test_read_manifests_errors_on_empty_and_missing(tmp_path: Path) -> None:
    with pytest.raises(BrokkrError):
        _read_manifests(tmp_path)  # empty dir
    with pytest.raises(BrokkrError):
        _read_manifests(tmp_path / "nope")  # missing path


def test_sha256_hex_matches_known_vector() -> None:
    assert _sha256_hex("") == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    a = "apiVersion: v1\nkind: ConfigMap\n"
    assert _sha256_hex(a) == _sha256_hex(a)
    assert _sha256_hex(a) != _sha256_hex("apiVersion: v1\nkind: Secret\n")


# ---------------------------------------------------------------------------
# apply: who owns the stack (BROKKR-T-0332)
# ---------------------------------------------------------------------------

GENERATOR_ID = UUID("22222222-2222-2222-2222-222222222222")
STACK_ID = UUID("33333333-3333-3333-3333-333333333333")


def _manifests(tmp_path: Path) -> Path:
    _write(tmp_path, "cm.yaml", "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: c\n")
    return tmp_path


def _patch(monkeypatch: pytest.MonkeyPatch, target: str, result: object) -> list:
    """Replace a generated ``asyncio_detailed`` with one that records its call
    and returns ``result`` (a ``_resp``)."""
    calls: list = []

    async def fake(*args: object, **kwargs: object) -> object:
        calls.append((args, kwargs))
        return result

    monkeypatch.setattr(f"brokkr_broker_client.api.{target}.asyncio_detailed", fake)
    return calls


async def test_apply_admin_names_the_generator_and_owns_the_stack(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _patch(monkeypatch, "auth.verify_pak", _resp(200, SimpleNamespace(admin=True, generator=None)))
    _patch(
        monkeypatch,
        "generators.list_generators",
        _resp(200, [SimpleNamespace(id=GENERATOR_ID, name="acme")]),
    )
    _patch(monkeypatch, "stacks.list_stacks", _resp(200, []))
    created = _patch(
        monkeypatch,
        "stacks.create_stack",
        _resp(201, SimpleNamespace(id=STACK_ID, name="payments", generator_id=GENERATOR_ID)),
    )
    _patch(monkeypatch, "stacks.list_deployment_objects", _resp(200, []))
    _patch(
        monkeypatch,
        "stacks.create_deployment_object",
        _resp(201, SimpleNamespace(id="o1", stack_id=STACK_ID, sequence_id=1)),
    )
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_admin")
    result = await c.apply("payments", _manifests(tmp_path), generator="acme")
    assert result.status == "created"
    assert created[0][1]["body"].generator_id == GENERATOR_ID


async def test_apply_admin_without_generator_names_the_flag(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _patch(monkeypatch, "auth.verify_pak", _resp(200, SimpleNamespace(admin=True, generator=None)))
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_admin")
    with pytest.raises(BrokkrError, match="--generator"):
        await c.apply("payments", _manifests(tmp_path))


async def test_apply_generator_pak_cannot_apply_for_another(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _patch(
        monkeypatch,
        "auth.verify_pak",
        _resp(200, SimpleNamespace(admin=False, generator=str(GENERATOR_ID))),
    )
    _patch(
        monkeypatch,
        "generators.get_generator",
        _resp(200, SimpleNamespace(id=GENERATOR_ID, name="acme")),
    )
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_gen")
    with pytest.raises(BrokkrError, match='belongs to generator "acme"'):
        await c.apply("payments", _manifests(tmp_path), generator="globex")


async def test_apply_refuses_a_stack_owned_by_another_generator(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _patch(monkeypatch, "auth.verify_pak", _resp(200, SimpleNamespace(admin=True, generator=None)))
    _patch(
        monkeypatch,
        "generators.get_generator",
        _resp(200, SimpleNamespace(id=GENERATOR_ID, name="acme")),
    )
    other = UUID("44444444-4444-4444-4444-444444444444")
    _patch(
        monkeypatch,
        "stacks.list_stacks",
        _resp(200, [SimpleNamespace(id=STACK_ID, name="payments", generator_id=other)]),
    )
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_admin")
    with pytest.raises(BrokkrError, match="belongs to another generator"):
        await c.apply("payments", _manifests(tmp_path), generator=str(GENERATOR_ID))
