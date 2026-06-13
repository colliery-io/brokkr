from http import HTTPStatus
from typing import Any, cast
from urllib.parse import quote
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.error_response import ErrorResponse
from ...models.heartbeat_report import HeartbeatReport
from ...types import Response


def _get_kwargs(
    id: UUID,
    *,
    body: HeartbeatReport,
) -> dict[str, Any]:
    headers: dict[str, Any] = {}

    _kwargs: dict[str, Any] = {
        "method": "post",
        "url": "/agents/{id}/heartbeat".format(
            id=quote(str(id), safe=""),
        ),
    }

    _kwargs["json"] = body.to_dict()

    headers["Content-Type"] = "application/json"

    _kwargs["headers"] = headers
    return _kwargs


def _parse_response(*, client: AuthenticatedClient | Client, response: httpx.Response) -> Any | ErrorResponse | None:
    if response.status_code == 204:
        response_204 = cast(Any, None)
        return response_204

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 500:
        response_500 = ErrorResponse.from_dict(response.json())

        return response_500

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(*, client: AuthenticatedClient | Client, response: httpx.Response) -> Response[Any | ErrorResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: HeartbeatReport,
) -> Response[Any | ErrorResponse]:
    """
    Args:
        id (UUID):
        body (HeartbeatReport): Optional heartbeat report body (BROKKR-T-0227).

            A plain heartbeat carries no body; agents that probe their own Kubernetes
            API attach this to self-report reachability. Both fields are optional so a
            body may carry only what the agent could measure, and the entire body may
            be omitted (legacy/no-body heartbeats still work).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[Any | ErrorResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        body=body,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: HeartbeatReport,
) -> Any | ErrorResponse | None:
    """
    Args:
        id (UUID):
        body (HeartbeatReport): Optional heartbeat report body (BROKKR-T-0227).

            A plain heartbeat carries no body; agents that probe their own Kubernetes
            API attach this to self-report reachability. Both fields are optional so a
            body may carry only what the agent could measure, and the entire body may
            be omitted (legacy/no-body heartbeats still work).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Any | ErrorResponse
    """

    return sync_detailed(
        id=id,
        client=client,
        body=body,
    ).parsed


async def asyncio_detailed(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: HeartbeatReport,
) -> Response[Any | ErrorResponse]:
    """
    Args:
        id (UUID):
        body (HeartbeatReport): Optional heartbeat report body (BROKKR-T-0227).

            A plain heartbeat carries no body; agents that probe their own Kubernetes
            API attach this to self-report reachability. Both fields are optional so a
            body may carry only what the agent could measure, and the entire body may
            be omitted (legacy/no-body heartbeats still work).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[Any | ErrorResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        body=body,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: HeartbeatReport,
) -> Any | ErrorResponse | None:
    """
    Args:
        id (UUID):
        body (HeartbeatReport): Optional heartbeat report body (BROKKR-T-0227).

            A plain heartbeat carries no body; agents that probe their own Kubernetes
            API attach this to self-report reachability. Both fields are optional so a
            body may carry only what the agent could measure, and the entire body may
            be omitted (legacy/no-body heartbeats still work).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Any | ErrorResponse
    """

    return (
        await asyncio_detailed(
            id=id,
            client=client,
            body=body,
        )
    ).parsed
