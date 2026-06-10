"""High-level convenience helpers for the internal-WS-channel surface
(BROKKR-I-0019).

The bulk of this SDK is generated from ``openapi/brokkr-v1.json`` and
lives under :mod:`brokkr_broker_client.api`. Those generated modules
are correct but verbose — each operation needs a separate import and a
specific ``client`` keyword. This module collapses the most common
calls into one-line helpers that mirror the Rust
(``crates/brokkr-client::BrokkrClient``) and TypeScript
(``sdks/typescript/brokkr-client::BrokkrClient``) wrappers.

Pattern: pass an :class:`AuthenticatedClient` plus the call arguments;
get back the parsed response type. Errors raise ``UnexpectedStatus``
from the generated layer; callers handle.

Per ADR-0008 / ``project_log_retention_stance``: telemetry retention is
capped at 6 hours. Responses carry a ``retention`` block — surface it
in any tool built on this SDK so users aren't surprised by missing
rows.
"""

from __future__ import annotations

import datetime
from typing import TYPE_CHECKING
from uuid import UUID

if TYPE_CHECKING:
    from .client import AuthenticatedClient
    from .models.k8s_event_history_response import K8SEventHistoryResponse
    from .models.pod_log_history_response import PodLogHistoryResponse
    from .models.ws_connections_response import WsConnectionsResponse


def list_telemetry_events(
    client: AuthenticatedClient,
    stack_id: UUID,
    *,
    since: datetime.datetime | None = None,
    limit: int | None = None,
) -> K8SEventHistoryResponse:
    """Paginated kube-event history for a stack within the 6h retention window."""
    from .api.stack_telemetry import list_telemetry_events as _op
    from .types import UNSET

    result = _op.sync(
        id=stack_id,
        client=client,
        since=since if since is not None else UNSET,
        limit=limit if limit is not None else UNSET,
    )
    if result is None:
        raise RuntimeError("list_telemetry_events returned None — check broker response")
    return result


async def list_telemetry_events_async(
    client: AuthenticatedClient,
    stack_id: UUID,
    *,
    since: datetime.datetime | None = None,
    limit: int | None = None,
) -> K8SEventHistoryResponse:
    """Async variant of :func:`list_telemetry_events`."""
    from .api.stack_telemetry import list_telemetry_events as _op
    from .types import UNSET

    result = await _op.asyncio(
        id=stack_id,
        client=client,
        since=since if since is not None else UNSET,
        limit=limit if limit is not None else UNSET,
    )
    if result is None:
        raise RuntimeError("list_telemetry_events returned None — check broker response")
    return result


def list_telemetry_logs(
    client: AuthenticatedClient,
    stack_id: UUID,
    *,
    since: datetime.datetime | None = None,
    limit: int | None = None,
) -> PodLogHistoryResponse:
    """Paginated pod-log history for a stack within the 6h retention window."""
    from .api.stack_telemetry import list_telemetry_logs as _op
    from .types import UNSET

    result = _op.sync(
        id=stack_id,
        client=client,
        since=since if since is not None else UNSET,
        limit=limit if limit is not None else UNSET,
    )
    if result is None:
        raise RuntimeError("list_telemetry_logs returned None — check broker response")
    return result


async def list_telemetry_logs_async(
    client: AuthenticatedClient,
    stack_id: UUID,
    *,
    since: datetime.datetime | None = None,
    limit: int | None = None,
) -> PodLogHistoryResponse:
    """Async variant of :func:`list_telemetry_logs`."""
    from .api.stack_telemetry import list_telemetry_logs as _op
    from .types import UNSET

    result = await _op.asyncio(
        id=stack_id,
        client=client,
        since=since if since is not None else UNSET,
        limit=limit if limit is not None else UNSET,
    )
    if result is None:
        raise RuntimeError("list_telemetry_logs returned None — check broker response")
    return result


def list_ws_connections(client: AuthenticatedClient) -> WsConnectionsResponse:
    """Admin-only snapshot of currently-connected agents on the internal WS channel.

    For continuous monitoring prefer scraping the
    ``brokkr_ws_connected_agents`` Prometheus gauge instead of polling this
    endpoint.
    """
    from .api.admin import list_ws_connections as _op

    result = _op.sync(client=client)
    if result is None:
        raise RuntimeError("list_ws_connections returned None — check broker response")
    return result


async def list_ws_connections_async(
    client: AuthenticatedClient,
) -> WsConnectionsResponse:
    """Async variant of :func:`list_ws_connections`."""
    from .api.admin import list_ws_connections as _op

    result = await _op.asyncio(client=client)
    if result is None:
        raise RuntimeError("list_ws_connections returned None — check broker response")
    return result


def live_subscription_url(base_url: str, stack_id: UUID) -> str:
    """Compute the WebSocket URL for a stack's live event + log tail.

    The base URL is the broker's HTTP root (e.g. ``http://broker:3000``
    or ``https://broker.example.com``). The scheme is swapped to
    ``ws``/``wss`` and the live endpoint path is appended. Callers
    open the WebSocket themselves using their preferred client
    (``websockets``, ``aiohttp``, etc.) and authenticate via the
    ``Authorization: Bearer <pak>`` header.
    """
    root = base_url.rstrip("/")
    if root.startswith("https://"):
        ws_root = "wss://" + root[len("https://") :]
    elif root.startswith("http://"):
        ws_root = "ws://" + root[len("http://") :]
    else:
        ws_root = "ws://" + root
    return f"{ws_root}/api/v1/stacks/{stack_id}/live"


__all__ = (
    "list_telemetry_events",
    "list_telemetry_events_async",
    "list_telemetry_logs",
    "list_telemetry_logs_async",
    "list_ws_connections",
    "list_ws_connections_async",
    "live_subscription_url",
)
