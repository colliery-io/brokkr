from http import HTTPStatus
from typing import Any

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.error_response import ErrorResponse
from ...models.work_order import WorkOrder
from ...types import UNSET, Response, Unset


def _get_kwargs(
    *,
    status: str | Unset = UNSET,
    work_type: str | Unset = UNSET,
) -> dict[str, Any]:

    params: dict[str, Any] = {}

    params["status"] = status

    params["work_type"] = work_type

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "get",
        "url": "/work-orders",
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
    *,
    client: AuthenticatedClient,
    status: str | Unset = UNSET,
    work_type: str | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrder]]:
    """
    Args:
        status (str | Unset):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrder]]
    """

    kwargs = _get_kwargs(
        status=status,
        work_type=work_type,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    *,
    client: AuthenticatedClient,
    status: str | Unset = UNSET,
    work_type: str | Unset = UNSET,
) -> ErrorResponse | list[WorkOrder] | None:
    """
    Args:
        status (str | Unset):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrder]
    """

    return sync_detailed(
        client=client,
        status=status,
        work_type=work_type,
    ).parsed


async def asyncio_detailed(
    *,
    client: AuthenticatedClient,
    status: str | Unset = UNSET,
    work_type: str | Unset = UNSET,
) -> Response[ErrorResponse | list[WorkOrder]]:
    """
    Args:
        status (str | Unset):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | list[WorkOrder]]
    """

    kwargs = _get_kwargs(
        status=status,
        work_type=work_type,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    *,
    client: AuthenticatedClient,
    status: str | Unset = UNSET,
    work_type: str | Unset = UNSET,
) -> ErrorResponse | list[WorkOrder] | None:
    """
    Args:
        status (str | Unset):
        work_type (str | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | list[WorkOrder]
    """

    return (
        await asyncio_detailed(
            client=client,
            status=status,
            work_type=work_type,
        )
    ).parsed
