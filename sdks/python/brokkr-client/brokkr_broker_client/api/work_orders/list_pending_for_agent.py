from http import HTTPStatus
from typing import Any
from urllib.parse import quote
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.error_response import ErrorResponse
from ...models.work_order import WorkOrder
from ...types import UNSET, Response, Unset


def _get_kwargs(
    agent_id: UUID,
    *,
    work_type: str | Unset = UNSET,
) -> dict[str, Any]:

    params: dict[str, Any] = {}

    params["work_type"] = work_type

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "get",
        "url": "/agents/{agent_id}/work-orders/pending".format(
            agent_id=quote(str(agent_id), safe=""),
        ),
        "params": params,
    }

    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> ErrorResponse | list[WorkOrder] | None:
    if response.status_code == 200:
        response_200 = []
        _response_200 = response.json()
        for response_200_item_data in _response_200:
            response_200_item = WorkOrder.from_dict(response_200_item_data)

            response_200.append(response_200_item)

        return response_200

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


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[ErrorResponse | list[WorkOrder]]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    agent_id: UUID,
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrder]]:
    """
    Args:
        agent_id (UUID):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrder]]
    """

    kwargs = _get_kwargs(
        agent_id=agent_id,
        work_type=work_type,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    agent_id: UUID,
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
) -> ErrorResponse | list[WorkOrder] | None:
    """
    Args:
        agent_id (UUID):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrder]
    """

    return sync_detailed(
        agent_id=agent_id,
        client=client,
        work_type=work_type,
    ).parsed


async def asyncio_detailed(
    agent_id: UUID,
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrder]]:
    """
    Args:
        agent_id (UUID):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrder]]
    """

    kwargs = _get_kwargs(
        agent_id=agent_id,
        work_type=work_type,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    agent_id: UUID,
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
) -> ErrorResponse | list[WorkOrder] | None:
    """
    Args:
        agent_id (UUID):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrder]
    """

    return (
        await asyncio_detailed(
            agent_id=agent_id,
            client=client,
            work_type=work_type,
        )
    ).parsed
