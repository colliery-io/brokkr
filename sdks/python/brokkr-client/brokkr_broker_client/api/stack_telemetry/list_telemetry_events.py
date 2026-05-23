import datetime
from http import HTTPStatus
from typing import Any
from urllib.parse import quote
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.error_response import ErrorResponse
from ...models.k8s_event_history_response import K8SEventHistoryResponse
from ...types import UNSET, Response, Unset


def _get_kwargs(
    id: UUID,
    *,
    since: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
) -> dict[str, Any]:

    params: dict[str, Any] = {}

    json_since: None | str | Unset
    if isinstance(since, Unset):
        json_since = UNSET
    elif isinstance(since, datetime.datetime):
        json_since = since.isoformat()
    else:
        json_since = since
    params["since"] = json_since

    json_limit: int | None | Unset
    if isinstance(limit, Unset):
        json_limit = UNSET
    else:
        json_limit = limit
    params["limit"] = json_limit

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "get",
        "url": "/stacks/{id}/events".format(
            id=quote(str(id), safe=""),
        ),
        "params": params,
    }

    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> ErrorResponse | K8SEventHistoryResponse | None:
    if response.status_code == 200:
        response_200 = K8SEventHistoryResponse.from_dict(response.json())

        return response_200

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 404:
        response_404 = ErrorResponse.from_dict(response.json())

        return response_404

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[ErrorResponse | K8SEventHistoryResponse]:
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
    since: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
) -> Response[ErrorResponse | K8SEventHistoryResponse]:
    """
    Args:
        id (UUID):
        since (datetime.datetime | None | Unset):
        limit (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | K8SEventHistoryResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        since=since,
        limit=limit,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    id: UUID,
    *,
    client: AuthenticatedClient,
    since: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
) -> ErrorResponse | K8SEventHistoryResponse | None:
    """
    Args:
        id (UUID):
        since (datetime.datetime | None | Unset):
        limit (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | K8SEventHistoryResponse
    """

    return sync_detailed(
        id=id,
        client=client,
        since=since,
        limit=limit,
    ).parsed


async def asyncio_detailed(
    id: UUID,
    *,
    client: AuthenticatedClient,
    since: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
) -> Response[ErrorResponse | K8SEventHistoryResponse]:
    """
    Args:
        id (UUID):
        since (datetime.datetime | None | Unset):
        limit (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | K8SEventHistoryResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        since=since,
        limit=limit,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    id: UUID,
    *,
    client: AuthenticatedClient,
    since: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
) -> ErrorResponse | K8SEventHistoryResponse | None:
    """
    Args:
        id (UUID):
        since (datetime.datetime | None | Unset):
        limit (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | K8SEventHistoryResponse
    """

    return (
        await asyncio_detailed(
            id=id,
            client=client,
            since=since,
            limit=limit,
        )
    ).parsed
