from http import HTTPStatus
from typing import Any
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.error_response import ErrorResponse
from ...models.work_order_log import WorkOrderLog
from ...types import UNSET, Response, Unset


def _get_kwargs(
    *,
    work_type: str | Unset = UNSET,
    success: bool | Unset = UNSET,
    agent_id: UUID | Unset = UNSET,
    limit: int | Unset = UNSET,
) -> dict[str, Any]:

    params: dict[str, Any] = {}

    params["work_type"] = work_type

    params["success"] = success

    json_agent_id: str | Unset = UNSET
    if not isinstance(agent_id, Unset):
        json_agent_id = str(agent_id)
    params["agent_id"] = json_agent_id

    params["limit"] = limit

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "get",
        "url": "/work-order-log",
        "params": params,
    }

    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> ErrorResponse | list[WorkOrderLog] | None:
    if response.status_code == 200:
        response_200 = []
        _response_200 = response.json()
        for response_200_item_data in _response_200:
            response_200_item = WorkOrderLog.from_dict(response_200_item_data)

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
) -> Response[ErrorResponse | list[WorkOrderLog]]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
    success: bool | Unset = UNSET,
    agent_id: UUID | Unset = UNSET,
    limit: int | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrderLog]]:
    """
    Args:
        work_type (str | Unset):
        success (bool | Unset):
        agent_id (UUID | Unset):
        limit (int | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrderLog]]
    """

    kwargs = _get_kwargs(
        work_type=work_type,
        success=success,
        agent_id=agent_id,
        limit=limit,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
    success: bool | Unset = UNSET,
    agent_id: UUID | Unset = UNSET,
    limit: int | Unset = UNSET,
) -> ErrorResponse | list[WorkOrderLog] | None:
    """
    Args:
        work_type (str | Unset):
        success (bool | Unset):
        agent_id (UUID | Unset):
        limit (int | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrderLog]
    """

    return sync_detailed(
        client=client,
        work_type=work_type,
        success=success,
        agent_id=agent_id,
        limit=limit,
    ).parsed


async def asyncio_detailed(
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
    success: bool | Unset = UNSET,
    agent_id: UUID | Unset = UNSET,
    limit: int | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrderLog]]:
    """
    Args:
        work_type (str | Unset):
        success (bool | Unset):
        agent_id (UUID | Unset):
        limit (int | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrderLog]]
    """

    kwargs = _get_kwargs(
        work_type=work_type,
        success=success,
        agent_id=agent_id,
        limit=limit,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    *,
    client: AuthenticatedClient,
    work_type: str | Unset = UNSET,
    success: bool | Unset = UNSET,
    agent_id: UUID | Unset = UNSET,
    limit: int | Unset = UNSET,
) -> ErrorResponse | list[WorkOrderLog] | None:
    """
    Args:
        work_type (str | Unset):
        success (bool | Unset):
        agent_id (UUID | Unset):
        limit (int | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrderLog]
    """

    return (
        await asyncio_detailed(
            client=client,
            work_type=work_type,
            success=success,
            agent_id=agent_id,
            limit=limit,
        )
    ).parsed
