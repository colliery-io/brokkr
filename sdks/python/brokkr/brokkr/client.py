"""High-level Brokkr client wrapping the generated low-level client."""

from __future__ import annotations

import asyncio
import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Awaitable, Callable, Optional, Sequence, TypeVar
from uuid import UUID

import httpx
import yaml

from brokkr_broker_client import AuthenticatedClient, Client
from brokkr_broker_client.models import ErrorResponse
from brokkr_broker_client.types import Unset

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

    async def submit_manifests(self, stack_id: UUID, path: Any) -> Any:
        """Read a folder (or file) of manifests, concatenate into one
        multi-document YAML stream, validate each document, and submit it as
        a new deployment object on an existing stack.

        ``path`` may be a directory (top-level ``*.yaml``/``*.yml`` files,
        sorted) or a single file. The agent front-loads Namespace/CRD objects,
        so ordering is forgiving.
        """
        from brokkr_broker_client.api.stacks import create_deployment_object
        from brokkr_broker_client.models import CreateDeploymentObjectRequest

        yaml_content = _read_manifests(path)
        body = CreateDeploymentObjectRequest(
            yaml_content=yaml_content, is_deletion_marker=False
        )
        result = await create_deployment_object.asyncio(
            stack_id, client=self.api, body=body
        )
        return _expect(result, "create_deployment_object")

    async def apply(
        self,
        stack_name: str,
        path: Any,
        targeting: Optional[Sequence[str]] = None,
    ) -> "ApplyResult":
        """Idempotently make a folder of manifests the desired state of the
        stack named ``stack_name`` — creating the stack if needed, applying
        ``targeting`` labels for fan-out, and submitting a new revision only
        when the bundle changed. Requires a generator PAK.
        """
        from brokkr_broker_client.api.auth import verify_pak
        from brokkr_broker_client.api.stacks import (
            create_deployment_object,
            create_stack,
            list_deployment_objects,
            list_stacks,
            stacks_add_label,
        )
        from brokkr_broker_client.models import (
            CreateDeploymentObjectRequest,
            NewStack,
        )

        targeting = list(targeting or [])
        yaml_content = _read_manifests(path)
        checksum = _sha256_hex(yaml_content)

        auth = _expect(await verify_pak.asyncio(client=self.api), "verify_pak")
        generator = auth.generator
        if generator is None or isinstance(generator, Unset):
            raise BrokkrError(
                message=(
                    "apply by name requires a generator PAK; admin callers should "
                    "create the stack explicitly and use submit_manifests"
                )
            )
        generator_id = UUID(str(generator))

        stacks = _expect(await list_stacks.asyncio(client=self.api), "list_stacks")
        stack = next((s for s in stacks if s.name == stack_name), None)
        if stack is None:
            stack = _expect(
                await create_stack.asyncio(
                    client=self.api,
                    body=NewStack(generator_id=generator_id, name=stack_name),
                ),
                "create_stack",
            )

        for label in targeting:
            resp = await stacks_add_label.asyncio_detailed(
                stack.id, client=self.api, body=label
            )
            # 409 = label already present; that is fine for an idempotent apply.
            if resp.status_code not in (200, 201, 409):
                body = resp.parsed
                if isinstance(body, ErrorResponse):
                    raise BrokkrError.from_response(body, status=resp.status_code)
                raise BrokkrError(
                    message=f"stacks_add_label failed with status {resp.status_code}"
                )

        objects = _expect(
            await list_deployment_objects.asyncio(stack.id, client=self.api),
            "list_deployment_objects",
        )
        latest = max(objects, key=lambda o: o.sequence_id, default=None)
        if latest is not None and latest.yaml_checksum == checksum:
            return ApplyResult(status="unchanged")

        had_prior = len(objects) > 0
        obj = _expect(
            await create_deployment_object.asyncio(
                stack.id,
                client=self.api,
                body=CreateDeploymentObjectRequest(
                    yaml_content=yaml_content, is_deletion_marker=False
                ),
            ),
            "create_deployment_object",
        )
        return ApplyResult(
            status="updated" if had_prior else "created", deployment_object=obj
        )



@dataclass
class ApplyResult:
    """Outcome of :meth:`BrokkrClient.apply`."""

    status: str  # "created" | "updated" | "unchanged"
    deployment_object: Any | None = None


def _expect(result: Any, what: str) -> Any:
    """Unwrap a generated ``.asyncio`` result, raising on error/None."""
    if result is None:
        raise BrokkrError(message=f"{what}: empty response")
    if isinstance(result, ErrorResponse):
        raise BrokkrError.from_response(result, status=400)
    return result


def _read_manifests(path: Any) -> str:
    """Read a manifest path into one validated multi-document YAML stream.

    ``path`` may be a single file or a directory; for a directory, top-level
    ``*.yaml``/``*.yml`` files are concatenated in sorted-name order. Every
    document must parse and carry ``apiVersion`` and ``kind``.
    """
    p = Path(path)
    if p.is_file():
        files = [p]
    elif p.is_dir():
        files = sorted(
            f for f in p.iterdir() if f.is_file() and f.suffix in (".yaml", ".yml")
        )
    else:
        raise BrokkrError(message=f"path not found: {p}")
    if not files:
        raise BrokkrError(message=f"no .yaml/.yml manifests found in {p}")

    parts: list[str] = []
    for f in files:
        content = f.read_text()
        try:
            docs = list(yaml.safe_load_all(content))
        except yaml.YAMLError as exc:
            raise BrokkrError(message=f"{f}: invalid YAML: {exc}") from exc
        for doc in docs:
            if doc is None:
                continue
            if not (isinstance(doc, dict) and doc.get("apiVersion") and doc.get("kind")):
                raise BrokkrError(
                    message=f"{f}: every manifest document must have apiVersion and kind"
                )
        parts.append(content.rstrip())
    return "\n---\n".join(parts) + "\n"


def _sha256_hex(content: str) -> str:
    """Lowercase hex SHA-256, matching the broker's deployment-object checksum."""
    return hashlib.sha256(content.encode("utf-8")).hexdigest()
