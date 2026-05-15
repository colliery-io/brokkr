from http import HTTPStatus
from typing import Any
from urllib.parse import quote
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.deployment_object import DeploymentObject
from ...models.error_response import ErrorResponse
from ...models.template_instantiation_request import TemplateInstantiationRequest
from ...types import Response


def _get_kwargs(
    stack_id: UUID,
    *,
    body: TemplateInstantiationRequest,
) -> dict[str, Any]:
    headers: dict[str, Any] = {}

    _kwargs: dict[str, Any] = {
        "method": "post",
        "url": "/stacks/{stack_id}/deployment-objects/from-template".format(
            stack_id=quote(str(stack_id), safe=""),
        ),
    }

    _kwargs["json"] = body.to_dict()

    headers["Content-Type"] = "application/json"

    _kwargs["headers"] = headers
    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> DeploymentObject | ErrorResponse | None:
    if response.status_code == 201:
        response_201 = DeploymentObject.from_dict(response.json())

        return response_201

    if response.status_code == 400:
        response_400 = ErrorResponse.from_dict(response.json())

        return response_400

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 404:
        response_404 = ErrorResponse.from_dict(response.json())

        return response_404

    if response.status_code == 422:
        response_422 = ErrorResponse.from_dict(response.json())

        return response_422

    if response.status_code == 500:
        response_500 = ErrorResponse.from_dict(response.json())

        return response_500

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[DeploymentObject | ErrorResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    stack_id: UUID,
    *,
    client: AuthenticatedClient,
    body: TemplateInstantiationRequest,
) -> Response[DeploymentObject | ErrorResponse]:
    """
    Args:
        stack_id (UUID):
        body (TemplateInstantiationRequest):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[DeploymentObject | ErrorResponse]
    """

    kwargs = _get_kwargs(
        stack_id=stack_id,
        body=body,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    stack_id: UUID,
    *,
    client: AuthenticatedClient,
    body: TemplateInstantiationRequest,
) -> DeploymentObject | ErrorResponse | None:
    """
    Args:
        stack_id (UUID):
        body (TemplateInstantiationRequest):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        DeploymentObject | ErrorResponse
    """

    return sync_detailed(
        stack_id=stack_id,
        client=client,
        body=body,
    ).parsed


async def asyncio_detailed(
    stack_id: UUID,
    *,
    client: AuthenticatedClient,
    body: TemplateInstantiationRequest,
) -> Response[DeploymentObject | ErrorResponse]:
    """
    Args:
        stack_id (UUID):
        body (TemplateInstantiationRequest):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[DeploymentObject | ErrorResponse]
    """

    kwargs = _get_kwargs(
        stack_id=stack_id,
        body=body,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    stack_id: UUID,
    *,
    client: AuthenticatedClient,
    body: TemplateInstantiationRequest,
) -> DeploymentObject | ErrorResponse | None:
    """
    Args:
        stack_id (UUID):
        body (TemplateInstantiationRequest):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        DeploymentObject | ErrorResponse
    """

    return (
        await asyncio_detailed(
            stack_id=stack_id,
            client=client,
            body=body,
        )
    ).parsed
