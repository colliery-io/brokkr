"""High-level Brokkr client wrapping the generated low-level client."""

from __future__ import annotations

import asyncio
from typing import Any, Awaitable, Callable, TypeVar

import httpx

from brokkr_broker_client import AuthenticatedClient, Client
from brokkr_broker_client.models import ErrorResponse

from brokkr.errors import BrokkrError

T = TypeVar("T")

# Default timeouts mirror the Rust wrapper.
_DEFAULT_REQUEST_TIMEOUT = 30.0  # seconds
_DEFAULT_CONNECT_TIMEOUT = 10.0  # seconds
_DEFAULT_MAX_RETRIES = 3
_DEFAULT_INITIAL_BACKOFF = 0.2  # seconds
_MAX_BACKOFF = 10.0


class BrokkrClient:
    """Ergonomic Brokkr broker client.

    Construct with a base URL and (optionally) a PAK token. The wrapper
    holds the generated `AuthenticatedClient` (or `Client`, when no token
    is supplied) plus a retry policy. Access the generated API surface via
    `client.api`; reach for the raw httpx session via `client.api.get_httpx_client()`.
    """

    def __init__(
        self,
        base_url: str,
        *,
        token: str | None = None,
        request_timeout: float = _DEFAULT_REQUEST_TIMEOUT,
        connect_timeout: float = _DEFAULT_CONNECT_TIMEOUT,
        max_retries: int = _DEFAULT_MAX_RETRIES,
        initial_backoff: float = _DEFAULT_INITIAL_BACKOFF,
    ) -> None:
        if max_retries < 0:
            raise ValueError("max_retries must be >= 0")
        if initial_backoff <= 0:
            raise ValueError("initial_backoff must be > 0")

        timeout = httpx.Timeout(request_timeout, connect=connect_timeout)
        self._max_retries = max_retries
        self._initial_backoff = initial_backoff

        if token is not None:
            self.api: AuthenticatedClient = AuthenticatedClient(
                base_url=base_url,
                token=token,
                timeout=timeout,
            )
        else:
            # mypy: AuthenticatedClient and Client share most of their surface
            # but are technically distinct types. Users who construct without
            # a token can only call endpoints that don't require auth.
            self.api = Client(base_url=base_url, timeout=timeout)  # type: ignore[assignment]

    @property
    def max_retries(self) -> int:
        return self._max_retries

    @property
    def initial_backoff(self) -> float:
        return self._initial_backoff

    async def retry(self, op: Callable[[Any], Awaitable[T]]) -> T:
        """Run ``op(client)`` with exponential backoff on retryable failures.

        ``op`` is an async callable that takes the generated client
        (``self.api``) and returns the operation's result. The closure form
        keeps the wrapper free of per-endpoint glue while letting callers
        decide which operations are safe to retry. Non-idempotent POSTs
        should generally **not** be wrapped.

        Retry classification matches the Rust wrapper: transport errors
        and HTTP 408/429/502/503/504 qualify. Other 4xx/5xx responses
        return immediately on the first failure.
        """
        attempt = 0
        while True:
            try:
                result = await op(self.api)
            except httpx.HTTPError as exc:
                err = BrokkrError.from_transport(exc)
                if not err.is_retryable() or attempt >= self._max_retries:
                    raise err from exc
            else:
                if isinstance(result, ErrorResponse):
                    # The generator folds documented errors into return
                    # unions; we surface them as raises here. We don't know
                    # the status code in this codepath — the *_detailed
                    # variants carry it. Callers wanting status-aware retry
                    # should use those and convert manually.
                    err = BrokkrError.from_response(result, status=500)
                    if not err.is_retryable() or attempt >= self._max_retries:
                        raise err
                else:
                    return result

            backoff = min(self._initial_backoff * (2**attempt), _MAX_BACKOFF)
            await asyncio.sleep(backoff)
            attempt += 1
