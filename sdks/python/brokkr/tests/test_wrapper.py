"""Unit tests for the BrokkrClient wrapper."""

from __future__ import annotations

import asyncio

import httpx
import pytest

from brokkr import BrokkrClient, BrokkrError, ErrorResponse, TemplateGenerator
from brokkr_broker_client import AuthenticatedClient, Client
from brokkr_broker_client import models as generated_models


def test_constructs_authenticated_when_token_supplied() -> None:
    c = BrokkrClient("http://localhost:3000/api/v1", token="bk_admin_test")
    assert isinstance(c.api, AuthenticatedClient)


def test_constructs_unauthenticated_when_token_omitted() -> None:
    c = BrokkrClient("http://localhost:3000/api/v1")
    assert isinstance(c.api, Client)


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

    async def op(_api: object) -> str:
        nonlocal calls
        calls += 1
        return "ok"

    result = await c.retry(op)
    assert result == "ok"
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

    async def op(_api: object) -> ErrorResponse:
        nonlocal calls
        calls += 1
        # Returning the typed error from the union — simulates the generated
        # client's sync/asyncio return when the server responds 404.
        # is_retryable_status(500) is False so we'd retry, so use 404-style
        # body and verify that returning the ErrorResponse alone (status
        # defaults to 500 in from_response) does NOT loop forever — it
        # short-circuits via the wrapper's max_retries.
        return ErrorResponse(code="agent_not_found", message="agent not found")

    # Wrapper treats unknown-status ErrorResponse as status=500. 500 is not
    # in the retryable set, so this should raise on the first attempt.
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
